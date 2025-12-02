use std::sync::Arc;

use reqwest::{header::HeaderMap, Client};

use crate::metrics::metrics::Metrics;

pub struct Scheduler<'a> {
    metrics: &'a Arc<Metrics>,
    url: String,
    concurrency: u64,
    duration: u64,
    headers: HeaderMap,
}

impl<'a> Scheduler<'a> {
    pub fn new(
        metrics: &'a Arc<Metrics>,
        url: String,
        concurrency: u64,
        duration: u64,
        headers: HeaderMap,
    ) -> Self {
        Scheduler {
            metrics,
            url,
            concurrency,
            duration,
            headers,
        }
    }

    pub async fn run(&self) {
        let start_bench = std::time::Instant::now();
        let mut tasks = Vec::new();

        let url = self.url.clone();
        let headers = self.headers.clone();

        for _ in 0..self.concurrency {
            let url = url.clone();
            let headers = headers.clone();
            let duration = self.duration;
            let metrics = Arc::clone(self.metrics);

            tasks.push(tokio::spawn(async move {
                Scheduler::run_client(&metrics, start_bench, url, duration, headers).await;
            }));
        }

        for task in tasks {
            task.await.unwrap();
        }

        println!(
            "Requests per second: {}",
            self.metrics.rps(start_bench).await
        );
        println!(
            "Successful requests: {}",
            self.metrics.successful_requests().await
        );
        println!("Failed requests: {}", self.metrics.failed_requests().await);

        println!(
            "50th percentile: {}",
            self.metrics
                .format_micros(self.metrics.histogram().await.value_at_quantile(0.50))
        );
        println!(
            "95th percentile: {}",
            self.metrics
                .format_micros(self.metrics.histogram().await.value_at_quantile(0.95))
        );
        println!(
            "99th percentile: {}",
            self.metrics
                .format_micros(self.metrics.histogram().await.value_at_quantile(0.99))
        );
    }

    async fn run_client(
        metrics: &'a Arc<Metrics>,
        start_bench: std::time::Instant,
        url: String,
        duration: u64,
        headers: HeaderMap,
    ) {
        let client = Client::builder().default_headers(headers).build().unwrap();

        loop {
            let result = Self::make_request(metrics, &client, &url).await;
            Self::handle_request_result(metrics, result).await;

            if std::time::Instant::now() >= start_bench + std::time::Duration::from_secs(duration) {
                break;
            }
        }
    }

    async fn make_request(
        metrics: &Arc<Metrics>,
        client: &Client,
        url: &str,
    ) -> Result<String, reqwest::Error> {
        let start = std::time::Instant::now();

        let resp = client.get(url).send().await?;
        let _status = resp.status();
        let body = resp.text().await?;

        let req_duration = start.elapsed();

        metrics
            .record_latency(req_duration.as_micros().try_into().unwrap_or(0))
            .await;

        Ok(body)
    }

    async fn handle_request_result(metrics: &Arc<Metrics>, result: Result<String, reqwest::Error>) {
        match result {
            Ok(_body) => {
                metrics.increment_successful_requests().await;
                metrics.increment_total_requests().await;
            }
            Err(_) => {
                metrics.increment_failed_requests().await;
            }
        };
    }
}
