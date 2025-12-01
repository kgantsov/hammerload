# Hammerload - HTTP benchmarking tool
### A fast, minimal, Rust-powered HTTP benchmarking tool.

hammerload is a lightweight, high-performance benchmarking CLI for stress-testing HTTP services.
It supports configurable concurrency and time-based test duration—making it ideal for quickly profiling APIs, microservices, and web backends.

## Installation

### Build from source:

```bash
git clone https://github.com/kgantsov/hammerload.git
cd hammerload
cargo build --release
```

## Usage

```bash
hammerload <PROTOCOL> --url <URL> [OPTIONS]

```

## Examples

Benchmark an HTTP service for 10 seconds with 1 worker
```bash
hammerload http -url http://localhost:8000/files/1
```

Benchmark an HTTP service for 30 seconds with 200 workers

```bash
hammerload http -url http://localhost:8000/files/1 --duration 30 --concurrency 200

    ██╗  ██╗ █████╗ ███╗   ███╗███╗   ███╗███████╗██████╗ ██╗      ██████╗  █████╗ ██████╗
    ██║  ██║██╔══██╗████╗ ████║████╗ ████║██╔════╝██╔══██╗██║     ██╔═══██╗██╔══██╗██╔══██╗
    ███████║███████║██╔████╔██║██╔████╔██║█████╗  ██████╔╝██║     ██║   ██║███████║██║  ██║
    ██╔══██║██╔══██║██║╚██╔╝██║██║╚██╔╝██║██╔══╝  ██╔══██╗██║     ██║   ██║██╔══██║██║  ██║
    ██║  ██║██║  ██║██║ ╚═╝ ██║██║ ╚═╝ ██║███████╗██║  ██║███████╗╚██████╔╝██║  ██║██████╔╝
    ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═════╝
    
Requests per second: 33010.18370607029
Successful requests: 330630
Failed requests: 0
50th percentile: 3509
95th percentile: 18063
99th percentile: 33535
```
