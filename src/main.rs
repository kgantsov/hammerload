use std::sync::Arc;

use clap::{Parser, ValueEnum};
use hammerload::{metrics::metrics::Metrics, scheduler::scheduler::Scheduler};

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

    #[arg(short, long, value_name = "CONCURRENCY", default_value_t = 1)]
    concurrency: u64,

    #[arg(short, long, value_name = "DURATION", default_value_t = 10)]
    duration: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();

    let metrics = Arc::new(Metrics::new());
    let scheduler = Scheduler::new(&metrics, args.url, args.concurrency, args.duration);

    scheduler.run().await;

    Ok(())
}
