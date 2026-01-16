use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use libp2p::futures::StreamExt;
use libp2p::{
    identify, identity,
    multiaddr::Protocol,
    ping,
    request_response::{
        Behaviour as RequestResponse, Config as RequestResponseConfig, Event as RequestResponseEvent,
        Message as RequestResponseMessage, OutboundRequestId, ProtocolSupport,
    },
    swarm::{NetworkBehaviour, Swarm, SwarmEvent, Stream, StreamProtocol},
    tcp, yamux, Multiaddr, PeerId, Transport,
};
use libp2p_stream::{Behaviour as StreamBehaviour, Control as StreamControl};
use tokio::sync::{mpsc, oneshot};
use libp2p::futures::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::compat::FuturesAsyncReadCompatExt;

use crate::{
    models::ServiceRegistryItem,
    state::Store,
};

use super::protocol::{JsonCodec, P2pRequest, P2pResponse, PortaProtocol, ServiceAnnouncement};
use super::STREAM_PROTOCOL;

#[derive(NetworkBehaviour)]
struct PortaBehaviour {
    request_response: RequestResponse<JsonCodec>,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    stream: StreamBehaviour,
}

enum Command {
    Dial {
        addr: Multiaddr,
        respond_to: oneshot::Sender<Result<PeerId>>,
    },
    Request {
        peer: PeerId,
        request: P2pRequest,
        respond_to: oneshot::Sender<Result<P2pResponse>>,
    },
}

#[derive(Clone)]
pub struct NodeHandle {
    sender: mpsc::Sender<Command>,
    peer_id: String,
    stream_control: Arc<tokio::sync::Mutex<StreamControl>>,
}

impl NodeHandle {
    pub async fn spawn(store: Arc<dyn Store>) -> Result<Self> {
        let keypair = load_or_generate_keypair(&store).await?;
        let peer_id = PeerId::from(keypair.public());

        let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(libp2p::noise::Config::new(&keypair)?)
            .multiplex(yamux::Config::default())
            .boxed();

        let rr_config = RequestResponseConfig::default();
        let protocols = std::iter::once((PortaProtocol("/porta/req/1"), ProtocolSupport::Full));
        let request_response = RequestResponse::new(protocols, rr_config);

        let stream = StreamBehaviour::new();
        let mut stream_control = stream.new_control();
        let behaviour = PortaBehaviour {
            request_response,
            ping: ping::Behaviour::new(ping::Config::new()),
            identify: identify::Behaviour::new(
                identify::Config::new("/porta/1.0".into(), keypair.public()),
            ),
            stream,
        };

        let mut swarm = Swarm::new(
            transport,
            behaviour,
            peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        let (sender, mut receiver) = mpsc::channel(32);
        let mut pending: HashMap<OutboundRequestId, oneshot::Sender<Result<P2pResponse>>> =
            HashMap::new();

        let store_clone = store.clone();
        let mut incoming = match stream_control.accept(StreamProtocol::new(STREAM_PROTOCOL)) {
            Ok(incoming) => incoming,
            Err(_) => {
                return Err(anyhow!("重复注册 stream 协议"));
            }
        };
        let store_for_streams = store.clone();
        let stream_control_for_relay = stream_control.clone();
        tokio::spawn(async move {
            while let Some((peer, stream)) = incoming.next().await {
                handle_incoming_stream(peer, stream, &store_for_streams, stream_control_for_relay.clone()).await;
            }
        });
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(cmd) = receiver.recv() => {
                        match cmd {
                            Command::Dial { addr, respond_to } => {
                                let peer = peer_id_from_addr(&addr)
                                    .ok_or_else(|| anyhow!("multiaddr 缺少 /p2p/peerId"));
                                if let Ok(peer_id) = peer {
                                    if let Err(err) = swarm.dial(addr) {
                                        let _ = respond_to.send(Err(err.into()));
                                    } else {
                                        let _ = respond_to.send(Ok(peer_id));
                                    }
                                } else {
                                    let _ = respond_to.send(Err(peer.err().unwrap()));
                                }
                            }
                            Command::Request { peer, request, respond_to } => {
                                let request_id = swarm.behaviour_mut().request_response.send_request(&peer, request);
                                pending.insert(request_id, respond_to);
                            }
                        }
                    }
                    event = swarm.select_next_some() => match event {
                        SwarmEvent::Behaviour(PortaBehaviourEvent::RequestResponse(event)) => {
                            handle_request_response_event(event, &mut swarm, &store_clone, &mut pending).await;
                        }
                        _ => {}
                    }
                }
            }
        });

        Ok(Self {
            sender,
            peer_id: peer_id.to_string(),
            stream_control: Arc::new(tokio::sync::Mutex::new(stream_control)),
        })
    }

    pub async fn dial(&self, addr: Multiaddr) -> Result<PeerId> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(Command::Dial { addr, respond_to: tx })
            .await
            .map_err(|_| anyhow!("p2p 通道已关闭"))?;
        rx.await.map_err(|_| anyhow!("p2p dial 失败"))?
    }

    pub async fn request(&self, peer: PeerId, request: P2pRequest) -> Result<P2pResponse> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(Command::Request {
                peer,
                request,
                respond_to: tx,
            })
            .await
            .map_err(|_| anyhow!("p2p 通道已关闭"))?;
        rx.await.map_err(|_| anyhow!("p2p request 失败"))?
    }

    pub fn peer_id(&self) -> String {
        self.peer_id.clone()
    }

    pub async fn open_stream(&self, peer: PeerId, service_uuid: &str) -> Result<Stream> {
        let protocol = StreamProtocol::new(STREAM_PROTOCOL);
        let mut control = self.stream_control.lock().await;
        let mut stream = control
            .open_stream(peer, protocol)
            .await
            .map_err(|err| anyhow!("打开流失败: {}", err))?;
        write_service_uuid(&mut stream, service_uuid).await?;
        Ok(stream)
    }
}

async fn handle_request_response_event(
    event: RequestResponseEvent<P2pRequest, P2pResponse>,
    swarm: &mut Swarm<PortaBehaviour>,
    store: &Arc<dyn Store>,
    pending: &mut HashMap<OutboundRequestId, oneshot::Sender<Result<P2pResponse>>>,
) {
    match event {
        RequestResponseEvent::Message { peer, message } => match message {
            RequestResponseMessage::Request {
                request, channel, ..
            } => {
                let response = handle_inbound_request(store, &peer, request).await;
                let _ = swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, response);
            }
            RequestResponseMessage::Response {
                request_id,
                response,
            } => {
                if let Some(ch) = pending.remove(&request_id) {
                    let _ = ch.send(Ok(response));
                }
            }
        },
        RequestResponseEvent::OutboundFailure { request_id, error, .. } => {
            if let Some(ch) = pending.remove(&request_id) {
                let _ = ch.send(Err(anyhow!("请求失败: {:?}", error)));
            }
        }
        RequestResponseEvent::InboundFailure { error, .. } => {
            let _ = error;
        }
        RequestResponseEvent::ResponseSent { .. } => {}
    }
}

async fn handle_incoming_stream(
    peer: PeerId,
    mut stream: Stream,
    store: &Arc<dyn Store>,
    mut stream_control: StreamControl,
) {
    if store.peer_is_banned(&peer.to_string()).await.unwrap_or(false) {
        return;
    }
    let role = store.peer_role(&peer.to_string()).await.ok().flatten();
    if role.as_deref() != Some("edge") {
        return;
    }
    let protocol = match read_service_uuid(&mut stream).await {
        Ok(uuid) => uuid,
        Err(_) => return,
    };
    
    if protocol.contains("|relay:") {
        let parts: Vec<&str> = protocol.split("|relay:").collect();
        if parts.len() != 2 {
            return;
        }
        let service_uuid = parts[0];
        let relay_peers: Vec<&str> = parts[1].split(',').collect();
        if relay_peers.is_empty() {
            return;
        }
        let next_hop = relay_peers[0];
        let remaining_chain = &relay_peers[1..];
        let next_protocol_str = if remaining_chain.is_empty() {
            service_uuid.to_string()
        } else {
            format!("{}|relay:{}", service_uuid, remaining_chain.join(","))
        };
        let next_protocol_static: &'static str = Box::leak(next_protocol_str.into_boxed_str());
        let Ok(next_peer) = next_hop.parse::<PeerId>() else {
            return;
        };
        if let Ok(outbound) = stream_control.open_stream(next_peer, StreamProtocol::new(next_protocol_static)).await {
            let mut inbound = stream.compat();
            let mut outbound = outbound.compat();
            let _ = tokio::io::copy_bidirectional(&mut inbound, &mut outbound).await;
        }
    } else {
        let Some(service) = store
            .published_service_by_id(&protocol)
            .await
            .ok()
            .flatten()
        else {
            return;
        };
        let target = format!("127.0.0.1:{}", service.port);
        if let Ok(mut socket) = tokio::net::TcpStream::connect(target).await {
            let mut stream = stream.compat();
            let _ = tokio::io::copy_bidirectional(&mut socket, &mut stream).await;
        }
    }
}

async fn handle_inbound_request(
    store: &Arc<dyn Store>,
    peer: &PeerId,
    request: P2pRequest,
) -> P2pResponse {
    match request {
        P2pRequest::Hello { hello } => {
            if hello.node_id.trim().is_empty() {
                return P2pResponse::Error {
                    message: "node_id 不能为空".into(),
                };
            }
            if hello.role.trim().is_empty() {
                return P2pResponse::Error {
                    message: "role 不能为空".into(),
                };
            }
            if let Err(err) = store
                .upsert_peer(&peer.to_string(), &hello.node_id, &hello.role, "online")
                .await
            {
                return P2pResponse::Error {
                    message: format!("记录 peer 失败: {}", err),
                };
            }
            let local = match store.node_info().await {
                Ok(info) => super::protocol::NodeHello {
                    node_id: info.node_id,
                    role: std::env::var("PORTA_ROLE").unwrap_or_else(|_| "edge".into()),
                },
                Err(err) => {
                    return P2pResponse::Error {
                        message: format!("读取本地节点失败: {}", err),
                    }
                }
            };
            return P2pResponse::HelloAck { hello: local };
        }
        _ => {}
    }

    if let Ok(true) = store.peer_is_banned(&peer.to_string()).await {
        return P2pResponse::Error {
            message: "peer 已被封禁".into(),
        };
    }
    let peer_role = match store.peer_role(&peer.to_string()).await {
        Ok(Some(role)) => role,
        Ok(None) => {
            return P2pResponse::Error {
                message: "peer 未握手".into(),
            };
        }
        Err(err) => {
            return P2pResponse::Error {
                message: format!("读取 peer 失败: {}", err),
            };
        }
    };

    if peer_role.is_empty() {
        return P2pResponse::Error {
            message: "peer 未握手".into(),
        };
    }

    match request {
        P2pRequest::DiscoverServices { .. } => match store.list_service_registry().await {
            Ok(list) => {
                let services = list
                    .into_iter()
                    .map(|item| ServiceAnnouncement {
                        uuid: item.uuid,
                        name: item.name,
                        r#type: item.r#type,
                        port: item.port,
                        description: item.description,
                        provider_peer: item.provider_peer,
                        provider_addr: item.provider_addr,
                    })
                    .collect();
                P2pResponse::ServiceList { services }
            }
            Err(err) => P2pResponse::Error {
                message: format!("读取服务失败: {}", err),
            },
        },
        P2pRequest::SubscribeService {
            service_uuid,
            subscriber_peer,
        } => {
            if peer_role != "edge" {
                return P2pResponse::Error {
                    message: "订阅角色不允许".into(),
                };
            }
            if subscriber_peer != peer.to_string() {
                return P2pResponse::Error {
                    message: "订阅 peer 不匹配".into(),
                };
            }
            if let Err(err) = store.record_subscription(&service_uuid, &subscriber_peer).await {
                return P2pResponse::Error {
                    message: format!("记录订阅失败: {}", err),
                };
            }
            P2pResponse::Ack
        }
        P2pRequest::ConnectService {
            service_uuid,
            subscriber_peer,
        } => {
            if peer_role != "edge" {
                return P2pResponse::Error {
                    message: "连接角色不允许".into(),
                };
            }
            if subscriber_peer != peer.to_string() {
                return P2pResponse::Error {
                    message: "连接 peer 不匹配".into(),
                };
            }
            match store.resolve_service_registry(&service_uuid).await {
                Ok(Some(service)) => P2pResponse::ConnectInfo {
                    provider_peer: service.provider_peer,
                    provider_addr: service.provider_addr,
                    port: service.port,
                },
                Ok(None) => P2pResponse::Error {
                    message: "未找到服务".into(),
                },
                Err(err) => P2pResponse::Error {
                    message: format!("解析服务失败: {}", err),
                },
            }
        }
        P2pRequest::PublishService { service } => {
            if peer_role != "edge" {
                return P2pResponse::Error {
                    message: "发布角色不允许".into(),
                };
            }
            if service.provider_peer != peer.to_string() {
                return P2pResponse::Error {
                    message: "服务提供者 peer 不匹配".into(),
                };
            }
            let registry = ServiceRegistryItem {
                uuid: service.uuid,
                name: service.name,
                r#type: service.r#type,
                port: service.port,
                description: service.description,
                provider_peer: service.provider_peer,
                provider_addr: service.provider_addr,
                online: true,
            };
            if let Err(err) = store.upsert_service_registry(registry).await {
                return P2pResponse::Error {
                    message: format!("服务注册失败: {}", err),
                };
            }
            P2pResponse::Ack
        }
        P2pRequest::UnpublishService { service_uuid } => {
            if peer_role != "edge" {
                return P2pResponse::Error {
                    message: "下架角色不允许".into(),
                };
            }
            match store.remove_service_registry(&service_uuid).await {
                Ok(true) => P2pResponse::Ack,
                Ok(false) => P2pResponse::Error {
                    message: "未找到服务".into(),
                },
                Err(err) => P2pResponse::Error {
                    message: format!("下架失败: {}", err),
                },
            }
        }
        P2pRequest::BuildRelayRoute {
            service_uuid,
            relay_chain,
            initiator_peer: _,
        } => {
            if relay_chain.is_empty() {
                match store.resolve_service_registry(&service_uuid).await {
                    Ok(Some(service)) => P2pResponse::ConnectInfo {
                        provider_peer: service.provider_peer,
                        provider_addr: service.provider_addr,
                        port: service.port,
                    },
                    Ok(None) => P2pResponse::Error {
                        message: "未找到服务".into(),
                    },
                    Err(err) => P2pResponse::Error {
                        message: format!("解析服务失败: {}", err),
                    },
                }
            } else {
                P2pResponse::RelayRouteReady {
                    next_hop: relay_chain.first().cloned(),
                }
            }
        }
        _ => P2pResponse::Error {
            message: "未知请求".into(),
        },
    }
}

fn peer_id_from_addr(addr: &Multiaddr) -> Option<PeerId> {
    addr.iter().find_map(|protocol| {
        if let Protocol::P2p(peer_id) = protocol {
            Some(peer_id)
        } else {
            None
        }
    })
}

async fn load_or_generate_keypair(store: &Arc<dyn Store>) -> Result<identity::Keypair> {
    let info = store.node_info().await?;
    let path = info.key_path;
    if let Ok(bytes) = tokio::fs::read(&path).await {
        if let Ok(keypair) = identity::Keypair::from_protobuf_encoding(&bytes) {
            return Ok(keypair);
        }
    }
    let keypair = identity::Keypair::generate_ed25519();
    let encoded = keypair.to_protobuf_encoding()?;
    tokio::fs::write(&path, encoded).await?;
    Ok(keypair)
}

async fn write_service_uuid(stream: &mut Stream, service_uuid: &str) -> Result<()> {
    let bytes = service_uuid.as_bytes();
    let len = bytes.len() as u16;
    let mut header = [0u8; 2];
    header[0] = (len >> 8) as u8;
    header[1] = (len & 0xff) as u8;
    stream.write_all(&header).await?;
    stream.write_all(bytes).await?;
    stream.flush().await?;
    Ok(())
}

async fn read_service_uuid(stream: &mut Stream) -> Result<String> {
    let mut header = [0u8; 2];
    stream.read_exact(&mut header).await?;
    let len = u16::from_be_bytes(header) as usize;
    if len == 0 || len > 512 {
        return Err(anyhow!("非法服务ID"));
    }
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}
