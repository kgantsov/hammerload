use std::collections::HashMap;
use std::sync::Arc;

use clap::{Parser, ValueEnum};
use hammerload::{metrics::metrics::Metrics, scheduler::scheduler::Scheduler};
use reqwest::Method;

#[derive(ValueEnum, Debug, Clone)]
enum Protocol {
    Http,
}

#[derive(Parser, Debug)]
#[command(
    author = "Kostiantyn Hantsov",
    version = "0.5.0",
    about = "Hammerload - A load testing tool for HTTP protocols"
)]
struct Args {
    #[arg(value_name = "PROTOCOL")]
    protocol: Protocol,

    #[arg(
        short = 'X',
        long,
        value_name = "METHOD",
        default_value = "GET",
        help = "HTTP method (GET, POST, PUT, PATCH, DELETE, ...)"
    )]
    method: Method,

    #[arg(short, long, value_name = "URL", help = "URL to send requests to")]
    url: String,

    #[arg(short, long, value_name = "BODY", help = "Request body")]
    body: Option<String>,

    #[arg(short = 'H', long = "header", help = "Request header (repeatable)")]
    headers: Vec<String>,

    #[arg(short = 'F', long = "form", help = "Form parameters (repeatable)")]
    form_params: Vec<String>,

    #[arg(
        short,
        long,
        value_name = "CONCURRENCY",
        default_value_t = 1,
        help = "Number of concurrent connections"
    )]
    concurrency: u64,

    #[arg(
        short,
        long,
        value_name = "DURATION",
        default_value_t = 10,
        help = "Duration of test in seconds"
    )]
    duration: u64,

    #[arg(
        short,
        long,
        value_name = "RATE",
        help = "Number of requests per second"
    )]
    rate: Option<u64>,

    #[arg(
        short,
        long,
        value_name = "TIMEOUT",
        default_value_t = 5,
        help = "Request timeout in seconds"
    )]
    timeout: u64,

    #[arg(long = "no-progress", default_value_t = false)]
    pub no_progress: bool,
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

    let mut form_params = HashMap::new();
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
    for h in &args.form_params {
        match h.split_once('=') {
            Some((key, value)) => {
                form_params.insert(key.trim().to_string(), value.trim().to_string());
            }
            None => {
                eprintln!("Invalid form parameter: {}", h);
            }
        }
    }

    let scheduler = Scheduler::new(
        &metrics,
        args.method,
        args.url,
        args.body,
        form_params,
        header_map,
        args.concurrency,
        args.duration,
        args.rate,
        args.timeout,
        !args.no_progress,
    );

    scheduler.run().await;

    Ok(())
}
