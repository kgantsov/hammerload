use hdrhistogram::Histogram;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::sync::Mutex;

pub struct Metrics {
    hist: Arc<Mutex<Histogram<u64>>>,
    min_latency: AtomicU64,
    max_latency: AtomicU64,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        let hist = Histogram::<u64>::new(3).unwrap();
        Self {
            hist: Arc::new(Mutex::new(hist)),
            min_latency: AtomicU64::new(u64::MAX),
            max_latency: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
        }
    }

    pub async fn increment_total_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn increment_successful_requests(&self) {
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn increment_failed_requests(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn add_bytes_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    pub async fn add_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }

    pub async fn total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    pub async fn successful_requests(&self) -> u64 {
        self.successful_requests.load(Ordering::Relaxed)
    }

    pub async fn failed_requests(&self) -> u64 {
        self.failed_requests.load(Ordering::Relaxed)
    }

    pub async fn bytes_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }

    pub async fn bytes_received(&self) -> u64 {
        self.bytes_received.load(Ordering::Relaxed)
    }

    pub async fn rps(&self, start_time: std::time::Instant) -> f64 {
        let total = self.total_requests().await;
        let elapsed = std::time::Instant::now().duration_since(start_time);
        total as f64 / elapsed.as_millis() as f64 * 1000.0
    }

    pub async fn throughput_sent(&self, start_time: std::time::Instant) -> f64 {
        let elapsed = start_time.elapsed().as_secs_f64();
        let total_bytes = self.bytes_sent.load(Ordering::Relaxed);
        total_bytes as f64 / elapsed
    }

    pub async fn throughput_received(&self, start_time: std::time::Instant) -> f64 {
        let elapsed = start_time.elapsed().as_secs_f64();
        let total_bytes = self.bytes_received.load(Ordering::Relaxed);
        total_bytes as f64 / elapsed
    }

    pub async fn record_latency(&self, latency: u64) {
        let mut hist = self.hist.lock().await;
        hist.record(latency).unwrap();

        self.min_latency.store(
            latency.min(self.min_latency.load(Ordering::Relaxed)),
            Ordering::Relaxed,
        );
        self.max_latency.store(
            latency.max(self.max_latency.load(Ordering::Relaxed)),
            Ordering::Relaxed,
        );
    }

    pub async fn histogram(&self) -> Histogram<u64> {
        self.hist.lock().await.clone()
    }

    pub async fn min_latency(&self) -> u64 {
        self.min_latency.load(Ordering::Relaxed)
    }

    pub async fn max_latency(&self) -> u64 {
        self.max_latency.load(Ordering::Relaxed)
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
    pub fn human_readable_bytes(&self, bytes: f64) -> String {
        const UNITS: [&str; 7] = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];

        let mut size = bytes as f64;
        let mut unit = 0;

        while size >= 1024.0 && unit < UNITS.len() - 1 {
            size /= 1024.0;
            unit += 1;
        }

        if unit == 0 {
            format!("{} {}", bytes, UNITS[unit])
        } else {
            format!("{:.2} {}", size, UNITS[unit])
        }
    }
}
