use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::requester::error::RequestError;
use reqwest::{header::HeaderMap, Client, Method};

use crate::metrics::metrics::Metrics;

use crate::requester::Requester;

pub struct HttpRequester<'a> {
    metrics: &'a Arc<Metrics>,
    method: Method,
    url: String,
    body: Option<String>,
    form_params: HashMap<String, String>,
    client: Client,
    request_size: u64,
}

impl<'a> HttpRequester<'a> {
    pub fn new(
        metrics: &'a Arc<Metrics>,
        method: Method,
        url: String,
        body: Option<String>,
        form_params: HashMap<String, String>,
        headers: HeaderMap,
        timeout: u64,
    ) -> Self {
        let client = Client::builder()
            .default_headers(headers.clone())
            .timeout(Duration::from_secs(timeout))
            .build()
            .unwrap();

        let mut request_size = 0;
        if let Some(b) = body.clone() {
            request_size += b.len() as u64;
        }
        for (key, value) in form_params.clone() {
            request_size += key.len() as u64 + value.len() as u64;
        }
        for (key, value) in headers.iter() {
            request_size += key.as_str().len() as u64 + value.as_bytes().len() as u64;
        }

        Self {
            metrics,
            method,
            url,
            body,
            form_params,
            client,
            request_size,
        }
    }
}

impl<'a> Requester for HttpRequester<'a> {
    async fn request(&self) -> Result<(), RequestError> {
        let start = std::time::Instant::now();

        let req_builder = self.client.request(self.method.clone(), self.url.clone());
        let req_builder = if self.form_params.len() > 0 {
            req_builder.form(&self.form_params)
        } else {
            req_builder
        };
        let req_builder = match self.body.clone() {
            Some(b) => req_builder.body(b.clone()),
            None => req_builder,
        };

        let resp = req_builder.send().await.map_err(|e| {
            if e.is_timeout() {
                RequestError::Timeout
            } else {
                RequestError::Network
            }
        })?;
        let _status = resp.status();
        let body = resp.bytes().await.map_err(|e| {
            if e.is_timeout() {
                RequestError::Timeout
            } else {
                RequestError::Network
            }
        })?;

        let response_size = body.len() as u64;
        self.metrics.add_bytes_received(response_size).await;
        self.metrics.add_bytes_sent(self.request_size).await;

        let req_duration = start.elapsed();

        self.metrics
            .record_latency(req_duration.as_micros().try_into().unwrap_or(0))
            .await;

        Ok(())
    }
}
