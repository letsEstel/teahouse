#!/usr/bin/env bash
# 茶馆 Relay 演示 · Teahouse relay demo
#
# Opens a teahouse on a throw-away local relay and walks through what is
# allowed and what is refused, using real git and the same forced-command
# path OpenSSH uses (sshd itself is replaced by `local-ssh`).
#
#   ./scripts/demo.sh            # build + run
#   KEEP=1 ./scripts/demo.sh     # keep the demo directory afterwards
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
cargo build --quiet --release --manifest-path "$here/Cargo.toml"
bin="$here/target/release/teahouse-relay"

demo="$(mktemp -d "${TMPDIR:-/tmp}/teahouse-demo.XXXXXX")"
[[ "${KEEP:-}" == 1 ]] || trap 'rm -rf "$demo"' EXIT
root="$demo/relay"; keys="$demo/keys"; mkdir -p "$keys"

export TEAHOUSE_LOCAL_KEYS="$keys"
export GIT_SSH_COMMAND="$bin --root $root local-ssh"
export GIT_SSH_VARIANT=simple
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1

bold=$'\e[1m'; green=$'\e[32m'; red=$'\e[31m'; dim=$'\e[2m'; off=$'\e[0m'
step()  { printf '\n%s▌ %s%s\n' "$bold" "$*" "$off"; }
ok()    { printf '  %s✓ %s%s\n' "$green" "$*" "$off"; }
expect_refused() {
  local out
  if out="$("$@" 2>&1)"; then printf '  %s!! this should have been refused%s\n' "$red" "$off"; exit 1; fi
  printf '  %s✗ 被拒绝（符合预期）· refused, as it should be%s\n' "$red" "$off"
  grep -E '✗ ' <<<"$out" | sed 's/^remote: *//; s/ *$//; s/^/    /' || true
}

# Demo keys: real ed25519 keys if ssh-keygen exists, otherwise structurally
# valid stand-ins (local-ssh never checks a signature — sshd would).
mkkey() {
  if command -v ssh-keygen >/dev/null; then
    ssh-keygen -q -t ed25519 -N '' -C "$1@demo" -f "$keys/$1"
  else
    python3 - "$1" "$keys/$1.pub" <<'PY'
import base64, hashlib, struct, sys
name, out = sys.argv[1], sys.argv[2]
alg = b"ssh-ed25519"; pt = hashlib.sha256(name.encode()).digest()
blob = struct.pack(">I", len(alg)) + alg + struct.pack(">I", 32) + pt
open(out, "w").write(f"ssh-ed25519 {base64.b64encode(blob).decode()} {name}@demo\n")
PY
  fi
}
for p in chenli zhou wang; do mkkey "$p"; done
key() { cut -d' ' -f1,2 "$keys/$1.pub"; }

step "1. 开一个 relay，开一间茶馆（老板：陈立）"
"$bin" --root "$root" init --binary "$bin" >/dev/null
"$bin" --root "$root" open wutong --name "梧桐里三号院" --signboard "车位、垃圾、楼道灯，咱们自己管。" \
  --host-id chenli --host-name "陈立" --host-key "@$keys/chenli.pub"

clone() { git clone -q "ssh://$1@local/wutong" "$demo/$1"; git -C "$demo/$1" config user.name "$1"; git -C "$demo/$1" config user.email "$1@demo"; }
commit() { local d="$demo/$1"; shift; git -C "$d" add -A; git -C "$d" commit -q -m "$1"; }

step "2. 老板请周小满做伙计、王阿姨做茶客，并提出问题 0001"
clone chenli
cat > "$demo/chenli/.teahouse/members.toml" <<EOF
version = 1

[teahouse]
name = "梧桐里三号院"
signboard = "车位、垃圾、楼道灯，咱们自己管。"

[[member]]
id = "chenli"
name = "陈立"
role = "host"
key = "$(key chenli)"

[[member]]
id = "zhou"
name = "周小满"
role = "steward"
key = "$(key zhou)"

[[member]]
id = "wang"
name = "王阿姨"
role = "guest"
key = "$(key wang)"
EOF
mkdir -p "$demo/chenli/problems"; echo "# 小区停车位不好找" > "$demo/chenli/problems/0001-停车位.md"
commit chenli $'入座，提出问题 0001\n\nProblem: meta'
git -C "$demo/chenli" push -q origin main && ok "老板推送成功"
"$bin" --root "$root" members wutong

step "3. 伙计直接添加成果"
clone zhou
mkdir -p "$demo/zhou/works/0001"; echo "42 个车位" > "$demo/zhou/works/0001/车位分布图.md"
commit zhou $'添加成果：车位分布图\n\nProblem: 0001'
git -C "$demo/zhou" push -q origin main && ok "伙计推送成功"

step "4. 伙计想给自己升成老板"
sed -i 's/role = "steward"/role = "host"/; 0,/role = "host"/s//role = "guest"/' "$demo/zhou/.teahouse/members.toml"
commit zhou $'我来当老板\n\nProblem: 0001'
expect_refused git -C "$demo/zhou" push -q origin main
git -C "$demo/zhou" reset -q --hard origin/main

step "5. 茶客想直接写 main"
clone wang
echo "错峰共享" > "$demo/wang/works/0001/错峰共享.md"
commit wang $'错峰共享方案\n\nProblem: 0001'
expect_refused git -C "$demo/wang" push -q origin main

step "6. 茶客改为发起提案"
git -C "$demo/wang" push -q origin HEAD:refs/proposals/wang/night-share && ok "提案已提交：refs/proposals/wang/night-share"

step "7. 老板采纳提案"
git -C "$demo/chenli" fetch -q origin main refs/proposals/wang/night-share:refs/remotes/origin/p
git -C "$demo/chenli" reset -q --hard origin/main
git -C "$demo/chenli" merge -q --no-ff origin/p -m $'采纳提案：错峰共享\n\nProblem: 0001'
git -C "$demo/chenli" push -q origin main :refs/proposals/wang/night-share && ok "已合并入茶馆，提案分支已清理"

step "8. 没有面向问题的提交"
echo x > "$demo/chenli/works/0001/x.md"; commit chenli "随手改改"
expect_refused git -C "$demo/chenli" push -q origin main
git -C "$demo/chenli" reset -q --hard origin/main

step "9. 连老板也不能改写历史"
git -C "$demo/chenli" reset -q --hard HEAD~1
echo y > "$demo/chenli/README.md"; commit chenli $'改写\n\nProblem: meta'
expect_refused git -C "$demo/chenli" push -q -f origin main

step "10. 继承：旧 relay 消失，伙计用自己的副本在新 relay 上重开"
git -C "$demo/zhou" pull -q --ff-only
root2="$demo/relay2"
"$bin" --root "$root2" init --binary "$bin" >/dev/null
"$bin" --root "$root2" adopt wutong --from "$demo/zhou" --key "@$keys/zhou.pub"
"$bin" --root "$root2" members wutong

step "11. 审计日志（最后 5 条）"
tail -n 5 "$root/log/audit.log" | sed "s/^/  $dim/; s/$/$off/"

step "12. 健康检查"
"$bin" --root "$root" check
printf '\n%s演示结束。%s%s\n' "$bold" "$off" "${KEEP:+ 目录保留在 $demo}"
