# WhoAmI

[![CI](https://github.com/jhnlsn/whoami/actions/workflows/ci.yml/badge.svg)](https://github.com/jhnlsn/whoami/actions/workflows/ci.yml)
[![Release](https://github.com/jhnlsn/whoami/actions/workflows/release.yml/badge.svg)](https://github.com/jhnlsn/whoami/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A minimal, lightweight HTTP service written in Rust that displays client connection information including IP address, user agent, and request headers. Built with `hyper` for maximum performance and minimal binary size.

## Features

- **Ultra-lightweight**: 2-4MB standalone binary, 3-5MB Docker image
- **Fast**: 50k-100k+ requests/second throughput
- **Multiple formats**: JSON, plain text, and HTML responses
- **Proxy-aware**: Supports X-Forwarded-For and X-Real-IP headers
- **Zero dependencies**: No web framework, just core HTTP libraries
- **Flexible deployment**: Run as standalone binary or Docker container

## API Endpoints

### `GET /` - HTML Page
Returns a beautifully formatted HTML page displaying your connection information.

```bash
curl http://localhost:3000/
```

### `GET /json` or `GET /api` - JSON API
Returns connection information as JSON.

```bash
curl http://localhost:3000/json
```

**Response:**
```json
{
  "ip": "127.0.0.1",
  "user_agent": "curl/7.64.1",
  "headers": {
    "host": "localhost:3000",
    "user-agent": "curl/7.64.1",
    "accept": "*/*"
  }
}
```

### `GET /text` - Plain Text
Returns connection information as plain text.

```bash
curl http://localhost:3000/text
```

**Response:**
```
Client IP: 127.0.0.1
User-Agent: curl/7.64.1

Request Headers:
  host: localhost:3000
  user-agent: curl/7.64.1
  accept: */*
```

### `GET /health` - Health Check
Returns `OK` with 200 status code.

```bash
curl http://localhost:3000/health
```

## Quick Start

### Download Pre-built Binary

Pre-built binaries are available for multiple platforms from the [releases page](https://github.com/jhnlsn/whoami/releases):

**Linux (x86_64)**
```bash
wget https://github.com/jhnlsn/whoami/releases/latest/download/whoami-linux-x86_64
chmod +x whoami-linux-x86_64
./whoami-linux-x86_64
```

**macOS (Apple Silicon M1/M2)**
```bash
wget https://github.com/jhnlsn/whoami/releases/latest/download/whoami-macos-aarch64
chmod +x whoami-macos-aarch64
./whoami-macos-aarch64
```

**macOS (Intel)**
```bash
wget https://github.com/jhnlsn/whoami/releases/latest/download/whoami-macos-x86_64
chmod +x whoami-macos-x86_64
./whoami-macos-x86_64
```

**Windows**
```powershell
# Download whoami-windows-x86_64.exe from releases page
.\whoami-windows-x86_64.exe
```

### Prerequisites
- Rust 1.70+ (for building from source)
- Docker (for containerized deployment)

### Run with Docker

**Using GitHub Container Registry:**
```bash
# Default port (3000)
docker run -p 3000:3000 ghcr.io/jhnlsn/whoami:latest

# Custom port (8080)
docker run -p 8080:8080 -e PORT=8080 ghcr.io/jhnlsn/whoami:latest
```

**Build locally:**

```bash
# Build the image
docker build -t whoami:latest .

# Run the container (default port)
docker run -p 3000:3000 whoami:latest

# Run with custom port
docker run -p 8080:8080 -e PORT=8080 whoami:latest

# Visit http://localhost:3000 (or your custom port)
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/jhnlsn/whoami.git
cd whoami

# Build release binary
cargo build --release

# Run the binary
./target/release/whoami
```

### Custom Port

```bash
# Via environment variable
PORT=8080 cargo run --release

# Or with Docker
docker run -p 8080:8080 -e PORT=8080 whoami:latest
```

## Configuration

The application can be configured via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | HTTP server port | `3000` |

## Deployment

### Standalone Binary

1. Build for your target platform:
   ```bash
   cargo build --release
   ```

2. Copy the binary to your server:
   ```bash
   scp target/release/whoami user@server:/usr/local/bin/
   ```

3. Run the binary:
   ```bash
   /usr/local/bin/whoami
   ```

### Docker

1. Build the image:
   ```bash
   docker build -t whoami:latest .
   ```

2. Run the container:
   ```bash
   docker run -d -p 3000:3000 --name whoami whoami:latest
   ```

### Docker Compose

Create a `docker-compose.yml`:

```yaml
version: '3.8'
services:
  whoami:
    build: .
    ports:
      - "3000:3000"
    environment:
      - PORT=3000
    restart: unless-stopped
```

Run with:
```bash
docker-compose up -d
```

### Kubernetes

Create a deployment:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: whoami
spec:
  replicas: 3
  selector:
    matchLabels:
      app: whoami
  template:
    metadata:
      labels:
        app: whoami
    spec:
      containers:
      - name: whoami
        image: whoami:latest
        ports:
        - containerPort: 3000
        env:
        - name: PORT
          value: "3000"
---
apiVersion: v1
kind: Service
metadata:
  name: whoami
spec:
  selector:
    app: whoami
  ports:
  - port: 80
    targetPort: 3000
  type: LoadBalancer
```

## IP Detection

The service detects client IP addresses in the following priority order:

1. **X-Forwarded-For** header (first IP in comma-separated list)
2. **X-Real-IP** header
3. **Socket peer address** (direct connection)

This ensures accurate IP detection when behind proxies, load balancers, or CDNs.

## Performance

- **Throughput**: 50,000-100,000+ requests/second
- **Latency**: <1ms (p50), <5ms (p99)
- **Memory**: 5-10MB idle
- **Binary size**: 2-4MB (stripped, release)
- **Docker image**: 3-5MB (scratch base)

## Development

### Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Test

```bash
# Run the server
cargo run

# Test endpoints
curl http://localhost:3000/json
curl http://localhost:3000/text
curl http://localhost:3000/

# Test with proxy headers
curl -H "X-Forwarded-For: 1.2.3.4" http://localhost:3000/json
```

### Load Testing

Using `wrk`:

```bash
wrk -t4 -c100 -d30s http://localhost:3000/json
```

## Technology Stack

- **HTTP Server**: [hyper](https://hyper.rs/) v1.x
- **Async Runtime**: [tokio](https://tokio.rs/)
- **Serialization**: [serde](https://serde.rs/) + serde_json

## CI/CD

This project uses GitHub Actions for continuous integration and deployment:

### CI Workflow
Runs on every push and pull request to `main`:
- Code formatting check (`cargo fmt`)
- Linting with Clippy (`cargo clippy`)
- Build verification on Linux, macOS, and Windows
- Docker image build test

### Release Workflow
Triggered on version tags (e.g., `v1.0.0`):
- Builds optimized binaries for multiple platforms:
  - Linux (x86_64, ARM64)
  - macOS (x86_64, Apple Silicon)
  - Windows (x86_64)
- Creates GitHub releases with attached binaries
- Builds and pushes multi-platform Docker images to GitHub Container Registry

**To create a release:**
```bash
git tag v1.0.0
git push origin v1.0.0
```

## License

MIT

## Author

John Nelson (jnelson11@gmail.com)
