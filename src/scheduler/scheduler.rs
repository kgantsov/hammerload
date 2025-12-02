use std::{collections::HashMap, sync::Arc, time::Duration};

use reqwest::{header::HeaderMap, Client, Method};

use crate::metrics::metrics::Metrics;

pub struct Scheduler<'a> {
    metrics: &'a Arc<Metrics>,
    method: Method,
    url: String,
    body: Option<String>,
    form_params: HashMap<String, String>,
    headers: HeaderMap,
    concurrency: u64,
    duration: u64,
    timeout: u64,
}

impl<'a> Scheduler<'a> {
    pub fn new(
        metrics: &'a Arc<Metrics>,
        method: Method,
        url: String,
        body: Option<String>,
        form_params: HashMap<String, String>,
        headers: HeaderMap,
        concurrency: u64,
        duration: u64,
        timeout: u64,
    ) -> Self {
        Scheduler {
            metrics,
            method,
            url,
            body,
            form_params,
            headers,
            concurrency,
            duration,
            timeout,
        }
    }

    pub async fn run(&self) {
        let start_bench = std::time::Instant::now();
        let mut tasks = Vec::new();

        let url = self.url.clone();
        let headers = self.headers.clone();

        for _ in 0..self.concurrency {
            let method = self.method.clone();
            let url = url.clone();
            let body = self.body.clone();
            let form_params = self.form_params.clone();
            let headers = headers.clone();
            let duration = self.duration;
            let timeout = self.timeout;
            let metrics = Arc::clone(self.metrics);

            tasks.push(tokio::spawn(async move {
                Scheduler::run_client(
                    &metrics,
                    start_bench,
                    method,
                    url,
                    body,
                    form_params,
                    headers,
                    duration,
                    timeout,
                )
                .await;
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
        method: Method,
        url: String,
        body: Option<String>,
        form_params: HashMap<String, String>,
        headers: HeaderMap,
        duration: u64,
        timeout: u64,
    ) {
        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(timeout))
            .build()
            .unwrap();

        loop {
            let result =
                Self::make_request(metrics, &client, &method, &url, &body, &form_params).await;
            Self::handle_request_result(metrics, result).await;

            if std::time::Instant::now() >= start_bench + std::time::Duration::from_secs(duration) {
                break;
            }
        }
    }

    async fn make_request(
        metrics: &Arc<Metrics>,
        client: &Client,
        method: &Method,
        url: &str,
        body: &Option<String>,
        form_params: &HashMap<String, String>,
    ) -> Result<String, reqwest::Error> {
        let start = std::time::Instant::now();

        let req_builder = client.request(method.clone(), url);
        let req_builder = if form_params.len() > 0 {
            req_builder.form(form_params)
        } else {
            req_builder
        };
        let req_builder = match body {
            Some(b) => req_builder.body(b.clone()),
            None => req_builder,
        };

        let resp = req_builder.send().await?;
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
