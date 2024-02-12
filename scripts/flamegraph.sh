#!/bin/sh

export CARGO_PROFILE_RELEASE_STRIP=false
export CARGO_PROFILE_RELEASE_DEBUG=true
ROOT_DIR=`realpath "$(dirname $0)/.."`

mkdir -p "$ROOT_DIR/.flamegraph"
cd "$ROOT_DIR/.flamegraph"

echo $PWD

FEATURES_LIST=(rss humans-txt webring add-endpoint dev)
FEATURES=$(IFS=, ; echo "${FEATURES_LIST[*]}")

BLOG_DATA_DIR=$(realpath "$ROOT_DIR/../blog")

export GIT_WEBHOOK_SECRET="test"

clear
cargo flamegraph --features "${FEATURES}" --image-width 8000 -- --data-dir "$BLOG_DATA_DIR" --out ./target/assets --savedata "$BLOG_DATA_DIR/saved_data"
