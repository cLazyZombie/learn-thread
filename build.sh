set -ex
cargo +nightly build --release --target wasm32-unknown-unknown
# cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir ./web ./target/wasm32-unknown-unknown/release/learn_thread.wasm