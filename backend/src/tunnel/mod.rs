use anyhow::Result;
use std::{collections::HashSet, sync::OnceLock};
use tokio::{
    io::copy_bidirectional,
    net::TcpListener,
    sync::Mutex,
};
use tokio_util::compat::FuturesAsyncReadCompatExt;

use libp2p::PeerId;

use crate::p2p::NodeHandle;

static ACTIVE_MAPPINGS: OnceLock<Mutex<HashSet<u16>>> = OnceLock::new();

fn mapping_registry() -> &'static Mutex<HashSet<u16>> {
    ACTIVE_MAPPINGS.get_or_init(|| Mutex::new(HashSet::new()))
}

pub async fn ensure_stream_mapping(
    local_port: u16,
    peer_id: PeerId,
    service_uuid: String,
    p2p: NodeHandle,
) -> Result<()> {
    let mut registry = mapping_registry().lock().await;
    if registry.contains(&local_port) {
        return Ok(());
    }
    registry.insert(local_port);
    drop(registry);

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
            let peer = peer_id;
            let service = service_uuid.clone();
            let p2p = p2p.clone();
            tokio::spawn(async move {
                if let Ok(stream) = p2p.open_stream(peer, &service).await {
                    let mut stream = stream.compat();
                    let _ = copy_bidirectional(&mut inbound, &mut stream).await;
                }
            });
        }
    });

    Ok(())
}

pub async fn ensure_secure_mapping(
    local_port: u16,
    first_relay_peer: PeerId,
    service_uuid: String,
    relay_chain: Vec<String>,
    p2p: NodeHandle,
) -> Result<()> {
    let mut registry = mapping_registry().lock().await;
    if registry.contains(&local_port) {
        return Ok(());
    }
    registry.insert(local_port);
    drop(registry);

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
            let first_peer = first_relay_peer;
            let service = service_uuid.clone();
            let chain = relay_chain.clone();
            let p2p = p2p.clone();
            tokio::spawn(async move {
                let stream_protocol = build_relay_protocol(&service, &chain);
                if let Ok(stream) = p2p.open_stream(first_peer, &stream_protocol).await {
                    let mut stream = stream.compat();
                    let _ = copy_bidirectional(&mut inbound, &mut stream).await;
                }
            });
        }
    });

    Ok(())
}

fn build_relay_protocol(service_uuid: &str, relay_chain: &[String]) -> String {
    if relay_chain.is_empty() {
        service_uuid.to_string()
    } else {
        format!("{}|relay:{}", service_uuid, relay_chain.join(","))
    }
}
