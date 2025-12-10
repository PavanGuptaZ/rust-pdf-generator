# --- Stage 1: Builder ---
FROM rust:1.83-slim-bookworm AS builder

# FIX: Install OpenSSL and pkg-config (Required for compiling reqwest/tungstenite)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Now this will succeed because pkg-config is found
RUN cargo build --release

# --- Stage 2: Runtime ---
FROM debian:bookworm-slim

# Install Chromium and curl (for health check)
RUN apt-get update && apt-get install -y \
    chromium \
    curl \
    fonts-liberation \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-pdf-gen /usr/local/bin/app
COPY entrypoint.sh /entrypoint.sh

RUN chmod +x /entrypoint.sh

# Create non-root user
RUN useradd -m appuser
USER appuser

ENV PORT=3000
EXPOSE 3000

ENTRYPOINT ["/entrypoint.sh"]