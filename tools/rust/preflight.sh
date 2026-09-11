#!/bin/sh
# Pré-vol des JS fournis par Dioxus : pas de CLI ni installation de Bun.
set -eu
repo_root=$(CDPATH= cd -P "$(dirname "$0")/../.." && pwd)
runner="$repo_root/tools/rust/run.sh"
manifest="$repo_root/Cargo.toml"
host=$(sh "$runner" rustc -vV | sed -n 's/^host: //p')
if [ "$host" != 'aarch64-apple-darwin' ]; then
    printf '%s\n' 'Cette qualification est limitée à macOS ARM64.' >&2
    exit 64
fi

probe_dir=$(mktemp -d "$repo_root/.local/rust/dioxus-preflight.XXXXXX")
cleanup() {
    # Seulement les fichiers connus créés dans ce mktemp ; pas de rm récursif.
    for fixture in matching changed missing hash; do
        fixture_dir="$probe_dir/$fixture"
        if [ -d "$fixture_dir" ]; then
            rm -f "$fixture_dir/src/ts/a.ts" "$fixture_dir/src/js/a.js" "$fixture_dir/src/js/hash.txt"
            rmdir "$fixture_dir/src/ts" "$fixture_dir/src/js" "$fixture_dir/src" "$fixture_dir"
        fi
    done
    rm -f "$probe_dir/tests" "$probe_dir/preflight" "$probe_dir/metadata.json"
    rmdir "$probe_dir"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM

sh "$runner" cargo metadata --manifest-path "$manifest" --locked --offline \
    --filter-platform wasm32-unknown-unknown --format-version 1 > "$probe_dir/metadata.json"
web_manifest=$(jq -er '[.packages[] | select(.name == "dioxus-web" and .version == "0.7.10")] | if length == 1 then .[0].manifest_path else error("dioxus-web inattendu") end' "$probe_dir/metadata.json")
document_manifest=$(jq -er '[.packages[] | select(.name == "dioxus-document" and .version == "0.7.10")] | if length == 1 then .[0].manifest_path else error("dioxus-document inattendu") end' "$probe_dir/metadata.json")
interpreter_manifest=$(jq -er '[.packages[] | select(.name == "dioxus-interpreter-js" and .version == "0.7.10")] | if length == 1 then .[0].manifest_path else error("dioxus-interpreter-js inattendu") end' "$probe_dir/metadata.json")
jq -e '[.packages[] | select(.name == "lazy-js-bundle")] | length == 1 and .[0].version == "0.7.10"' "$probe_dir/metadata.json" > /dev/null

sh "$runner" rustc --edition=2024 --deny warnings --test \
    "$repo_root/tools/rust/dioxus-js-preflight.rs" -o "$probe_dir/tests"
DIOXUS_PREFLIGHT_TEST_DIR="$probe_dir" "$probe_dir/tests"
sh "$runner" rustc --edition=2024 --deny warnings \
    "$repo_root/tools/rust/dioxus-js-preflight.rs" -o "$probe_dir/preflight"
"$probe_dir/preflight" "$(dirname "$web_manifest")" \
    "$(dirname "$document_manifest")" "$(dirname "$interpreter_manifest")"
