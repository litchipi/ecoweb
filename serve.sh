#!/bin/sh

BLOG_DATA_DIR=../blog

clear
cargo run --features dev -- \
  --config-file $BLOG_DATA_DIR/config.toml \
  --site-config-file $BLOG_DATA_DIR/site.toml \
  --favicon $BLOG_DATA_DIR/favicon.png \
  --scss $BLOG_DATA_DIR/scss \
  --js $BLOG_DATA_DIR/scripts \
  --html $BLOG_DATA_DIR/templates \
  --out $BLOG_DATA_DIR/assets \
  --add $BLOG_DATA_DIR/images \
  --posts $BLOG_DATA_DIR/posts \
  --series $BLOG_DATA_DIR/series.toml
