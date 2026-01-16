use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use futures::StreamExt;
use libp2p::{
    identify, identity,
    multiaddr::Protocol,
    ping,
    request_response::{
        Behaviour as RequestResponse, Config as RequestResponseConfig, Event as RequestResponseEvent,
        Message as RequestResponseMessage, ProtocolSupport, RequestId,
    },
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, SwarmBuilder, Transport,
};
use tokio::sync::{mpsc, oneshot};

use crate::{
    models::ServiceDescriptor,
    state::Store,
};

use super::protocol::{JsonCodec, P2pRequest, P2pResponse, PortaProtocol};

#[derive(NetworkBehaviour)]
struct PortaBehaviour {
    request_response: RequestResponse<JsonCodec>,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
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
        let protocols = std::iter::once((PortaProtocol, ProtocolSupport::Full));
        let request_response = RequestResponse::new(JsonCodec::default(), protocols, rr_config);

        let behaviour = PortaBehaviour {
            request_response,
            ping: ping::Behaviour::new(ping::Config::new().with_keep_alive(true)),
            identify: identify::Behaviour::new(
                identify::Config::new("/porta/1.0".into(), keypair.public()),
            ),
        };

        let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build();
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        let (sender, mut receiver) = mpsc::channel(32);
        let mut pending: HashMap<RequestId, oneshot::Sender<Result<P2pResponse>>> =
            HashMap::new();

        let store_clone = store.clone();
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
}

async fn handle_request_response_event(
    event: RequestResponseEvent<P2pRequest, P2pResponse>,
    swarm: &mut Swarm<PortaBehaviour>,
    store: &Arc<dyn Store>,
    pending: &mut HashMap<RequestId, oneshot::Sender<Result<P2pResponse>>>,
) {
    match event {
        RequestResponseEvent::Message { message, .. } => match message {
            RequestResponseMessage::Request {
                request, channel, ..
            } => {
                let response = handle_inbound_request(store, request).await;
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

async fn handle_inbound_request(store: &Arc<dyn Store>, request: P2pRequest) -> P2pResponse {
    match request {
        P2pRequest::DiscoverServices { .. } => match store.published_services().await {
            Ok(list) => {
                let services = list
                    .into_iter()
                    .map(|item| ServiceDescriptor {
                        uuid: item.id,
                        name: item.name,
                        r#type: item.r#type,
                        remote_port: item.port,
                        provider: "community-node".into(),
                        description: item.summary,
                    })
                    .collect();
                P2pResponse::ServiceList { services }
            }
            Err(err) => P2pResponse::Error {
                message: format!("读取服务失败: {}", err),
            },
        },
        P2pRequest::SubscribeService { .. } => P2pResponse::Ack,
        P2pRequest::PublishService { .. } => P2pResponse::Ack,
        P2pRequest::UnpublishService { .. } => P2pResponse::Ack,
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
    let encoded = keypair.to_protobuf_encoding();
    tokio::fs::write(&path, encoded).await?;
    Ok(keypair)
}
