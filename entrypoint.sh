#!/bin/bash

# 1. Start Chromium in background
# Added flags to silence errors:
# --log-level=3: Only show fatal crashes
# --disable-logging: Stop debug logs
# --disable-breakpad: Stop crash reporting to Google
# --no-first-run: Skip "Welcome to Chrome" logic
echo "🚀 Starting Chromium in background..."
chromium \
  --headless=new \
  --no-sandbox \
  --disable-gpu \
  --disable-dev-shm-usage \
  --remote-debugging-port=9222 \
  --remote-debugging-address=0.0.0.0 \
  --log-level=3 \
  --disable-logging \
  --disable-breakpad \
  --no-first-run \
  --mute-audio \
  &

# 2. Wait for Chromium to wake up
echo "⏳ Waiting for Chromium to be ready..."
until curl -s http://127.0.0.1:9222/json/version > /dev/null; do
  sleep 0.1
done
echo "✅ Chromium is ready!"

# 3. Start Rust App
exec /usr/local/bin/app