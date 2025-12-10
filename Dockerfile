# --- Builder ---
FROM rust:1.83-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# --- Runtime ---
FROM debian:bookworm-slim

# Install Chromium
RUN apt-get update && apt-get install -y \
    chromium \
    fonts-liberation \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-pdf-gen /usr/local/bin/app

# Non-root user
RUN useradd -m appuser
USER appuser

EXPOSE 3000
CMD ["app"]