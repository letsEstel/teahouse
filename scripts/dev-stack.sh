#!/usr/bin/env bash
# The whole development stack in one command (docs/CLIENT-API.md §6):
#
#   1. a local relay in a scratch directory (no sshd: the engines reach it
#      through its forced command, `teahouse-local://`);
#   2. the six example teahouses of fixtures/houses.zh.json, built as real
#      git history: every member has an engine and a device key of their
#      own, every commit is signed by its author and judged by the relay;
#   3. `teahouse-client serve` on 127.0.0.1:7377 as the member you choose;
#   4. the interface (`npm run dev` in app/ with VITE_BACKEND=local).
#
# Usage:
#   scripts/dev-stack.sh                 # as chen (host of 梧桐里三号院)
#   TEAHOUSE_DEV_ME=zhao scripts/dev-stack.sh
#   scripts/dev-stack.sh --fresh         # throw the scratch data away first
#   NO_UI=1 scripts/dev-stack.sh         # engine only (for curl, tests, shoot)
#
# Settings: TEAHOUSE_DEV_DIR (default ~/.cache/teahouse/dev), PORT (7377).
# The data survives restarts; --fresh rebuilds it from the fixtures.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
dev="${TEAHOUSE_DEV_DIR:-$HOME/.cache/teahouse/dev}"
me="${TEAHOUSE_DEV_ME:-chen}"
port="${PORT:-7377}"
fixtures="$here/fixtures/houses.zh.json"

die() { printf 'dev-stack: %s\n' "$*" >&2; exit 2; }

[[ ${1:-} == --fresh ]] && rm -rf "$dev"
[[ -f $fixtures ]] || die "missing $fixtures (run \`npm run export-fixtures\` in app/)"

echo "building relay and engine…"
cargo build --quiet --release --manifest-path "$here/Cargo.toml" -p teahouse-relay -p teahouse-client ||
  die "build failed"
relay_bin="$here/target/release/teahouse-relay"
client_bin="$here/target/release/teahouse-client"
export TEAHOUSE_RELAY_BIN="$relay_bin"

if [[ ! -d $dev/relay ]]; then
  echo "seeding the example teahouses into $dev …"
  mkdir -p "$dev"
  "$relay_bin" --root "$dev/relay" init --binary "$relay_bin" >/dev/null
  if ! "$client_bin" dev-seed "$fixtures" --relay "$dev/relay" --people "$dev/people"; then
    rm -rf "$dev"
    die "seeding failed"
  fi
fi
[[ -d $dev/people/$me ]] || die "no member '$me'; choose one of: $(ls "$dev/people" | tr '\n' ' ')"

pids=()
cleanup() { for p in "${pids[@]}"; do kill "$p" 2>/dev/null || true; done; }
trap cleanup EXIT INT TERM

"$client_bin" --root "$dev/people/$me" serve --port "$port" &
pids+=($!)
for _ in $(seq 50); do
  curl -sf "http://127.0.0.1:$port/v1/health" >/dev/null 2>&1 && break
  sleep 0.1
done
echo "engine: http://127.0.0.1:$port/v1/  (as $me; data in $dev)"

if [[ ${NO_UI:-} == 1 || ! -f $here/app/package.json ]]; then
  echo "engine only; Ctrl-C to stop"
  wait
else
  [[ -d $here/app/node_modules ]] || (cd "$here/app" && npm install)
  (cd "$here/app" && VITE_BACKEND=local VITE_BACKEND_URL="http://127.0.0.1:$port" npm run dev)
fi
