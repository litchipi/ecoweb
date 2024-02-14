#!/bin/sh

ROOT_DIR=`realpath "$(dirname $0)/.."`

FEATURES_LIST=(rss humans-txt webring add-endpoint save-data dev)
# FEATURES_LIST=(rss humans-txt webring local_storage add-endpoint save-data githook minify)
FEATURES=$(IFS=, ; echo "${FEATURES_LIST[*]}")

BLOG_DATA_DIR=$(realpath "$ROOT_DIR/../blog")

export GIT_WEBHOOK_SECRET="test"

cd "$ROOT_DIR"

clear
rm -rf ./target/assets
cargo run --features "${FEATURES}" -- --data-dir "$BLOG_DATA_DIR" --out ./target/assets --savedata "$BLOG_DATA_DIR/saved_data"
