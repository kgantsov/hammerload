use std::collections::HashMap;

use reqwest::Method;

pub enum RequestParams {
    Http(HttpParams),
    Grpc(GrpcParams),
}

impl RequestParams {
    pub(crate) fn clone(&self) -> RequestParams {
        match self {
            RequestParams::Http(params) => RequestParams::Http(params.clone()),
            RequestParams::Grpc(params) => RequestParams::Grpc(params.clone()),
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
    pub proto: String,
    pub url: String,
    pub method: String,
    pub data: Option<String>,
}
