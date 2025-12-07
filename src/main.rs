use std::{collections::HashMap, sync::Arc};

use clap::Parser;
use hammerload::{
    commands::{Cli, Command},
    metrics::metrics::Metrics,
    requester::params::RequestParams,
    scheduler::scheduler::Scheduler,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    if !cli.no_logo {
        let logo = r#"
    ██╗  ██╗ █████╗ ███╗   ███╗███╗   ███╗███████╗██████╗ ██╗      ██████╗  █████╗ ██████╗
    ██║  ██║██╔══██╗████╗ ████║████╗ ████║██╔════╝██╔══██╗██║     ██╔═══██╗██╔══██╗██╔══██╗
    ███████║███████║██╔████╔██║██╔████╔██║█████╗  ██████╔╝██║     ██║   ██║███████║██║  ██║
    ██╔══██║██╔══██║██║╚██╔╝██║██║╚██╔╝██║██╔══╝  ██╔══██╗██║     ██║   ██║██╔══██║██║  ██║
    ██║  ██║██║  ██║██║ ╚═╝ ██║██║ ╚═╝ ██║███████╗██║  ██║███████╗╚██████╔╝██║  ██║██████╔╝
    ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═════╝
    "#;
        println!("{}", logo);
    }

    let metrics = Arc::new(Metrics::new());

    let request_params = parse_request_params(cli.command);

    let scheduler = Scheduler::new(
        &metrics,
        cli.concurrency,
        cli.duration,
        cli.rate,
        cli.timeout,
        !cli.no_progress,
        request_params,
    );

    scheduler.run().await;

    Ok(())
}

fn parse_request_params(command: Command) -> RequestParams {
    match command {
        Command::Http {
            url,
            method,
            body,
            headers,
            form,
        } => {
            let mut form_params = HashMap::new();
            let mut header_map = reqwest::header::HeaderMap::new();

            for h in &headers {
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
            for h in &form {
                match h.split_once('=') {
                    Some((key, value)) => {
                        form_params.insert(key.trim().to_string(), value.trim().to_string());
                    }
                    None => {
                        eprintln!("Invalid form parameter: {}", h);
                    }
                }
            }

            RequestParams::Http(hammerload::requester::params::HttpParams {
                url,
                method,
                body,
                headers: header_map,
                form: form_params,
            })
        }
        Command::Grpc {
            address,
            proto,
            method,
            data,
        } => RequestParams::Grpc(hammerload::requester::params::GrpcParams {
            address,
            proto,
            method,
            data,
        }),
    }
}
