use async_trait::async_trait;
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::request_response::ProtocolName;
use serde::{Deserialize, Serialize};

use crate::models::ServiceDescriptor;

#[derive(Clone)]
pub struct PortaProtocol;

impl ProtocolName for PortaProtocol {
    fn protocol_name(&self) -> &[u8] {
        b"/porta/req/1"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2pRequest {
    DiscoverServices { community_id: String },
    SubscribeService { community_id: String, service_uuid: String },
    PublishService { service: ServiceDescriptor },
    UnpublishService { service_uuid: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2pResponse {
    ServiceList { services: Vec<ServiceDescriptor> },
    Ack,
    Error { message: String },
}

#[derive(Clone, Default)]
pub struct JsonCodec;

#[async_trait]
impl libp2p::request_response::RequestResponseCodec for JsonCodec {
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
        let req = P2pRequest::DiscoverServices {
            community_id: "dev".into(),
        };
        let data = serde_json::to_vec(&req).unwrap();
        let decoded: P2pRequest = serde_json::from_slice(&data).unwrap();
        match decoded {
            P2pRequest::DiscoverServices { community_id } => assert_eq!(community_id, "dev"),
            _ => panic!("unexpected request"),
        }
    }
}
