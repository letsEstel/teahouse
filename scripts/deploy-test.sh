#!/usr/bin/env bash
# 部署冒烟测试 · Deployment smoke test
#
# Proves that a relay installed the shipped way actually serves members:
#   install  — runs deploy/install.sh on this machine (needs root; meant for a
#              throw-away CI runner or VM), then tests it on 127.0.0.1:22
#   docker   — builds deploy/docker/Dockerfile, runs it on 127.0.0.1:2222,
#              then runs the same member checks
#
#   sudo ./scripts/deploy-test.sh install target/release/teahouse-relay
#   ./scripts/deploy-test.sh docker
set -euo pipefail
mode="${1:?install|docker}"
here="$(cd "$(dirname "$0")/.." && pwd)"
work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT
pass=0; fail=0
check() { local name="$1"; shift; if "$@" >"$work/out" 2>&1; then echo "  ✓ $name"; pass=$((pass+1)); else echo "  ✗ $name"; sed 's/^/      /' "$work/out" | tail -20; fail=$((fail+1)); fi; }
refused() { local name="$1"; shift; if "$@" >"$work/out" 2>&1; then echo "  ✗ $name (should have been refused)"; fail=$((fail+1)); else echo "  ✓ $name (refused)"; pass=$((pass+1)); fi; }

ssh-keygen -q -t ed25519 -N '' -C host@test -f "$work/host"
ssh-keygen -q -t ed25519 -N '' -C stranger@test -f "$work/stranger"
chmod 600 "$work/host" "$work/stranger"

case "$mode" in
  install)
    bin="${2:?path to teahouse-relay}"
    [ "$(id -u)" -eq 0 ] || { echo "run as root"; exit 2; }
    sh "$here/deploy/install.sh" --binary "$bin" --open smoke --name "冒烟测试" --host-name "Tester" --host-key "@$work/host.pub"
    echo "▌ second run (idempotent, upgrade path)"
    sh "$here/deploy/install.sh" --binary "$bin" --open smoke --name "冒烟测试" --host-name "Tester" --host-key "@$work/host.pub" >/dev/null
    port=22 ;;
  docker)
    docker build -q -f "$here/deploy/docker/Dockerfile" -t teahouse-relay:test "$here" >/dev/null
    docker rm -f th-smoke >/dev/null 2>&1 || true
    docker run -d --name th-smoke -p 127.0.0.1:2222:22 -v th-smoke:/var/lib/teahouse teahouse-relay:test >/dev/null
    trap 'docker rm -f th-smoke >/dev/null 2>&1; docker volume rm th-smoke >/dev/null 2>&1; rm -rf "$work"' EXIT
    for _ in $(seq 30); do docker logs th-smoke 2>&1 | grep -q "relay ready" && break; sleep 1; done
    docker exec -u teahouse th-smoke teahouse-relay open smoke --name "冒烟测试" --host-id tester --host-name Tester --host-key "$(cat "$work/host.pub")"
    port=2222 ;;
  *) echo "unknown mode $mode"; exit 2 ;;
esac

export GIT_SSH_COMMAND="ssh -p $port -o BatchMode=yes -o ConnectTimeout=10 -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile=$work/known_hosts -o IdentitiesOnly=yes -i $work/host"
stranger="ssh -p $port -o BatchMode=yes -o ConnectTimeout=10 -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile=$work/known_hosts -o IdentitiesOnly=yes -i $work/stranger"
echo "▌ member checks on 127.0.0.1:$port"
check "host sees their teahouse (ssh -T)" bash -c "$GIT_SSH_COMMAND -T teahouse@127.0.0.1 | grep -q smoke"
check "host clones" git clone -q teahouse@127.0.0.1:smoke "$work/clone"
cd "$work/clone" || { echo "▌ $pass passed, $((fail+1)) failed (no clone)"; exit 1; }
git config user.name Tester; git config user.email tester@teahouse
git config gpg.format ssh; git config user.signingkey "$work/host.pub"; git config commit.gpgsign true
mkdir -p problems; printf -- '---\ntitle: 第一个问题\nstatus: open\n---\n测试\n' > problems/0001-first.md
git add -A; git commit -q -m $'提出问题\n\nProblem: meta\nAct: open-problem'
check "host pushes a signed commit" git push -q origin main
git commit -q --allow-empty --no-gpg-sign -m $'未签名\n\nProblem: 0001'
refused "unsigned commit" git push -q origin main
git reset -q --hard origin/main
refused "stranger cannot clone" bash -c "GIT_SSH_COMMAND='$stranger' git clone -q teahouse@127.0.0.1:smoke $work/x"
refused "no shell for members" bash -c "$GIT_SSH_COMMAND teahouse@127.0.0.1 bash -c id"
check "stranger reaches only redeem (bad code refused politely)" bash -c "$stranger -T teahouse@127.0.0.1 redeem TH-0000-0000-0000-0000 --name X 2>&1 | grep -qiE 'invit|邀请'"
if [ "$fail" -gt 0 ] && [ "$mode" = install ]; then
  echo "▌ diagnostics"
  namei -l /usr/lib/teahouse/teahouse-relay 2>&1 | sed 's/^/    /'
  (journalctl --no-pager -n 200 2>/dev/null | grep -iE 'sshd|teahouse' | grep -iE 'unsafe|refus|denied|invalid|error|fail' | tail -12) | sed 's/^/    /'
fi
echo "▌ $pass passed, $fail failed"
[ "$fail" -eq 0 ]
