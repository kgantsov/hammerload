use clap::{Parser, ValueEnum};
use hdrhistogram::Histogram;
use reqwest::Client;

#[derive(ValueEnum, Debug, Clone)]
enum Protocol {
    Http,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(value_name = "PROTOCOL")]
    protocol: Protocol,

    #[arg(short, long, value_name = "URL")]
    url: String,

    #[arg(short, long, value_name = "DURATION", default_value_t = 10)]
    duration: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();

    let client = Client::new();

    let mut hist = Histogram::<u64>::new(3).unwrap(); // 3 digits of precision

    let start_bench = std::time::Instant::now();

    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut failed_requests = 0;

    loop {
        let start = std::time::Instant::now();

        let resp = client.get(args.url.clone()).send().await?;
        let status_code = resp.status();

        resp.text().await?;

        if status_code.is_success() {
            successful_requests += 1;
        } else {
            failed_requests += 1;
        }
        total_requests += 1;

        let duration = start.elapsed();
        hist.record(duration.as_micros().try_into().unwrap_or(0))
            .unwrap();

        if std::time::Instant::now() >= start_bench + std::time::Duration::from_secs(args.duration)
        {
            break;
        }
    }

    let bench_elapsed = start_bench.elapsed().as_secs_f64();
    let rps = total_requests as f64 / bench_elapsed;

    println!("Requests per second: {}", rps);
    println!("Successful requests: {}", successful_requests);
    println!("Failed requests: {}", failed_requests);

    println!("50th percentile: {}", hist.value_at_quantile(0.50));
    println!("95th percentile: {}", hist.value_at_quantile(0.95));
    println!("99th percentile: {}", hist.value_at_quantile(0.99));

    Ok(())
}
