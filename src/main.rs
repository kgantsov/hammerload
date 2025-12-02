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
    author = "Your Name",
    version = "1.0.0",
    about = "Hammerload - A load testing tool for HTTP protocols"
)]
struct Args {
    #[arg(value_name = "PROTOCOL")]
    protocol: Protocol,

    #[arg(short = 'X', long, value_name = "METHOD", default_value = "GET")]
    method: Method,

    #[arg(short, long, value_name = "URL")]
    url: String,

    #[arg(short, long, value_name = "BODY")]
    body: Option<String>,

    #[arg(short, long, value_name = "CONCURRENCY", default_value_t = 1)]
    concurrency: u64,

    #[arg(short, long, value_name = "DURATION", default_value_t = 10)]
    duration: u64,

    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,

    #[arg(short = 'F', long = "form")]
    form_params: Vec<String>,

    // timeout
    #[arg(short, long, value_name = "TIMEOUT", default_value_t = 5)]
    timeout: u64,
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
        args.timeout,
    );

    scheduler.run().await;

    Ok(())
}
