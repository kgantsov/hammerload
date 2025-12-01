use hdrhistogram::Histogram;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Metrics {
    hist: Arc<Mutex<Histogram<u64>>>,
    total_requests: Arc<Mutex<u64>>,
    successful_requests: Arc<Mutex<u64>>,
    failed_requests: Arc<Mutex<u64>>,
}

impl Metrics {
    pub fn new() -> Self {
        let hist = Histogram::<u64>::new(3).unwrap();
        Self {
            hist: Arc::new(Mutex::new(hist)),
            total_requests: Arc::new(Mutex::new(0)),
            successful_requests: Arc::new(Mutex::new(0)),
            failed_requests: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn increment_total_requests(&self) {
        let mut tr = self.total_requests.lock().await;
        *tr += 1;
    }

    pub async fn increment_successful_requests(&self) {
        let mut sr = self.successful_requests.lock().await;
        *sr += 1;
    }

    pub async fn increment_failed_requests(&self) {
        let mut fr = self.failed_requests.lock().await;
        *fr += 1;
    }

    pub async fn total_requests(&self) -> u64 {
        *self.total_requests.lock().await
    }

    pub async fn successful_requests(&self) -> u64 {
        *self.successful_requests.lock().await
    }

    pub async fn failed_requests(&self) -> u64 {
        *self.failed_requests.lock().await
    }

    pub async fn rps(&self, start_time: std::time::Instant) -> f64 {
        let total = self.total_requests().await;
        let elapsed = std::time::Instant::now().duration_since(start_time);
        total as f64 / elapsed.as_millis() as f64 * 1000.0
    }

    pub async fn record_latency(&self, latency: u64) {
        let mut hist = self.hist.lock().await;
        hist.record(latency).unwrap();
    }

    pub async fn histogram(&self) -> Histogram<u64> {
        self.hist.lock().await.clone()
    }

    pub fn format_micros(&self, us: u64) -> String {
        const MICROS_PER_MS: u64 = 1_000;
        const MICROS_PER_SEC: u64 = 1_000_000;
        const MICROS_PER_MIN: u64 = 60 * MICROS_PER_SEC;
        const MICROS_PER_HOUR: u64 = 60 * MICROS_PER_MIN;

        if us < MICROS_PER_MS {
            format!("{}µs", us)
        } else if us < MICROS_PER_SEC {
            format!("{}ms", us / MICROS_PER_MS)
        } else if us < MICROS_PER_MIN {
            format!("{}s", us / MICROS_PER_SEC)
        } else if us < MICROS_PER_HOUR {
            format!("{}m", us / MICROS_PER_MIN)
        } else {
            format!("{}h", us / MICROS_PER_HOUR)
        }
    }
}
