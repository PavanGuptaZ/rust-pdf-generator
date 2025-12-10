#!/bin/bash

echo "🚀 Starting Chromium..."
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

echo "⏳ Waiting for Chromium..."
until curl -s http://127.0.0.1:9222/json/version > /dev/null; do
  sleep 0.1
done
echo "✅ Chromium ready!"

exec /usr/local/bin/app