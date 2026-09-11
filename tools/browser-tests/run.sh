#!/bin/sh
set -eu
case "${1:-}" in
  policy|browser|audit|app) task="$1" ;;
  *) echo 'Usage: sh tools/browser-tests/run.sh policy|browser|audit|app' >&2; exit 64 ;;
esac
tool_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(CDPATH= cd -- "$tool_dir/../.." && pwd)
node_bin="$repo_dir/.local/browser-runtime/node-v22.23.2-darwin-arm64/bin/node"
# Refuse injection before Node starts, not only once JavaScript is running.
if env | LC_ALL=C grep -E '^(NODE_OPTIONS|NODE_PATH|PUPPETEER_[A-Za-z0-9_]*|DYLD_[A-Za-z0-9_]*|LD_PRELOAD)=.+' >/dev/null; then
  echo 'Refus : variable pouvant modifier le runtime/navigateur.' >&2; exit 1
fi
test -x "$node_bin"
actual=$(shasum -a 256 "$node_bin")
case "$actual" in
  '18e387c90ab8a8400183e8bdd396376e1e875b91b4c874b894dcade7b35bf572  '*) ;;
  *) echo 'Refus : empreinte du runtime Node.' >&2; exit 1 ;;
esac
cd "$tool_dir"
case "$task" in
  policy) exec "$node_bin" --test --test-concurrency=1 launch-policy.test.mjs audit.test.mjs ;;
  browser) exec "$node_bin" --test --test-concurrency=1 --test-timeout=45000 scenarios/runtime.test.mjs ;;
  audit) exec "$node_bin" audit.mjs ;;
  app) exec "$node_bin" --test --test-concurrency=1 --test-timeout=65000 scenarios/app.test.mjs ;;
esac
