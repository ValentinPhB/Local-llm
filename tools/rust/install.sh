#!/bin/sh
# Installation confinée au dépôt ; pas de modification du profil du shell.
set -eu
repo_root=$(CDPATH= cd -P "$(dirname "$0")/../.." && pwd)
test "$(uname -s)" = Darwin && test "$(uname -m)" = arm64
export CARGO_HOME="$repo_root/.local/rust/cargo" RUSTUP_HOME="$repo_root/.local/rust/rustup"
umask 077
if [ ! -x "$CARGO_HOME/bin/rustup" ]; then
    mkdir -p "$repo_root/.local/rust"
    install_dir=$(mktemp -d "$repo_root/.local/rust/install.XXXXXX")
    trap 'rmdir "$install_dir" 2>/dev/null || true' EXIT
    curl --proto '=https' --tlsv1.2 -fL --max-time 120 'https://static.rust-lang.org/rustup/archive/1.29.1/aarch64-apple-darwin/rustup-init' -o "$install_dir/rustup-init"
    actual=$(shasum -a 256 "$install_dir/rustup-init")
    case "$actual" in 'ec1b9233e7f72990ecd8e62063fa7f6c3dfc2bec8e97f88bff165f9100ac696a  '*) ;; *) echo 'Empreinte rustup invalide.' >&2; exit 1 ;; esac
    chmod 700 "$install_dir/rustup-init"
    "$install_dir/rustup-init" -y --no-modify-path --default-toolchain none --profile minimal
    # Supprime seulement le binaire téléchargé dans ce dossier temporaire créé ici.
    rm "$install_dir/rustup-init"
fi
"$CARGO_HOME/bin/rustup" toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy --target wasm32-unknown-unknown
