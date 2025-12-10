# --- Stage 1: Builder ---
FROM rust:1.83-slim-bookworm AS builder

# Install build deps for reqwest/openssl
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

# --- Stage 2: Runtime ---
FROM debian:bookworm-slim

# Install Chromium + Fonts (Chinese, Japanese, Emoji)
RUN apt-get update && apt-get install -y \
    chromium \
    curl \
    fonts-liberation \
    fonts-noto-cjk \
    fonts-noto-color-emoji \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-pdf-gen /usr/local/bin/app
COPY entrypoint.sh /entrypoint.sh

RUN chmod +x /entrypoint.sh

RUN useradd -m appuser
USER appuser

ENV PORT=3000
EXPOSE 3000

ENTRYPOINT ["/entrypoint.sh"]