#!/bin/sh

BLOG_DATA_DIR=$(realpath ../blog)

export GIT_WEBHOOK_SECRET="test"
clear
cargo run --features "dev" -- \
  --data-dir "$BLOG_DATA_DIR" \
  --out ./target/assets
