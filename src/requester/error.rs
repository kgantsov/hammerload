#[derive(Debug, Clone)]
pub enum RequestError {
    Network,
    Timeout,
}
