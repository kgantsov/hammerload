use clap::{Parser, Subcommand};
use reqwest::Method;

#[derive(Parser, Debug)]
#[command(
    name = "hammerload",
    version,
    about = "Hammerload - A load testing tool"
)]
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "CONCURRENCY",
        default_value_t = 1,
        help = "Number of concurrent connections"
    )]
    pub concurrency: u64,

    #[arg(
        short,
        long,
        value_name = "DURATION",
        default_value_t = 10,
        help = "Duration of test in seconds"
    )]
    pub duration: u64,

    #[arg(
        short,
        long,
        value_name = "RATE",
        help = "Number of requests per second"
    )]
    pub rate: Option<u64>,

    #[arg(
        short,
        long,
        value_name = "TIMEOUT",
        default_value_t = 5,
        help = "Request timeout in seconds"
    )]
    pub timeout: u64,

    #[arg(
        long = "no-progress",
        default_value_t = false,
        help = "Disable progress bar"
    )]
    pub no_progress: bool,

    #[arg(long = "no-logo", default_value_t = false, help = "Disable logo")]
    pub no_logo: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// HTTP load testing
    Http {
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
        form: Vec<String>,
    },

    /// gRPC load testing
    Grpc {
        #[arg(
            short,
            long,
            value_name = "ADDRESS",
            help = "Address to send requests to"
        )]
        address: String,

        #[arg(long, value_name = "PROTO", help = "Path to the proto file")]
        proto: String,

        #[arg(
            short = 'X',
            long,
            value_name = "METHOD",
            help = "GRPC method for example UserService.GetUser"
        )]
        method: String,

        #[arg(short, long, value_name = "DATA", help = "Data to send")]
        data: Option<String>,
    },
    /// Websocket load testing
    Websocket {
        #[arg(short, long, value_name = "URL", help = "URL to send requests to")]
        url: String,

        #[arg(short, long, value_name = "DATA", help = "Data to send")]
        data: String,
    },
}
