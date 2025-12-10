use std::collections::HashMap;

use reqwest::Method;

pub enum RequestParams {
    Http(HttpParams),
    Grpc(GrpcParams),
    Websocket(WebsocketParams),
}

impl RequestParams {
    pub(crate) fn clone(&self) -> RequestParams {
        match self {
            RequestParams::Http(params) => RequestParams::Http(params.clone()),
            RequestParams::Grpc(params) => RequestParams::Grpc(params.clone()),
            RequestParams::Websocket(params) => RequestParams::Websocket(params.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpParams {
    pub url: String,
    pub method: Method,
    pub headers: reqwest::header::HeaderMap,
    pub body: Option<String>,
    pub form: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct GrpcParams {
    pub address: String,
    pub proto: String,
    pub method: String,
    pub data: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WebsocketParams {
    pub url: String,
    pub data: String,
}
