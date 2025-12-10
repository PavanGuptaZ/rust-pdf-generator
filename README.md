# Rust PDF Generator

A high-performance PDF generation service built with Rust using the Chrome DevTools Protocol. This service converts HTML to PDF using Chrome/Chromium running in headless mode.

## Features

- **High Performance**: Leverages Chrome's rendering engine for accurate PDF generation
- **WebSocket Integration**: Direct communication with Chrome DevTools Protocol
- **Async/Await**: Built on Tokio for efficient concurrent request handling
- **Connection Pooling**: Reuses HTTP connections for optimal performance
- **Large File Support**: Supports up to 10MB request bodies

## Prerequisites

- Rust 1.56+ (2021 edition)
- Chrome or Chromium browser with remote debugging enabled on port 9222
- Docker (optional, for containerized deployment)

## Installation

1. Clone the repository:

```bash
git clone <repository-url>
cd rust-pdf-gen
```

2. Build the project:

```bash
cargo build --release
```

3. Run the server:

```bash
cargo run --release
```

## Usage

To generate a PDF, send a POST request to the `/generate` endpoint with the following JSON payload:

```json
{
  "html": "<h1>Hello, World!</h1>",
  "landscape": false
}
```

The `html` field is required and should contain the HTML content to be converted to PDF. The `landscape` field is optional and defaults to `false`. If set to `true`, the PDF will be generated in landscape orientation.

## Deployment

### Docker

To deploy the service using Docker, follow these steps:

1. Build the Docker image:

```bash
docker build -t rust-pdf-gen .
```

2. Run the Docker container:

```bash
docker run --rm -p 3000:3000 rust-pdf-gen
```
