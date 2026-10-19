#!/usr/bin/env bash
# 真实 sshd 集成测试 · Real-OpenSSH integration test
#
# Runs an unprivileged sshd on 127.0.0.1:$PORT in front of throw-away relays
# and checks the whole member path with real ssh and git, once per way of
# wiring sshd to the relay:
#
#   A  AuthorizedKeysCommand        (deploy/sshd/teahouse.conf, keys looked up live)
#   B  init --authorized-keys file  (sync-keys / post-receive keep the file current)
#
# The sshd configs are generated from deploy/sshd/teahouse.conf, so this tests
# the file we ship. No root needed. If no sshd is installed, the Debian/Ubuntu
# package is unpacked into ~/.cache/teahouse/sshd (apt-get download + dpkg -x).
#
#   ./scripts/sshd-test.sh             # both methods
#   ./scripts/sshd-test.sh A           # one method (A or B)
#   PORT=2299 KEEP=1 ./scripts/sshd-test.sh
#
# Non-root caveat for A: sshd only runs an AuthorizedKeysCommand whose path is
# root-owned, which a binary in target/ is not. The test first checks that
# sshd refuses the shipped line as is, then starts the relay through the
# root-owned /usr/bin/env. In production install the binary as root (RELAY.md §5).
set -uo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
port="${PORT:-2222}"
methods="${1:-A B}"
user="$(id -un)"
conf_src="$here/deploy/sshd/teahouse.conf"

die() { printf 'sshd-test: %s\n' "$*" >&2; exit 2; }

# ── sshd ──────────────────────────────────────────────────────────────────
cache="$HOME/.cache/teahouse/sshd"
sshd="${SSHD:-}"
[[ -z $sshd ]] && sshd="$(command -v sshd || true)"
[[ -z $sshd && -x /usr/sbin/sshd ]] && sshd=/usr/sbin/sshd
[[ -z $sshd && -x $cache/usr/sbin/sshd ]] && sshd="$cache/usr/sbin/sshd"
if [[ -z $sshd ]]; then
  command -v apt-get >/dev/null && command -v dpkg >/dev/null ||
    die "no sshd found; install openssh-server or set SSHD=/path/to/sshd"
  echo "sshd not installed; unpacking openssh-server into $cache (no root needed)"
  mkdir -p "$cache"
  (cd "$cache" && apt-get download -q openssh-server libwrap0 >/dev/null &&
    for d in ./*.deb; do dpkg -x "$d" .; done && rm -f ./*.deb) || die "could not fetch openssh-server"
  sshd="$cache/usr/sbin/sshd"
fi
[[ $sshd == "$cache"/* ]] && export LD_LIBRARY_PATH="$cache/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
"$sshd" -V >/dev/null 2>&1 || die "$sshd does not start (missing libraries?)"
command -v ssh >/dev/null && command -v ssh-keygen >/dev/null || die "need the openssh client (ssh, ssh-keygen)"
(exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null && die "port $port is already in use (set PORT=…)"

cargo build --quiet --release --manifest-path "$here/Cargo.toml" || die "build failed"
bin="$here/target/release/teahouse-relay"
client="$here/target/release/teahouse-client"

# ── scratch space ─────────────────────────────────────────────────────────
# Under $HOME, not /tmp: with StrictModes, sshd refuses an authorized_keys
# file below a world-writable directory.
mkdir -p "$HOME/.cache/teahouse"
work="$(mktemp -d "$HOME/.cache/teahouse/sshd-test.XXXXXX")"
chmod 700 "$work"
sshd_pid=""; bg_pids=()
cleanup() {
  [[ -n $sshd_pid ]] && kill "$sshd_pid" 2>/dev/null
  for p in "${bg_pids[@]}"; do kill "$p" 2>/dev/null; done
  wait 2>/dev/null
  if [[ "${KEEP:-}" == 1 ]]; then echo "kept $work"; else rm -rf "$work"; fi
}
trap cleanup EXIT

export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_TERMINAL_PROMPT=0
unset SSH_AUTH_SOCK GIT_SSH GIT_SSH_COMMAND GIT_SSH_VARIANT TEAHOUSE_ROOT

keys="$work/keys"; mkdir -p "$keys"
ssh-keygen -q -t ed25519 -N '' -f "$work/host_key"
for p in chenli zhou wang outsider; do ssh-keygen -q -t ed25519 -N '' -C "$p@test" -f "$keys/$p"; done
pubkey() { cut -d' ' -f1,2 "$keys/$1.pub"; }

# ── reporting ─────────────────────────────────────────────────────────────
bold=$'\e[1m'; green=$'\e[32m'; red=$'\e[31m'; dim=$'\e[2m'; off=$'\e[0m'
[[ -t 1 ]] || { bold=; green=; red=; dim=; off=; }
passed=0; failed=0; failures=()
section() { printf '\n%s%s%s\n' "$bold" "$*" "$off"; }
pass() { passed=$((passed + 1)); printf '  %s✓%s %s %s%s%s\n' "$green" "$off" "$1" "$dim" "${2:-}" "$off"; }
fail() {
  failed=$((failed + 1)); failures+=("$method $1")
  printf '  %s✗ %s%s\n' "$red" "$1" "$off"
  [[ -n ${2:-} ]] && sed 's/^/      /' <<<"$2" | tail -n 12
}
# expect ok|fail <name> <needle> <command…>: the command must succeed or fail,
# and (if given) its output must contain <needle>.
expect() {
  local want=$1 name=$2 needle=$3; shift 3
  local out status
  out="$("$@" 2>&1)"; status=$?
  local got=ok; [[ $status -ne 0 ]] && got=fail
  if [[ $got != "$want" ]]; then
    fail "$name (expected $want, got exit $status)" "$out"
  elif [[ -n $needle && $out != *"$needle"* ]]; then
    fail "$name (output lacks: $needle)" "$out"
  else
    local line
    if [[ -n $needle ]]; then line="$(grep -m1 -F -- "$needle" <<<"$out")"; else line="$(head -n1 <<<"$out")"; fi
    line="$(sed 's/^remote: *//; s/ *$//' <<<"$line")"; pass "$name" "${line:0:90}"
  fi
}
# Setup steps must work; if one does not, the rest of the method is meaningless.
must() { local out; out="$("$@" 2>&1)" || { fail "setup: $*" "$out"; return 1; }; }

# ── ssh / git as a member ─────────────────────────────────────────────────
ssh_opts() {
  printf '%s ' -F /dev/null -p "$port" -i "$keys/$1" -o IdentitiesOnly=yes -o BatchMode=yes \
    -o StrictHostKeyChecking=accept-new -o "UserKnownHostsFile=$work/known_hosts"
}
ssh_as() { local who=$1; shift; ssh $(ssh_opts "$who") "$@"; }
git_as() { local who=$1; shift; GIT_SSH_COMMAND="ssh $(ssh_opts "$who")" git "$@"; }
url="ssh://$user@localhost:$port/wutong"
commit() { git -C "$1" add -A && git -C "$1" commit -q -m "$2"; }
stamp() { date -u +%Y%m%dT%H%M%SZ; }

# ── sshd configs, generated from the shipped file ─────────────────────────
global_conf() {
  cat <<EOF
Port $port
ListenAddress 127.0.0.1
HostKey $work/host_key
PidFile $work/sshd.pid
UsePAM no
StrictModes yes
LogLevel VERBOSE
EOF
}
# shipped_conf <relay-root> <AuthorizedKeysCommand program prefix>
shipped_conf() {
  global_conf
  sed -e "s|^Match User teahouse\$|Match User $user|" \
      -e "s|^\(    AuthorizedKeysCommand \)/usr/local/bin/teahouse-relay --root /var/lib/teahouse |\1$2 --root $1 |" \
      -e "s|^\(    AuthorizedKeysCommandUser \)teahouse\$|\1$user|" "$conf_src"
}
conf_A() { shipped_conf "$1" "$2"; }
conf_B() {
  shipped_conf "$1" "$bin" | sed -e "s|^\(    AuthorizedKeysFile \)none\$|\1$2|" -e '/^    AuthorizedKeysCommand/d'
}
# Make sure every substitution hit: a silent miss would test a different config.
verify_conf() {
  local f=$1 m=$2
  grep -q "^Match User $user\$" "$f" || die "deploy/sshd/teahouse.conf changed shape: no 'Match User teahouse'"
  if [[ $m == A ]]; then
    grep -q "^    AuthorizedKeysCommand .* --root $work/" "$f" || die "deploy/sshd/teahouse.conf changed shape: AuthorizedKeysCommand"
    grep -q "^    AuthorizedKeysCommandUser $user\$" "$f" || die "deploy/sshd/teahouse.conf changed shape: AuthorizedKeysCommandUser"
  else
    grep -q "^    AuthorizedKeysFile $work/" "$f" || die "deploy/sshd/teahouse.conf changed shape: AuthorizedKeysFile none"
  fi
  "$sshd" -t -f "$f" || die "sshd -t rejected $f"
}

start_sshd() {
  stop_sshd
  "$sshd" -D -e -f "$1" >"$2" 2>&1 & sshd_pid=$!
  for _ in $(seq 50); do grep -q "Server listening" "$2" && return 0; sleep 0.1; done
  cat "$2" >&2; die "sshd did not start"
}
stop_sshd() { [[ -n $sshd_pid ]] && kill "$sshd_pid" 2>/dev/null && wait "$sshd_pid" 2>/dev/null; sshd_pid=""; }

# ── forwarding probes ─────────────────────────────────────────────────────
# -L: the client listens locally; whatever connects must get nothing back.
probe_L() {
  local who=$1 lport=$((port + 77))
  ssh $(ssh_opts "$who") -N -L "127.0.0.1:$lport:127.0.0.1:$port" "$user@localhost" 2>/dev/null &
  local p=$! up=""; bg_pids+=("$p")
  for _ in $(seq 30); do (exec 3<>"/dev/tcp/127.0.0.1/$lport") 2>/dev/null && { up=1; break; }; sleep 0.1; done
  [[ -n $up ]] || { kill "$p" 2>/dev/null; echo "the -L listener never came up"; return 2; }
  local got
  got="$(timeout 3 bash -c "exec 3<>/dev/tcp/127.0.0.1/$lport && head -c 32 <&3" 2>/dev/null)"
  kill "$p" 2>/dev/null; wait "$p" 2>/dev/null
  if [[ -n $got ]]; then echo "tunnel carried: $got"; return 0; fi
  echo "tunnel carried nothing"; return 1
}

# ── one method ────────────────────────────────────────────────────────────
run_method() {
  method=$1
  local root="$work/relay-$method" clones="$work/clones-$method" ak="$work/ak-$method/authorized_keys"
  local conf="$work/sshd-$method.conf" log="$work/sshd-$method.log"
  mkdir -p "$clones"

  if [[ $method == A ]]; then
    section "方式 A · AuthorizedKeysCommand"
    must "$bin" --root "$root" init --binary "$bin" || return
  else
    section "方式 B · init --authorized-keys + sync-keys"
    mkdir -p "$(dirname "$ak")"; chmod 700 "$(dirname "$ak")"
    printf '# hand-written line, must survive\n' >"$ak"; chmod 600 "$ak"
    must "$bin" --root "$root" init --binary "$bin" --authorized-keys "$ak" || return
  fi
  must "$bin" --root "$root" open wutong --name "梧桐里三号院" \
    --host-id chenli --host-name "陈立" --host-key "@$keys/chenli.pub" || return

  if [[ $method == A ]]; then
    # A0: the shipped line, run from a binary that is not root-owned.
    if [[ $(stat -c %u "$bin") != 0 ]]; then
      conf_A "$root" "$bin" >"$conf"; verify_conf "$conf" A
      start_sshd "$conf" "$log"
      expect fail "A0 非 root 属主的程序被 sshd 拒绝（安全检查生效）" "Permission denied" ssh_as chenli "$user@localhost"
      if grep -q "Unsafe AuthorizedKeysCommand" "$log"; then pass "A0 sshd 日志" "Unsafe AuthorizedKeysCommand … bad ownership or modes"
      else fail "A0 sshd 日志里没有 Unsafe AuthorizedKeysCommand" "$(cat "$log")"; fi
      conf_A "$root" "/usr/bin/env $bin" >"$conf"
    else
      conf_A "$root" "$bin" >"$conf"
    fi
    verify_conf "$conf" A
  else
    must "$bin" --root "$root" sync-keys --file "$ak" || return
    expect ok "B0 sync-keys 写出托管块，手写行保留" "# hand-written line" cat "$ak"
    conf_B "$root" "$ak" >"$conf"; verify_conf "$conf" B
  fi
  start_sshd "$conf" "$log"

  local c="$clones"
  expect ok "T1 老板 clone" "" git_as chenli clone -q "$url" "$c/chenli" || return
  git -C "$c/chenli" config user.name chenli; git -C "$c/chenli" config user.email chenli@test
  cat >>"$c/chenli/.teahouse/members.toml" <<EOF

[[member]]
id = "zhou"
name = "周小满"
role = "steward"
key = "$(pubkey zhou)"

[[member]]
id = "wang"
name = "王阿姨"
role = "guest"
key = "$(pubkey wang)"
EOF
  mkdir -p "$c/chenli/problems"; echo "# 停车位" >"$c/chenli/problems/0001-parking.md"
  commit "$c/chenli" $'入座，提出问题 0001\n\nProblem: meta'
  expect ok "T2 老板推送（Problem: meta）" "main -> main" git_as chenli -C "$c/chenli" push origin main
  if [[ $method == B ]]; then
    expect ok "T2 post-receive 把新座位同步进 authorized_keys" "" \
      test "$(grep -c '^restrict,command=' "$ak")" -eq 3
  fi

  for who in zhou wang; do
    must git_as "$who" clone -q "$url" "$c/$who" || return
    git -C "$c/$who" config user.name "$who"; git -C "$c/$who" config user.email "$who@test"
  done
  mkdir -p "$c/wang/works/0001"; echo "错峰共享" >"$c/wang/works/0001/idea.md"
  commit "$c/wang" $'错峰共享\n\nProblem: 0001'
  expect fail "T3 茶客推送 main 被拒" "guests cannot write main" git_as wang -C "$c/wang" push origin main
  expect ok "T4 茶客推送 refs/proposals/wang/x" "refs/proposals/wang/x" \
    git_as wang -C "$c/wang" push origin HEAD:refs/proposals/wang/x
  expect fail "T4b 茶客写别人的提案命名空间被拒" "belongs to zhou" \
    git_as wang -C "$c/wang" push origin HEAD:refs/proposals/zhou/x
  git -C "$c/wang" reset -q --hard origin/main

  mkdir -p "$c/zhou/works/0001"; echo "42 个车位" >"$c/zhou/works/0001/map.md"
  commit "$c/zhou" $'车位分布图\n\nProblem: 0001'
  expect ok "T4c 伙计推送 works/ 到 main" "main -> main" git_as zhou -C "$c/zhou" push origin main

  expect fail "T5 茶馆外的密钥：ssh 被拒" "Permission denied" ssh_as outsider "$user@localhost"
  expect fail "T5 茶馆外的密钥：clone 失败" "" git_as outsider clone -q "$url" "$c/outsider"

  expect ok "T6 直接登录：只打印你的茶馆" "你的茶馆" ssh_as chenli "$user@localhost" </dev/null
  expect ok "T6 ssh -T 登录" "老板 (host)" ssh_as chenli -T "$user@localhost" </dev/null
  if command -v script >/dev/null; then
    expect ok "T6′ 真实终端登录（script 提供 PTY）" "你的茶馆" \
      script -qec "ssh $(ssh_opts chenli) $user@localhost" /dev/null </dev/null
  else
    pass "T6′ 跳过（没有 script 命令）"
  fi
  expect fail "T7 ssh … bash 被拒" "only git fetch/clone/push" ssh_as chenli "$user@localhost" bash -c id
  expect fail "T7b 路径穿越被拒" "no such teahouse" ssh_as chenli "$user@localhost" "git-upload-pack '/../etc'"

  expect fail "T8 -L 端口转发不通" "carried nothing" probe_L chenli
  if grep -q "refused local port forward" "$log"; then pass "T8 sshd 日志" "refused local port forward"
  else fail "T8 sshd 日志里没有 refused local port forward" "$(tail -n 20 "$log")"; fi
  expect fail "T8b -W 被拒" "administratively prohibited" \
    timeout 10 ssh $(ssh_opts chenli) -W 127.0.0.1:"$port" "$user@localhost" </dev/null
  expect fail "T8c -R 被拒" "remote port forwarding failed" \
    timeout 10 ssh $(ssh_opts chenli) -o ExitOnForwardFailure=yes -N \
      -R "127.0.0.1:$((port + 78)):127.0.0.1:$port" "$user@localhost"

  # v0.1.1: written discussion under talk/.
  must git_as wang -C "$c/wang" pull -q --ff-only || return
  local post="talk/0001/$(stamp)-wang.md"
  mkdir -p "$c/wang/talk/0001"; echo "晚上可以错峰停车。" >"$c/wang/$post"
  commit "$c/wang" $'留言：错峰停车\n\nProblem: 0001'
  expect ok "T12 茶客在 main 上新增自己的留言" "main -> main" git_as wang -C "$c/wang" push origin main
  echo "我同意。" >"$c/wang/talk/0001/$(stamp)-zhou.md"
  commit "$c/wang" $'冒名留言\n\nProblem: 0001'
  expect fail "T12b 冒名（伙计的名字）留言被拒" "is signed by zhou, not by you" git_as wang -C "$c/wang" push origin main
  git -C "$c/wang" reset -q --hard origin/main
  echo "改一下。" >>"$c/wang/$post"
  commit "$c/wang" $'改留言\n\nProblem: 0001'
  expect fail "T12c 修改已有留言被拒（只能新增）" "posts are never edited" git_as wang -C "$c/wang" push origin main
  git -C "$c/wang" reset -q --hard origin/main
  mkdir -p "$c/wang/talk/board/no-such-notice"; echo "?" >"$c/wang/talk/board/no-such-notice/$(stamp)-wang.md"
  commit "$c/wang" $'回复\n\nProblem: board'
  expect fail "T12d 回复不存在的公告被拒" "which does not exist" git_as wang -C "$c/wang" push origin main
  git -C "$c/wang" reset -q --hard origin/main
  head -c 40000 /dev/zero | tr '\0' 'a' >"$c/wang/talk/0001/$(stamp)-wang.md"
  commit "$c/wang" $'超长留言\n\nProblem: 0001'
  expect fail "T12e 超过 32 KiB 的留言被拒" "limited to 32 KiB" git_as wang -C "$c/wang" push origin main
  git -C "$c/wang" reset -q --hard origin/main

  expect ok "T10 scp 风格地址 user@host:wutong" "" \
    env GIT_SSH_COMMAND="ssh $(ssh_opts chenli)" git clone -q "$user@localhost:wutong" "$c/scp"

  must git_as chenli -C "$c/chenli" pull -q --ff-only || return
  python3 - "$c/chenli/.teahouse/members.toml" <<'PY' || { fail "setup: edit members.toml"; return; }
import re, sys
p = sys.argv[1]; s = open(p, encoding="utf-8").read()
open(p, "w", encoding="utf-8").write(re.sub(r'\n\[\[member\]\]\nid = "wang"\n(?:[^\[\n][^\n]*\n?)*', "\n", s))
PY
  commit "$c/chenli" $'请王阿姨离座\n\nProblem: meta'
  expect ok "T9 老板移除茶客" "main -> main" git_as chenli -C "$c/chenli" push origin main
  expect fail "T9 被移除的密钥立即失效" "" git_as wang -C "$c/wang" fetch origin
  if [[ $method == B ]]; then
    expect ok "T9 authorized_keys 同步为 2 把，手写行仍在" "# hand-written line" \
      bash -c "test \$(grep -c '^restrict,command=' '$ak') -eq 2 && cat '$ak'"
  fi

  # E: the client engine over real ssh: libssh2 inside the engine, the
  # device key from memory, no ssh or git program involved.
  local eng="$work/engine-$method" ekey
  ekey="$("$client" --root "$eng" key)" || { fail "setup: teahouse-client key"; return; }
  must "$bin" --root "$root" open kehu --name "引擎测试" --host-id eng --host-name "引擎" --host-key "$ekey" || return
  [[ $method == B ]] && { must "$bin" --root "$root" sync-keys --file "$ak" || return; }
  expect ok "E1 引擎 join（libssh2，设备钥匙在内存里）" '"ok":true' \
    "$client" --root "$eng" call join "{\"relay\":\"$user@127.0.0.1:$port\",\"house\":\"kehu\"}"
  expect ok "E1 Relay 的主机钥匙首次记下" "127.0.0.1" cat "$eng/known_hosts"
  expect ok "E2 引擎写入、签名、推送" '"commit":"pushed"' \
    "$client" --root "$eng" call openProblem '{"house":"kehu","title":"经真实 sshd 推送"}'
  expect ok "E2 Relay 上收到这次提交" "经真实 sshd 推送" git -C "$root/houses/kehu.git" log -1 --format=%s main
  expect ok "E3 没有座位的茶馆进不去" '"code":"not_a_member"' \
    bash -c "'$client' --root '$eng' call join '{\"relay\":\"$user@127.0.0.1:$port\",\"house\":\"wutong\"}'; true"
  sed -i 's/ [0-9a-f]*$/ 00/' "$eng/known_hosts"
  expect ok "E4 主机钥匙变了：拒绝连接" "different host key" \
    "$client" --root "$eng" call sync '{"house":"kehu"}'

  local audit="$root/log/audit.log" ev events="shell.info shell.read shell.write shell.refused push.allowed push.denied push.applied seats.changed"
  [[ $method == B ]] && events+=" keys.synced"
  local missing=()
  for ev in $events; do grep -q "\"event\":\"$ev\"" "$audit" || missing+=("$ev"); done
  if [[ ${#missing[@]} -eq 0 ]]; then pass "审计日志包含 $events"
  else fail "审计日志缺少：${missing[*]}" "$(cat "$audit")"; fi

  if [[ $method == B ]]; then
    # T11: the key line's `restrict` holds even with no hardening in sshd_config.
    { global_conf; echo "AuthorizedKeysFile $ak"; } >"$work/sshd-B-bare.conf"
    start_sshd "$work/sshd-B-bare.conf" "$work/sshd-B-bare.log"
    expect fail "T11 无加固配置：bash 仍被拒（restrict + command=）" "only git fetch/clone/push" \
      ssh_as chenli "$user@localhost" bash
    expect fail "T11 无加固配置：-W 仍被拒" "administratively prohibited" \
      timeout 10 ssh $(ssh_opts chenli) -W 127.0.0.1:"$port" "$user@localhost" </dev/null
    expect fail "T11 无加固配置：-R 仍被拒" "remote port forwarding failed" \
      timeout 10 ssh $(ssh_opts chenli) -o ExitOnForwardFailure=yes -N \
        -R "127.0.0.1:$((port + 78)):127.0.0.1:$port" "$user@localhost"
  fi
  stop_sshd
}

echo "$("$sshd" -V 2>&1 | head -n1) · $sshd · port $port · $work"
for m in $methods; do
  case $m in A | B) run_method "$m" ;; *) die "unknown method $m (use A or B)" ;; esac
done

section "结果 · $passed passed, $failed failed"
for f in "${failures[@]}"; do printf '  %s✗ %s%s\n' "$red" "$f" "$off"; done
[[ $failed -eq 0 ]]
