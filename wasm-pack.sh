#!/bin/sh

date=$(date +%s)
if [ "$1" = *"release"* ]; then
  CONF="--release"
  FILE_PATH="release"
else
  # Otherwise, build in debug mode
  CONF=""
  FILE_PATH="debug"
fi

rm -rf pkg/gifski_bg.wasm

RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' \
  cargo +nightly build --target wasm32-unknown-unknown \
  --no-default-features \
  --features wasm \
  ${CONF} -Z build-std=std,panic_abort
#   -Z build-std-features=panic_immediate_abort

# Note the usage of `--target no-modules` here which is required for passing
# the memory import to each wasm module.
wasm-bindgen \
  target/wasm32-unknown-unknown/${FILE_PATH}/gifski.wasm \
  --keep-debug \
  --target web \
  --out-dir ./pkg
