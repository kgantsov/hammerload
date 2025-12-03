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

Core load parameters
```
-d, --duration <SECONDS>           Duration of test in seconds
-c, --concurrency <N>              Number of concurrent connections
-r, --rate <N>                     Requests per second
-t, --timeout <SECONDS>            Timeout in seconds
```

Request options
```
-u, --url <URL>                    URL to send requests to
-X, --method <METHOD>              HTTP method (GET, POST, PUT, PATCH, DELETE, ...)
-H, --header <HEADER>              Custom headers (repeatable)
-b, --body <STRING>                Request body
-F, --form <KEY=VALUE>             Form fields (repeatable)
```

## Examples

Benchmark an HTTP service for 10 seconds with 1 worker
```bash
hammerload http -url http://localhost:8000/files/1
```

Passing headers
```bash
hammerload http -u http://localhost:8000/files/1 -H "Authorization: Bearer TOKEN" -H "Content-Type: application/json"
```

Make POST request with json body
```bash
hammerload http -X POST -u http://localhost:8000/files/ --duration 1 --concurrency 1 -b '{"filename": "test.txt", "directory_path": "", "file_type": "file", "checksum": "checksum", "size": 0}'
```

Make POST request with form parameters
```bash
hammerload http -X POST -u http://localhost:8000/files/ --duration 1 --concurrency 1 -F "filename=test.txt" -F "file_type=file" -F "checksum=checksum" -F "size=0"
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
    
Requests per second: 33638.40847745677
Successful requests: 336485
Failed requests: 0
50th percentile: 3ms
95th percentile: 17ms
99th percentile: 33ms
```
