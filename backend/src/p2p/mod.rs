pub mod node;
pub mod protocol;

pub use node::NodeHandle;
pub use protocol::{P2pRequest, P2pResponse};
pub const STREAM_PROTOCOL: &str = "/porta/stream/1";
