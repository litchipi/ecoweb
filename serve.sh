#!/bin/sh

FEATURES_LIST=(rss humans-txt webring hireme add-endpoint dev)
FEATURES=$(IFS=, ; echo "${FEATURES_LIST[*]}")

BLOG_DATA_DIR=$(realpath ../blog)

export GIT_WEBHOOK_SECRET="test"

clear
cargo run --features "${FEATURES}" -- --data-dir "$BLOG_DATA_DIR" --out ./target/assets
