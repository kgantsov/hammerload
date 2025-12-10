use std::sync::Arc;
use std::sync::OnceLock;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::Message;
use tungstenite::Utf8Bytes;

use crate::requester::error::RequestError;

use crate::metrics::metrics::Metrics;

use crate::requester::Requester;

pub struct WebsocketRequester<'a> {
    metrics: &'a Arc<Metrics>,
    url: String,
    data: String,
    request_size: u64,
    writer: OnceLock<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
}

impl<'a> WebsocketRequester<'a> {
    pub fn new(metrics: &'a Arc<Metrics>, url: String, data: String) -> Self {
        let request_size = data.clone().len() as u64;

        Self {
            metrics,
            url,
            data,
            request_size,
            writer: OnceLock::new(),
        }
    }
}

impl<'a> Requester for WebsocketRequester<'a> {
    async fn initialize(&self) -> Result<(), RequestError> {
        if self.writer.get().is_some() {
            return Ok(());
        }

        let (ws_stream, _) = connect_async(self.url.clone()).await.map_err(|e| {
            RequestError::ConfigError(format!("Failed to connect to {}: {}", self.url.clone(), e))
        })?;

        let (write, mut read) = ws_stream.split();

        self.writer
            .set(Mutex::new(write))
            .map_err(|_| RequestError::InternalError("Writer already set".to_string()))?;

        let metrics = self.metrics.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if let Ok(msg) = msg {
                    if let Message::Text(text) = msg {
                        let response_size = text.len() as u64;
                        metrics.add_bytes_received(response_size).await;
                    }
                }
            }
        });

        Ok(())
    }

    async fn request(&self) -> Result<(), RequestError> {
        let writer = self.writer.get().ok_or(RequestError::InternalError(
            "Requester not initialised: Missing writer".to_string(),
        ))?;

        let mut writer = writer.lock().await;

        let start = std::time::Instant::now();

        self.metrics.add_bytes_sent(self.request_size).await;

        writer
            .send(Message::Text(Utf8Bytes::from(self.data.clone())))
            .await
            .map_err(|e| RequestError::InternalError(e.to_string()))?;

        let req_duration = start.elapsed();

        self.metrics
            .record_latency(req_duration.as_micros().try_into().unwrap_or(0))
            .await;

        Ok(())
    }
}
