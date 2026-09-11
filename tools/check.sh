#!/bin/sh
# Suite déterministe complète hors audits distants : aucune recette manuelle.
set -eu
repo_root=$(CDPATH= cd -P "$(dirname "$0")/.." && pwd)
cd "$repo_root"
export AWS_LC_SYS_USE_SYSTEM=0 AWS_LC_SYS_CMAKE_BUILDER=0 AWS_LC_SYS_STATIC=1
runner="$repo_root/tools/rust/run.sh"
node_bin="$repo_root/.local/browser-runtime/node-v22.23.2-darwin-arm64/bin/node"
host=$(sh "$runner" rustc -vV | sed -n 's/^host: //p')
sh "$runner" cargo fmt --all --check
sh "$runner" cargo clippy --locked --offline --target "$host" --all-targets -- -D warnings
sh "$runner" cargo test --locked --offline --target "$host"
sh "$runner" cargo clippy -p chatpurp-web --locked --offline --target wasm32-unknown-unknown -- -D warnings
"$node_bin" --test tools/specdd/validation.test.mjs
"$node_bin" --test tools/repository.test.mjs
sh tools/browser-tests/run.sh policy
build_log=$(mktemp "$repo_root/.local/rust/build-log.XXXXXX")
sh tools/rust/build-app.sh >"$build_log"
export CHATPURP_WEB_ASSETS=$(sed -n 's/^CHATPURP_WEB_ASSETS=//p' "$build_log")
test -n "$CHATPURP_WEB_ASSETS"
export CHATPURP_APP_WASM="$repo_root/.local/rust/target/wasm32-unknown-unknown/debug/chatpurp-web.wasm"
export CHATPURP_TRANSFORM_TEST_DIR=$(mktemp -d "$repo_root/.local/rust/transform-tests.XXXXXX")
sh "$runner" cargo test --manifest-path tools/wasm-build/Cargo.toml --target "$host" --locked --offline -- --test-threads=1
sh tools/browser-tests/run.sh browser
sh tools/browser-tests/run.sh app
