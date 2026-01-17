use anyhow::Result;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

pub struct ProxyServer {
    listen_addr: SocketAddr,
    running: Arc<Mutex<bool>>,
}

impl ProxyServer {
    pub fn new(port: u16) -> Self {
        Self {
            listen_addr: SocketAddr::from(([0, 0, 0, 0], port)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        let listener = TcpListener::bind(self.listen_addr).await?;
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            loop {
                if !*running_flag.lock().await {
                    break;
                }
                let (socket, _) = match listener.accept().await {
                    Ok(pair) => pair,
                    Err(_) => break,
                };
                tokio::spawn(handle_proxy_connection(socket));
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        *running = false;
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }
}

async fn handle_proxy_connection(mut socket: TcpStream) {
    let mut buf = [0u8; 1];
    if socket.read_exact(&mut buf).await.is_err() {
        return;
    }

    match buf[0] {
        0x05 => {
            handle_socks5(socket).await;
        }
        b'C' | b'G' | b'P' | b'H' | b'D' | b'O' | b'T' => {
            handle_http_proxy(socket, buf[0]).await;
        }
        _ => {}
    }
}

async fn handle_socks5(mut socket: TcpStream) {
    let mut buf = [0u8; 256];
    if socket.read(&mut buf[..1]).await.is_err() {
        return;
    }
    let nmethods = buf[0];
    if socket
        .read_exact(&mut buf[..nmethods as usize])
        .await
        .is_err()
    {
        return;
    }
    if socket.write_all(&[0x05, 0x00]).await.is_err() {
        return;
    }
    if socket.read_exact(&mut buf[..4]).await.is_err() {
        return;
    }
    let cmd = buf[1];
    let atyp = buf[3];
    if cmd != 0x01 {
        let _ = socket.write_all(&[0x05, 0x07]).await;
        return;
    }
    let target = match atyp {
        0x01 => {
            if socket.read_exact(&mut buf[..6]).await.is_err() {
                return;
            }
            let ip = std::net::Ipv4Addr::new(buf[0], buf[1], buf[2], buf[3]);
            let port = u16::from_be_bytes([buf[4], buf[5]]);
            SocketAddr::from((ip, port))
        }
        0x03 => {
            if socket.read_exact(&mut buf[..1]).await.is_err() {
                return;
            }
            let len = buf[0] as usize;
            if socket.read_exact(&mut buf[..len + 2]).await.is_err() {
                return;
            }
            let domain = String::from_utf8_lossy(&buf[..len]);
            let port = u16::from_be_bytes([buf[len], buf[len + 1]]);
            match tokio::net::lookup_host(format!("{}:{}", domain, port))
                .await
                .ok()
                .and_then(|mut iter| iter.next())
            {
                Some(addr) => addr,
                None => {
                    let _ = socket.write_all(&[0x05, 0x04]).await;
                    return;
                }
            }
        }
        _ => {
            let _ = socket.write_all(&[0x05, 0x08]).await;
            return;
        }
    };

    let remote = match TcpStream::connect(target).await {
        Ok(stream) => stream,
        Err(_) => {
            let _ = socket.write_all(&[0x05, 0x04]).await;
            return;
        }
    };

    let reply = [0x05, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    if socket.write_all(&reply).await.is_err() {
        return;
    }

    let (mut client_read, mut client_write) = socket.into_split();
    let (mut remote_read, mut remote_write) = remote.into_split();

    let client_to_remote = tokio::io::copy(&mut client_read, &mut remote_write);
    let remote_to_client = tokio::io::copy(&mut remote_read, &mut client_write);

    let _ = tokio::try_join!(client_to_remote, remote_to_client);
}

async fn handle_http_proxy(mut socket: TcpStream, first_byte: u8) {
    let mut buf = vec![first_byte];
    let mut line_buf = vec![0u8; 8192];

    let n = match socket.read(&mut line_buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    buf.extend_from_slice(&line_buf[..n]);

    let request_line = String::from_utf8_lossy(&buf);
    let lines: Vec<&str> = request_line.lines().collect();
    if lines.is_empty() {
        return;
    }

    let parts: Vec<&str> = lines[0].split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let url = parts[1];

    if method == "CONNECT" {
        let target = match url.parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(_) => {
                if let Some((host, port_str)) = url.split_once(':') {
                    let port = port_str.parse::<u16>().unwrap_or(443);
                    match tokio::net::lookup_host(format!("{}:{}", host, port))
                        .await
                        .ok()
                        .and_then(|mut iter| iter.next())
                    {
                        Some(addr) => addr,
                        None => return,
                    }
                } else {
                    return;
                }
            }
        };

        let remote = match TcpStream::connect(target).await {
            Ok(stream) => stream,
            Err(_) => return,
        };

        if socket
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await
            .is_err()
        {
            return;
        }

        let (mut client_read, mut client_write) = socket.into_split();
        let (mut remote_read, mut remote_write) = remote.into_split();

        let client_to_remote = tokio::io::copy(&mut client_read, &mut remote_write);
        let remote_to_client = tokio::io::copy(&mut remote_read, &mut client_write);

        let _ = tokio::try_join!(client_to_remote, remote_to_client);
    }
}
