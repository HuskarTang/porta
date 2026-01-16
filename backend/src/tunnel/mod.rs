use anyhow::{anyhow, Result};
use std::{collections::HashSet, net::SocketAddr, sync::OnceLock};
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

static ACTIVE_MAPPINGS: OnceLock<Mutex<HashSet<u16>>> = OnceLock::new();

fn mapping_registry() -> &'static Mutex<HashSet<u16>> {
    ACTIVE_MAPPINGS.get_or_init(|| Mutex::new(HashSet::new()))
}

pub async fn ensure_mapping(local_port: u16, remote_addr: String) -> Result<()> {
    let mut registry = mapping_registry().lock().await;
    if registry.contains(&local_port) {
        return Ok(());
    }
    registry.insert(local_port);
    drop(registry);

    let remote_socket = parse_remote_addr(&remote_addr)?;
    tokio::spawn(async move {
        let listener = match TcpListener::bind(("0.0.0.0", local_port)).await {
            Ok(listener) => listener,
            Err(_) => return,
        };
        loop {
            let (mut inbound, _) = match listener.accept().await {
                Ok(pair) => pair,
                Err(_) => break,
            };
            let remote = remote_socket;
            tokio::spawn(async move {
                if let Ok(mut outbound) = TcpStream::connect(remote).await {
                    let _ = copy_bidirectional(&mut inbound, &mut outbound).await;
                }
            });
        }
    });

    Ok(())
}

fn parse_remote_addr(remote_addr: &str) -> Result<SocketAddr> {
    remote_addr
        .parse::<SocketAddr>()
        .map_err(|_| anyhow!("无法解析远端地址: {}", remote_addr))
}
