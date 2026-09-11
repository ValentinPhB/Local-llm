#!/bin/sh
set -eu
repo_root=$(CDPATH= cd -P "$(dirname "$0")/../.." && pwd)
runner="$repo_root/tools/rust/run.sh"
export AWS_LC_SYS_USE_SYSTEM=0 AWS_LC_SYS_CMAKE_BUILDER=0 AWS_LC_SYS_STATIC=1 RAYON_NUM_THREADS=1
umask 077
host=$(sh "$runner" rustc -vV | sed -n 's/^host: //p')
sh "$repo_root/tools/rust/preflight.sh"
sh "$runner" cargo build --manifest-path "$repo_root/Cargo.toml" -p chatpurp-api --bins --target "$host" --locked --offline
sh "$runner" cargo build --manifest-path "$repo_root/Cargo.toml" -p chatpurp-web --target wasm32-unknown-unknown --locked --offline
app_assets=$(mktemp -d "$repo_root/.local/rust/web-build.XXXXXX")
sh "$runner" cargo run --manifest-path "$repo_root/tools/wasm-build/Cargo.toml" --bin chatpurp-wasm-transform --target "$host" --locked --offline -- "$repo_root/.local/rust/target/wasm32-unknown-unknown/debug/chatpurp-web.wasm" "$app_assets/assets"
printf 'CHATPURP_WEB_ASSETS=%s\n' "$app_assets/assets"
