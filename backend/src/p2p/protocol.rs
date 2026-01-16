use async_trait::async_trait;
use libp2p::futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHello {
    pub node_id: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAnnouncement {
    pub uuid: String,
    pub name: String,
    pub r#type: String,
    pub port: u16,
    pub description: String,
    pub provider_peer: String,
    pub provider_addr: String,
}

#[derive(Clone)]
pub struct PortaProtocol(pub &'static str);

impl AsRef<str> for PortaProtocol {
    fn as_ref(&self) -> &str {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2pRequest {
    Hello { hello: NodeHello },
    DiscoverServices { community_id: String },
    SubscribeService { service_uuid: String, subscriber_peer: String },
    ConnectService { service_uuid: String, subscriber_peer: String },
    PublishService { service: ServiceAnnouncement },
    UnpublishService { service_uuid: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2pResponse {
    HelloAck { hello: NodeHello },
    ServiceList { services: Vec<ServiceAnnouncement> },
    ConnectInfo {
        provider_peer: String,
        provider_addr: String,
        port: u16,
    },
    Ack,
    Error { message: String },
}

#[derive(Clone, Default)]
pub struct JsonCodec;

#[async_trait]
impl libp2p::request_response::Codec for JsonCodec {
    type Protocol = PortaProtocol;
    type Request = P2pRequest;
    type Response = P2pResponse;

    async fn read_request<T>(
        &mut self,
        _: &PortaProtocol,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        serde_json::from_slice(&buf)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
    }

    async fn read_response<T>(
        &mut self,
        _: &PortaProtocol,
        io: &mut T,
    ) -> std::io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        serde_json::from_slice(&buf)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
    }

    async fn write_request<T>(
        &mut self,
        _: &PortaProtocol,
        io: &mut T,
        request: Self::Request,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&request)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        io.write_all(&data).await?;
        io.close().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &PortaProtocol,
        io: &mut T,
        response: Self::Response,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&response)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        io.write_all(&data).await?;
        io.close().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_roundtrip_request() {
        let req = P2pRequest::Hello {
            hello: NodeHello {
                node_id: "node-1".into(),
                role: "edge".into(),
            },
        };
        let data = serde_json::to_vec(&req).unwrap();
        let decoded: P2pRequest = serde_json::from_slice(&data).unwrap();
        match decoded {
            P2pRequest::Hello { hello } => {
                assert_eq!(hello.node_id, "node-1");
                assert_eq!(hello.role, "edge");
            }
            _ => panic!("unexpected request"),
        }
    }
}
