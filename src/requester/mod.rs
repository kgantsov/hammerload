pub mod error;
pub mod grpc_requester;
pub mod http_requester;
pub mod params;
pub mod websocket_requester;

use crate::requester::error::RequestError;

#[allow(async_fn_in_trait)]
pub trait Requester {
    async fn initialize(&self) -> Result<(), RequestError>;
    async fn request(&self) -> Result<(), RequestError>;
}
