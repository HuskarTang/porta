use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use libp2p::futures::io::{AsyncReadExt, AsyncWriteExt};
use libp2p::futures::StreamExt;
use libp2p::{
    identify, identity,
    multiaddr::Protocol,
    ping,
    request_response::{
        Behaviour as RequestResponse, Config as RequestResponseConfig,
        Event as RequestResponseEvent, Message as RequestResponseMessage, OutboundRequestId,
        ProtocolSupport,
    },
    swarm::{NetworkBehaviour, Stream, StreamProtocol, Swarm, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Transport,
};
use libp2p_stream::{Behaviour as StreamBehaviour, Control as StreamControl};
use tokio::sync::{mpsc, oneshot};
use tokio_util::compat::FuturesAsyncReadCompatExt;

use crate::{models::ServiceRegistryItem, state::Store};

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
        peer_id: PeerId,
        respond_to: oneshot::Sender<Result<()>>,
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
    connected_peers: Arc<tokio::sync::RwLock<HashSet<PeerId>>>,
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

        // Configure RequestResponse with longer timeouts to prevent connection closure
        let rr_config = RequestResponseConfig::default()
            .with_request_timeout(std::time::Duration::from_secs(30));
        let protocols = std::iter::once((PortaProtocol("/porta/req/1"), ProtocolSupport::Full));
        let request_response = RequestResponse::new(protocols, rr_config);

        let stream = StreamBehaviour::new();
        let mut stream_control = stream.new_control();
        // Use shorter ping interval to keep connections alive
        let ping_config = ping::Config::new().with_interval(std::time::Duration::from_secs(10));
        let behaviour = PortaBehaviour {
            request_response,
            ping: ping::Behaviour::new(ping_config),
            identify: identify::Behaviour::new(identify::Config::new(
                "/porta/1.0".into(),
                keypair.public(),
            )),
            stream,
        };

        // Use a longer idle timeout to prevent connections from closing too quickly
        // Ping protocol will keep connections alive, but we need time for initial requests
        let swarm_config = libp2p::swarm::Config::with_tokio_executor()
            .with_idle_connection_timeout(std::time::Duration::from_secs(120));
        let mut swarm = Swarm::new(transport, behaviour, peer_id, swarm_config);
        // Try to use configured port from environment, or auto-assign
        let tcp_port = std::env::var("PORTA_P2P_TCP_PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);
        let listen_addr = if tcp_port > 0 {
            format!("/ip4/0.0.0.0/tcp/{}", tcp_port)
        } else {
            "/ip4/0.0.0.0/tcp/0".to_string()
        };
        tracing::info!("[P2P] Listening on: {}", listen_addr);
        swarm.listen_on(listen_addr.parse()?)?;

        let (sender, mut receiver) = mpsc::channel(32);
        let mut pending: HashMap<OutboundRequestId, oneshot::Sender<Result<P2pResponse>>> =
            HashMap::new();
        let mut pending_dials: HashMap<PeerId, Vec<oneshot::Sender<Result<()>>>> = HashMap::new();
        let connected_peers = Arc::new(tokio::sync::RwLock::new(HashSet::new()));
        let connected_peers_clone = connected_peers.clone();

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
                handle_incoming_stream(
                    peer,
                    stream,
                    &store_for_streams,
                    stream_control_for_relay.clone(),
                )
                .await;
            }
        });
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(cmd) = receiver.recv() => {
                        match cmd {
                            Command::Dial { addr, peer_id, respond_to } => {
                                tracing::info!("[P2P] 开始拨号到 peer: {} (地址: {})", peer_id, addr);
                                if let Err(err) = swarm.dial(addr.clone()) {
                                    let err_msg = format!("无法连接到 {}: {:?}", addr, err);
                                    tracing::error!("[P2P] 拨号失败: {}", err_msg);
                                    let _ = respond_to.send(Err(anyhow!("连接失败: {}", err_msg)));
                                } else {
                                    tracing::info!("[P2P] 拨号请求已发送，等待连接建立: {}", peer_id);
                                    // Store the responder to notify when connection is established
                                    pending_dials.entry(peer_id).or_insert_with(Vec::new).push(respond_to);
                                }
                            }
                            Command::Request { peer, request, respond_to } => {
                                tracing::info!("[P2P] 发送请求: peer={}, request={:?}", peer, request);
                                let request_id = swarm.behaviour_mut().request_response.send_request(&peer, request);
                                tracing::info!("[P2P] 请求已发送: peer={}, request_id={:?}", peer, request_id);
                                pending.insert(request_id, respond_to);
                            }
                        }
                    }
                    event = swarm.select_next_some() => match event {
                        SwarmEvent::Behaviour(PortaBehaviourEvent::RequestResponse(event)) => {
                            handle_request_response_event(event, &mut swarm, &store_clone, &mut pending).await;
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                            tracing::info!("[P2P] 连接已建立: peer={}, endpoint={:?}", peer_id, endpoint);
                            // Track connected peer
                            connected_peers_clone.write().await.insert(peer_id);
                            // Don't notify dial waiters yet - wait for Identify protocol to complete
                        }
                        SwarmEvent::Behaviour(PortaBehaviourEvent::Identify(identify::Event::Received { peer_id, info, .. })) => {
                            tracing::info!("[P2P] Identify 协议完成: peer={}, listen_addrs={:?}", peer_id, info.listen_addrs);
                            // Now that Identify protocol is complete, connection is fully ready
                            // Notify pending dials
                            tracing::debug!("[P2P] 检查 pending_dials，当前 key: peer={}, pending_dials keys: {:?}",
                                peer_id, pending_dials.keys().collect::<Vec<_>>());
                            if let Some(responders) = pending_dials.remove(&peer_id) {
                                tracing::info!("[P2P] 通知等待的 dial: peer={}, 等待者数量={}", peer_id, responders.len());
                                for responder in responders {
                                    let _ = responder.send(Ok(()));
                                }
                            } else {
                                tracing::warn!("[P2P] Identify 完成但未找到 pending_dials 条目: peer={}, 当前 pending_dials keys: {:?}",
                                    peer_id, pending_dials.keys().collect::<Vec<_>>());
                            }
                        }
                        SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                            tracing::warn!("[P2P] 连接已关闭: peer={}, cause={:?}", peer_id, cause);
                            // Remove from connected peers
                            connected_peers_clone.write().await.remove(&peer_id);
                            // Notify pending dials that connection failed
                            if let Some(responders) = pending_dials.remove(&peer_id) {
                                tracing::warn!("[P2P] 连接关闭，通知等待的 dial 失败: peer={}", peer_id);
                                for responder in responders {
                                    let _ = responder.send(Err(anyhow!("连接已关闭: {:?}", cause)));
                                }
                            }
                        }
                        SwarmEvent::NewListenAddr { address, .. } => {
                            tracing::info!("[P2P] 新监听地址: {}", address);
                        }
                        SwarmEvent::ExpiredListenAddr { address, .. } => {
                            tracing::debug!("[P2P] 监听地址过期: {}", address);
                        }
                        SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                            tracing::error!("[P2P] 出站连接错误: peer={:?}, error={:?}", peer_id, error);
                            // Notify pending dials that connection failed
                            if let Some(peer) = peer_id {
                                if let Some(responders) = pending_dials.remove(&peer) {
                                    tracing::warn!("[P2P] 出站连接错误，通知等待的 dial 失败: peer={}", peer);
                                    for responder in responders {
                                        let _ = responder.send(Err(anyhow!("出站连接错误: {:?}", error)));
                                    }
                                }
                            } else {
                                // For errors without peer_id, check if there are any pending dials
                                // This shouldn't happen in normal flow, but log for debugging
                                tracing::warn!("[P2P] 出站连接错误但没有 peer_id，当前 pending_dials keys: {:?}",
                                    pending_dials.keys().collect::<Vec<_>>());
                            }
                        }
                        SwarmEvent::IncomingConnectionError { error, .. } => {
                            tracing::warn!("[P2P] 入站连接错误: error={:?}", error);
                        }
                        _ => {
                            // Log other events at debug level for troubleshooting
                            tracing::debug!("[P2P] SwarmEvent: {:?}", event);
                        }
                    }
                }
            }
        });

        Ok(Self {
            sender,
            peer_id: peer_id.to_string(),
            stream_control: Arc::new(tokio::sync::Mutex::new(stream_control)),
            connected_peers,
        })
    }

    pub async fn dial(&self, addr: Multiaddr) -> Result<PeerId> {
        let peer_id =
            peer_id_from_addr(&addr).ok_or_else(|| anyhow!("multiaddr 缺少 /p2p/peerId"))?;

        let (tx, rx) = oneshot::channel();
        self.sender
            .send(Command::Dial {
                addr,
                peer_id,
                respond_to: tx,
            })
            .await
            .map_err(|_| anyhow!("p2p 通道已关闭"))?;

        // Wait for connection to be established (with timeout)
        // Connection establishment includes: TCP connection, TLS/Noise handshake
        // We wait for Identify protocol completion, which means the connection is fully ready
        // Use a longer timeout to account for slow networks or busy nodes
        let timeout_duration = std::time::Duration::from_secs(30);
        tracing::debug!(
            "[P2P] 等待连接建立: peer={}, 超时={:?}",
            peer_id,
            timeout_duration
        );
        match tokio::time::timeout(timeout_duration, rx).await {
            Ok(Ok(Ok(()))) => {
                tracing::info!("[P2P] 连接建立成功: peer={}", peer_id);
                Ok(peer_id)
            }
            Ok(Ok(Err(e))) => {
                tracing::error!("[P2P] 连接建立失败: peer={}, error={}", peer_id, e);
                Err(e)
            }
            Ok(Err(e)) => {
                tracing::error!(
                    "[P2P] oneshot channel 错误: peer={}, error={:?}",
                    peer_id,
                    e
                );
                Err(anyhow!("p2p 连接建立失败：channel 错误"))
            }
            Err(_) => {
                // Timeout occurred - this means Identify event was never received
                tracing::error!(
                    "[P2P] 连接建立超时: peer={}, 超时时间={:?}",
                    peer_id,
                    timeout_duration
                );
                tracing::error!(
                    "[P2P] 可能原因: 1) Identify 事件未触发 2) peer_id 不匹配 3) 目标节点未响应"
                );
                Err(anyhow!(
                    "p2p 连接建立超时（30秒）。可能原因：Identify 协议未完成或 peer_id 不匹配"
                ))
            }
        }
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

    /// Check if a peer is currently connected
    pub async fn is_connected(&self, peer_id: &PeerId) -> bool {
        self.connected_peers.read().await.contains(peer_id)
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
        RequestResponseEvent::OutboundFailure {
            request_id, error, ..
        } => {
            if let Some(ch) = pending.remove(&request_id) {
                let error_msg: String = match &error {
                    libp2p::request_response::OutboundFailure::DialFailure => {
                        "无法连接到目标节点，请检查网络连接和节点地址".to_string()
                    }
                    libp2p::request_response::OutboundFailure::Timeout => {
                        "请求超时，目标节点可能无响应".to_string()
                    }
                    libp2p::request_response::OutboundFailure::ConnectionClosed => {
                        "连接已关闭".to_string()
                    }
                    libp2p::request_response::OutboundFailure::UnsupportedProtocols => {
                        "目标节点不支持请求的协议".to_string()
                    }
                    libp2p::request_response::OutboundFailure::Io(err) => {
                        tracing::error!("[P2P] IO 错误: {:?}", err);
                        format!("IO 错误: {}", err)
                    }
                };
                tracing::error!(
                    "[P2P] 出站请求失败: request_id={:?}, error={:?}, msg={}",
                    request_id,
                    error,
                    error_msg
                );
                let _ = ch.send(Err(anyhow!("请求失败: {}", error_msg)));
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
    if store
        .peer_is_banned(&peer.to_string())
        .await
        .unwrap_or(false)
    {
        tracing::warn!("拒绝已封禁 peer {} 的 stream", peer);
        return;
    }
    let role = store.peer_role(&peer.to_string()).await.ok().flatten();
    if role.as_deref() != Some("edge") {
        tracing::warn!("拒绝非 edge 角色 peer {} 的 stream", peer);
        return;
    }
    let protocol = match read_service_uuid(&mut stream).await {
        Ok(uuid) => uuid,
        Err(err) => {
            tracing::error!("读取协议失败: {:?}", err);
            return;
        }
    };
    tracing::debug!("收到 peer {} 的 stream 请求: {}", peer, protocol);

    if protocol.contains("|relay:") {
        let parts: Vec<&str> = protocol.split("|relay:").collect();
        if parts.len() != 2 {
            tracing::warn!("无效的中继协议格式: {}", protocol);
            return;
        }
        let service_uuid = parts[0];
        let relay_peers: Vec<&str> = parts[1].split(',').collect();
        if relay_peers.is_empty() {
            return;
        }
        let next_hop = relay_peers[0];
        let remaining_chain = &relay_peers[1..];
        tracing::info!(
            "中继转发: {} -> {}, 剩余 {} 跳",
            peer,
            next_hop,
            remaining_chain.len()
        );
        let next_protocol_str = if remaining_chain.is_empty() {
            service_uuid.to_string()
        } else {
            format!("{}|relay:{}", service_uuid, remaining_chain.join(","))
        };
        let next_protocol_static: &'static str = Box::leak(next_protocol_str.into_boxed_str());
        let Ok(next_peer) = next_hop.parse::<PeerId>() else {
            tracing::error!("无效的下一跳 peerId: {}", next_hop);
            return;
        };
        match stream_control
            .open_stream(next_peer, StreamProtocol::new(next_protocol_static))
            .await
        {
            Ok(outbound) => {
                let mut inbound = stream.compat();
                let mut outbound = outbound.compat();
                let _ = tokio::io::copy_bidirectional(&mut inbound, &mut outbound).await;
                tracing::debug!("中继转发完成");
            }
            Err(err) => {
                tracing::error!("打开下一跳 stream 失败: {:?}", err);
            }
        }
    } else {
        let Some(service) = store
            .published_service_by_id(&protocol)
            .await
            .ok()
            .flatten()
        else {
            tracing::warn!("未找到服务: {}", protocol);
            return;
        };
        let target = format!("127.0.0.1:{}", service.port);
        tracing::info!("转发 stream 到本地服务: {} -> {}", protocol, target);
        match tokio::net::TcpStream::connect(&target).await {
            Ok(mut socket) => {
                let mut stream = stream.compat();
                match tokio::io::copy_bidirectional(&mut socket, &mut stream).await {
                    Ok((sent, received)) => {
                        tracing::debug!(
                            "服务 {} 转发完成: 发送 {} 字节, 接收 {} 字节",
                            protocol,
                            sent,
                            received
                        );
                    }
                    Err(err) => {
                        tracing::error!("服务 {} 转发失败: {}", protocol, err);
                    }
                }
            }
            Err(err) => {
                tracing::error!("连接本地服务 {} 失败: {}", target, err);
            }
        }
    }
}

async fn handle_inbound_request(
    store: &Arc<dyn Store>,
    peer: &PeerId,
    request: P2pRequest,
) -> P2pResponse {
    tracing::debug!(
        "收到 peer {} 的请求: {:?}",
        peer,
        std::mem::discriminant(&request)
    );
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
            if let Err(err) = store
                .record_subscription(&service_uuid, &subscriber_peer)
                .await
            {
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
    // Priority: 1. Environment variable PORTA_KEY_PATH, 2. Database node_info.key_path, 3. Generate based on DB path
    let key_path = if let Ok(env_path) = std::env::var("PORTA_KEY_PATH") {
        tracing::info!("[P2P] Using key path from environment: {}", env_path);
        env_path
    } else {
        let info = store.node_info().await?;
        let db_path = info.key_path.clone();
        if db_path.is_empty() || db_path == "porta.node.key" {
            // Generate unique key path based on database path
            let db_path_env = std::env::var("PORTA_DB").unwrap_or_else(|_| "porta.db".to_string());
            let db_file = std::path::Path::new(&db_path_env);
            let key_file = db_file
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| format!("{}.key", s))
                .unwrap_or_else(|| "porta.node.key".to_string());
            tracing::info!(
                "[P2P] Generated key path from database: {} -> {}",
                db_path_env,
                key_file
            );
            key_file
        } else {
            tracing::info!("[P2P] Using key path from database: {}", db_path);
            db_path
        }
    };

    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(&key_path).parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                tracing::warn!(
                    "[P2P] Failed to create key directory {}: {}",
                    parent.display(),
                    e
                );
            }
        }
    }

    // Try to load existing key
    if let Ok(bytes) = tokio::fs::read(&key_path).await {
        if let Ok(keypair) = identity::Keypair::from_protobuf_encoding(&bytes) {
            let peer_id = PeerId::from(keypair.public());
            tracing::info!(
                "[P2P] Loaded existing key from {}: peer_id={}",
                key_path,
                peer_id
            );
            return Ok(keypair);
        } else {
            tracing::warn!(
                "[P2P] Key file {} exists but is invalid, generating new key",
                key_path
            );
        }
    }

    // Generate new key
    let keypair = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(keypair.public());
    let encoded = keypair.to_protobuf_encoding()?;
    tokio::fs::write(&key_path, encoded).await?;
    tracing::info!(
        "[P2P] Generated new key at {}: peer_id={}",
        key_path,
        peer_id
    );
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
