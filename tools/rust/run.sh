#!/bin/sh
# Utilise uniquement l'installation Rust locale du projet.
set -eu

repo_root=$(CDPATH= cd -P "$(dirname "$0")/../.." && pwd)
case "${1-}" in
    cargo|rustc|rustup|rustfmt|cargo-clippy|clippy-driver) rust_tool=$1 ;;
    *) printf '%s\n' 'Usage: sh tools/rust/run.sh <cargo|rustc|rustup|rustfmt|cargo-clippy|clippy-driver> [arguments]' >&2; exit 64 ;;
esac
shift

export CARGO_HOME="$repo_root/.local/rust/cargo"
export RUSTUP_HOME="$repo_root/.local/rust/rustup"
export CARGO_TARGET_DIR="$repo_root/.local/rust/target"
export CARGO_BUILD_JOBS=1
export PATH="$CARGO_HOME/bin:$PATH"
unset RUSTUP_TOOLCHAIN

if [ ! -x "$CARGO_HOME/bin/$rust_tool" ]; then
    printf '%s\n' 'Installation Rust locale absente ou incomplète. Voir docs/release-management.md.' >&2
    exit 69
fi

cd "$repo_root"
exec "$CARGO_HOME/bin/$rust_tool" "$@"
