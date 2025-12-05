use std::sync::Arc;

use crate::requester::error::RequestError;

use crate::metrics::metrics::Metrics;

use crate::requester::Requester;

pub struct GrpcRequester<'a> {
    metrics: &'a Arc<Metrics>,
}

impl<'a> GrpcRequester<'a> {
    pub fn new(metrics: &'a Arc<Metrics>, _timeout: u64) -> Self {
        Self { metrics }
    }
}

impl<'a> Requester for GrpcRequester<'a> {
    async fn request(&self) -> Result<(), RequestError> {
        let start = std::time::Instant::now();

        // println!("Sending grpc request");

        let req_duration = start.elapsed();

        self.metrics
            .record_latency(req_duration.as_micros().try_into().unwrap_or(0))
            .await;

        Ok(())
    }
}
