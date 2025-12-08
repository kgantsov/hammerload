# Hammerload - HTTP benchmarking tool
### A fast, minimal, Rust-powered HTTP benchmarking tool.

hammerload is a lightweight, high-performance benchmarking CLI for stress-testing HTTP services.
It supports multiple protocols, configurable concurrency, and time-based test duration - making it ideal for quickly profiling APIs, microservices, and web backends.

```bash
hammerload --duration 10 --concurrency 200 http -X POST -u http://localhost:8080/webhooks -H  "Content-Type: application/json" --body '{"name": "Test webhook", "method": "POST"}'

    ██╗  ██╗ █████╗ ███╗   ███╗███╗   ███╗███████╗██████╗ ██╗      ██████╗  █████╗ ██████╗
    ██║  ██║██╔══██╗████╗ ████║████╗ ████║██╔════╝██╔══██╗██║     ██╔═══██╗██╔══██╗██╔══██╗
    ███████║███████║██╔████╔██║██╔████╔██║█████╗  ██████╔╝██║     ██║   ██║███████║██║  ██║
    ██╔══██║██╔══██║██║╚██╔╝██║██║╚██╔╝██║██╔══╝  ██╔══██╗██║     ██║   ██║██╔══██║██║  ██║
    ██║  ██║██║  ██║██║ ╚═╝ ██║██║ ╚═╝ ██║███████╗██║  ██║███████╗╚██████╔╝██║  ██║██████╔╝
    ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═════╝
    

Requests:......................732056       73183.64/s
Requests succeded:.............732056          100.00%
Requests failed:...............0                 0.00%
Data sent:.....................48.87 MB      4.89 MB/s
Data received:.................46.78 MB      4.68 MB/s
Latencies:
   Min:........................61µs
   P(50):......................2ms
   P(90):......................4ms
   P(95):......................5ms
   P(99):......................7ms
   P(99.9):....................14ms
   P(99.99):...................95ms
   Max:........................125ms
```

## Installation

### Build from source:

```bash
git clone https://github.com/kgantsov/hammerload.git
cd hammerload
cargo build --release
```

## Usage

```bash
hammerload --help
Hammerload - A load testing tool

Usage: hammerload [OPTIONS] <COMMAND>

Commands:
  http  HTTP load testing
  grpc  gRPC load testing
  help  Print this message or the help of the given subcommand(s)

Options:
  -c, --concurrency <CONCURRENCY>  Number of concurrent connections [default: 1]
  -d, --duration <DURATION>        Duration of test in seconds [default: 10]
  -r, --rate <RATE>                Number of requests per second
  -t, --timeout <TIMEOUT>          Request timeout in seconds [default: 5]
      --no-progress                Disable progress bar
      --no-logo                    Disable logo
  -h, --help                       Print help
  -V, --version                    Print version
```

HTTP Request options
```
-X, --method <METHOD>   HTTP method (GET, POST, PUT, PATCH, DELETE, ...) [default: GET]
-u, --url <URL>         URL to send requests to
-b, --body <BODY>       Request body
-H, --header <HEADERS>  Request header (repeatable)
-F, --form <FORM>       Form parameters (repeatable)
```

GRPC Request options
```
-a, --address <ADDRESS>  Address to send requests to
    --proto <PROTO>      Path to the proto file
-X, --method <METHOD>    GRPC method for example UserService.GetUser
-d, --data <DATA>        Data to send
-h, --help               Print help
```

## Examples

Benchmark an HTTP service for 10 seconds with 1 worker
```bash
hammerload http --url http://localhost:8000/files/1
```

Benchmark an HTTP service for 10 seconds with 10 workers and 100 requests per second
```bash
hammerload --concurrency 10 --rate 100 http --url http://localhost:8000/files/1
```

Make HTTP request and pass some headers
```bash
hammerload http -u http://localhost:8000/files/1 -H "Authorization: Bearer TOKEN" -H "Content-Type: application/json"
```

Make HTTP POST request with json body
```bash
hammerload --duration 1 --concurrency 1 http -X POST -u http://localhost:8000/files/ -b '{"filename": "test.txt", "directory_path": "", "file_type": "file", "checksum": "checksum", "size": 0}'
```

Make HTTP POST request with form parameters
```bash
hammerload --duration 1 --concurrency 1 http -X POST -u http://localhost:8000/files/ -F "filename=test.txt" -F "file_type=file" -F "checksum=checksum" -F "size=0"
```

Make GRPC request

```bash
hammerload --duration 10 --concurrency 200 grpc --address http://localhost:10000 --proto ./proto/doq.proto --method "queue.DOQ.Enqueue" --data '{"queueName": "test", "group": "default", "priority": 300, "content": "test message 3"}'
```
