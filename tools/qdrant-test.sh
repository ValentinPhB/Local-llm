#!/bin/sh
# Le conteneur créé ici n'a aucun volume du lab et possède un port hôte aléatoire.
set -eu
repo_root=$(CDPATH= cd -P "$(dirname "$0")/.." && pwd)
image='qdrant/qdrant:v1.19.1-unprivileged@sha256:801777072776dc81b2a9dd2007b2ed487571f21ecd30efffd15ddb1671f2193d'
container=$(docker run -d --read-only --cap-drop=ALL --security-opt=no-new-privileges --memory=256m --cpus=1 --tmpfs /qdrant/storage:rw,uid=1000,gid=1000,mode=0700 --tmpfs /qdrant/snapshots:rw,uid=1000,gid=1000,mode=0700 -p 127.0.0.1::6333 "$image")
case "$container" in ''|*[!a-f0-9]*) echo 'ID de conteneur invalide.' >&2;exit 1;;esac
test "${#container}" -eq 64
trap 'docker rm -f "$container" >/dev/null' EXIT INT TERM
mapping=$(docker port "$container" 6333/tcp)
case "$mapping" in 127.0.0.1:*) port=${mapping#127.0.0.1:};;*) echo 'Port inattendu.' >&2;exit 1;;esac
case "$port" in ''|*[!0-9]*) exit 1;;esac
attempt=0
until curl -fsS --max-time 1 "http://127.0.0.1:$port/readyz" >/dev/null; do
    attempt=$((attempt+1));test "$attempt" -lt 20 || exit 1;sleep 1
done
export AWS_LC_SYS_USE_SYSTEM=0 AWS_LC_SYS_CMAKE_BUILDER=0 AWS_LC_SYS_STATIC=1
host=$(sh "$repo_root/tools/rust/run.sh" rustc -vV | sed -n 's/^host: //p')
sh "$repo_root/tools/rust/run.sh" cargo run -p chatpurp-api --bin qdrant-contract-test --target "$host" --locked --offline -- "$port"
