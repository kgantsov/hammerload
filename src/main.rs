use std::sync::Arc;

use clap::{Parser, ValueEnum};
use hammerload::{metrics::metrics::Metrics, scheduler::scheduler::Scheduler};

#[derive(ValueEnum, Debug, Clone)]
enum Protocol {
    Http,
}

#[derive(Parser, Debug)]
#[command(
    author = "Your Name",
    version = "1.0.0",
    about = "Hammerload - A load testing tool for HTTP protocols"
)]
struct Args {
    #[arg(value_name = "PROTOCOL")]
    protocol: Protocol,

    #[arg(short, long, value_name = "URL")]
    url: String,

    #[arg(short, long, value_name = "CONCURRENCY", default_value_t = 1)]
    concurrency: u64,

    #[arg(short, long, value_name = "DURATION", default_value_t = 10)]
    duration: u64,

    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let logo = r#"
    ██╗  ██╗ █████╗ ███╗   ███╗███╗   ███╗███████╗██████╗ ██╗      ██████╗  █████╗ ██████╗
    ██║  ██║██╔══██╗████╗ ████║████╗ ████║██╔════╝██╔══██╗██║     ██╔═══██╗██╔══██╗██╔══██╗
    ███████║███████║██╔████╔██║██╔████╔██║█████╗  ██████╔╝██║     ██║   ██║███████║██║  ██║
    ██╔══██║██╔══██║██║╚██╔╝██║██║╚██╔╝██║██╔══╝  ██╔══██╗██║     ██║   ██║██╔══██║██║  ██║
    ██║  ██║██║  ██║██║ ╚═╝ ██║██║ ╚═╝ ██║███████╗██║  ██║███████╗╚██████╔╝██║  ██║██████╔╝
    ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═════╝
    "#;
    println!("{}", logo);

    let args = Args::parse();

    let metrics = Arc::new(Metrics::new());

    let mut header_map = reqwest::header::HeaderMap::new();

    for h in &args.headers {
        match h.split_once(':') {
            Some((key, value)) => {
                if let (Ok(header_name), Ok(header_value)) = (
                    reqwest::header::HeaderName::from_bytes(key.trim().as_bytes()),
                    reqwest::header::HeaderValue::from_str(value.trim()),
                ) {
                    header_map.insert(header_name, header_value);
                } else {
                    eprintln!("Invalid header: {}", h);
                }
            }
            None => {
                eprintln!("Invalid header: {}", h);
            }
        }
    }

    let scheduler = Scheduler::new(
        &metrics,
        args.url,
        args.concurrency,
        args.duration,
        header_map,
    );

    scheduler.run().await;

    Ok(())
}
