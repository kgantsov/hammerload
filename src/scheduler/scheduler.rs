use std::{collections::HashMap, sync::Arc, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{header::HeaderMap, Method};

use crate::{
    metrics::metrics::Metrics,
    requester::{error::RequestError, http_requester::HttpRequester, Requester},
};

pub struct Scheduler<'a> {
    metrics: &'a Arc<Metrics>,
    method: Method,
    url: String,
    body: Option<String>,
    form_params: HashMap<String, String>,
    headers: HeaderMap,
    concurrency: u64,
    duration: u64,
    rate: Option<u64>,
    timeout: u64,
    show_progress: bool,
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
        rate: Option<u64>,
        timeout: u64,
        show_progress: bool,
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
            rate,
            timeout,
            show_progress,
        }
    }

    pub async fn run(&self) {
        let start_bench = std::time::Instant::now();
        let mut tasks = Vec::new();

        let url = self.url.clone();
        let headers = self.headers.clone();
        let duration = self.duration;

        if self.show_progress {
            let bar = ProgressBar::new(duration);
            let bar = bar.with_message("Hammering");
            bar.set_style(
                ProgressStyle::with_template("{msg} {bar:40.cyan/blue} [{elapsed_precise}]")
                    .unwrap()
                    .progress_chars("##-"),
            );

            tasks.push(tokio::spawn(async move {
                let mut seconds_left = duration;
                let mut interval = tokio::time::interval(Duration::from_secs(1));

                while seconds_left > 0 {
                    interval.tick().await;
                    bar.inc(1);
                    seconds_left -= 1;
                }
                bar.finish();
            }));
        }

        for _ in 0..self.concurrency {
            let method = self.method.clone();
            let url = url.clone();
            let body = self.body.clone();
            let form_params = self.form_params.clone();
            let headers = headers.clone();
            let concurrency = self.concurrency;
            let duration = self.duration;
            let rate = self.rate;
            let timeout = self.timeout;
            let metrics = Arc::clone(self.metrics);

            tasks.push(tokio::spawn(async move {
                let requester =
                    HttpRequester::new(&metrics, method, url, body, form_params, headers, timeout);

                Scheduler::run_client(
                    &metrics,
                    start_bench,
                    requester,
                    concurrency,
                    duration,
                    rate,
                )
                .await;
            }));
        }

        for task in tasks {
            task.await.unwrap();
        }

        self.print_report(start_bench).await;
    }

    async fn run_client<R>(
        metrics: &Arc<Metrics>,
        start_bench: std::time::Instant,
        requester: R,
        concurrency: u64,
        duration: u64,
        rate: Option<u64>,
    ) where
        R: Requester + Send,
    {
        let interval = rate.map(|rps| {
            let per_worker = (rps as f64) / (concurrency as f64);
            Duration::from_secs_f64(1.0 / per_worker)
        });

        loop {
            let loop_start = std::time::Instant::now();

            let result = requester.request().await;

            Self::handle_request_result(metrics, result).await;

            if std::time::Instant::now() >= start_bench + std::time::Duration::from_secs(duration) {
                break;
            }

            // Enforce rate limiting
            if let Some(interval) = interval {
                let elapsed = loop_start.elapsed();
                if elapsed < interval {
                    tokio::time::sleep(interval - elapsed).await;
                }
            }
        }
    }

    async fn handle_request_result(metrics: &Arc<Metrics>, result: Result<(), RequestError>) {
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

    async fn print_report(&self, start_bench: std::time::Instant) {
        let total_requests = self.metrics.total_requests().await;
        let successful_requests = self.metrics.successful_requests().await;
        let failed_requests = self.metrics.failed_requests().await;

        let success_rate = successful_requests as f64 / total_requests as f64 * 100.0;
        let fail_rate = failed_requests as f64 / total_requests as f64 * 100.0;

        println!("");
        println!(
            "Requests:......................{:<10} {:>10.2}/s",
            total_requests,
            self.metrics.rps(start_bench).await
        );
        println!(
            "Requests succeded:.............{:<10}  {:>10.2}%",
            successful_requests, success_rate
        );
        println!(
            "Requests failed:...............{:<10}  {:>10.2}%",
            failed_requests, fail_rate
        );
        println!(
            "Data sent:.....................{:<10} {:>10}/s",
            self.metrics
                .human_readable_bytes(self.metrics.bytes_sent().await as f64),
            self.metrics
                .human_readable_bytes(self.metrics.throughput_sent(start_bench).await)
        );
        println!(
            "Data received:.................{:<10} {:>10}/s",
            self.metrics
                .human_readable_bytes(self.metrics.bytes_received().await as f64),
            self.metrics
                .human_readable_bytes(self.metrics.throughput_received(start_bench).await)
        );
        println!("Latencies:");
        println!(
            "   Min:........................{}",
            self.metrics.format_micros(self.metrics.min_latency().await)
        );
        println!(
            "   P(50):......................{}",
            self.metrics
                .format_micros(self.metrics.histogram().await.value_at_quantile(0.50))
        );
        println!(
            "   P(90):......................{}",
            self.metrics
                .format_micros(self.metrics.histogram().await.value_at_quantile(0.90))
        );
        println!(
            "   P(95):......................{}",
            self.metrics
                .format_micros(self.metrics.histogram().await.value_at_quantile(0.95))
        );
        println!(
            "   P(99):......................{}",
            self.metrics
                .format_micros(self.metrics.histogram().await.value_at_quantile(0.99))
        );
        println!(
            "   P(99.9):....................{}",
            self.metrics
                .format_micros(self.metrics.histogram().await.value_at_quantile(0.999))
        );
        println!(
            "   P(99.99):...................{}",
            self.metrics
                .format_micros(self.metrics.histogram().await.value_at_quantile(0.9999))
        );
        println!(
            "   Max:........................{}",
            self.metrics.format_micros(self.metrics.max_latency().await)
        );
    }
}
