#[derive(Debug, Clone)]
pub enum RequestError {
    Network,
    Timeout,
    ConfigError(String),
    InvalidRequest(String),
    RequestFailed(String),
    ConnectionError(String),
    InternalError(String),
    GrpcError(String),
}
