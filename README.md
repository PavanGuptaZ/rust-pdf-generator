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
