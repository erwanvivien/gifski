#!/bin/sh

date=$(date +%s)
case $1 in
  *"-r"*) # Matches -r or --release
    CONF="--release -Z build-std-features=panic_immediate_abort"
    FILE_PATH="release"
    ;;
  *)
    CONF=""
    FILE_PATH="debug"
    ;;
esac

rm -rf pkg/gifski_bg.wasm

RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' \
  cargo +nightly build --target wasm32-unknown-unknown \
  --no-default-features \
  --features wasm \
  ${CONF} -Z build-std=std,panic_abort

# Note the usage of `--target no-modules` here which is required for passing
# the memory import to each wasm module.
wasm-bindgen \
  target/wasm32-unknown-unknown/${FILE_PATH}/gifski.wasm \
  --keep-debug \
  --target no-modules \
  --out-dir ./pkg

sed -i '' 's/input = fetch(input)/input = fetch("http:\/\/localhost:3000\/gifski_bg.wasm")/' pkg/gifski.js
npx esbuild --bundle pkg/simple-worker.js | sed 's/new URL("gifski/new URL("http:\/\/localhost:3000\/gifski/' > pkg/dist_worker.js

echo "Built with $FILE_PATH"
