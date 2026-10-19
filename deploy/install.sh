#!/bin/sh
# 茶馆 Relay 一键安装 · Teahouse relay, one-command install
#
# On a fresh Debian 12 / Ubuntu 22.04+ server, as root:
#
#   sh install.sh --binary ./teahouse-relay                 # use a binary you copied over
#   sh install.sh --from-source                             # build it here (installs Rust)
#   sh install.sh --binary ./teahouse-relay \
#       --open wutong --name "梧桐里三号院" --host-name "陈立" --host-key @host.pub
#                                                           # ...and open the first teahouse
#
# What it does (idempotent — safe to run again, e.g. to upgrade the binary):
#   1. installs openssh-server, git and ssh-keygen if missing
#   2. installs the relay binary at /usr/local/bin/teahouse-relay (root-owned,
#      which sshd requires for AuthorizedKeysCommand)
#   3. creates the unprivileged system user `teahouse` and /var/lib/teahouse
#   4. runs `teahouse-relay init` (hooks, config, the relay's signing key)
#   5. installs the sshd drop-in (method A: keys looked up live, which is also
#      the only method that can redeem invitations) and reloads sshd
#   6. installs and starts the daily integrity check (systemd timer)
#   7. optionally opens the first teahouse
#
# It never touches other users, never changes sshd settings outside
# `Match User teahouse`, and never opens ports other than the one sshd
# already listens on.
set -eu

ROOT=/var/lib/teahouse
BIN=/usr/local/bin/teahouse-relay
USER_NAME=teahouse
HERE=$(cd "$(dirname "$0")" && pwd)

binary=""
from_source=0
open_id=""
open_name=""
open_sign=""
host_id=""
host_name=""
host_key=""

say()  { printf '\033[1m▌ %s\033[0m\n' "$*"; }
ok()   { printf '  \033[32m✓\033[0m %s\n' "$*"; }
die()  { printf '\033[31m✗ %s\033[0m\n' "$*" >&2; exit 1; }

# Run a command as the relay's system user (runuser ships with util-linux).
as_relay() { runuser -u "$USER_NAME" -- "$@"; }

usage() {
  sed -n '2,24p' "$0" | sed 's/^# \{0,1\}//'
  exit "${1:-0}"
}

while [ $# -gt 0 ]; do
  case "$1" in
    --binary)      binary="$2"; shift 2 ;;
    --from-source) from_source=1; shift ;;
    --open)        open_id="$2"; shift 2 ;;
    --name)        open_name="$2"; shift 2 ;;
    --signboard)   open_sign="$2"; shift 2 ;;
    --host-id)     host_id="$2"; shift 2 ;;
    --host-name)   host_name="$2"; shift 2 ;;
    --host-key)    host_key="$2"; shift 2 ;;
    -h|--help)     usage 0 ;;
    *)             printf 'unknown option %s\n\n' "$1" >&2; usage 1 ;;
  esac
done

[ "$(id -u)" -eq 0 ] || die "请用 root 运行（sudo sh install.sh …） · run as root"
[ -r /etc/os-release ] && . /etc/os-release
case "${ID:-}${ID_LIKE:-}" in
  *debian*|*ubuntu*) ;;
  *) die "目前只支持 Debian / Ubuntu（检测到 ${PRETTY_NAME:-unknown}）" ;;
esac
if [ -z "$binary" ] && [ "$from_source" -eq 0 ]; then
  if [ -x "$BIN" ]; then
    binary="$BIN"   # re-run: keep the installed binary
  else
    die "需要 --binary <路径> 或 --from-source · need --binary or --from-source"
  fi
fi
if [ -n "$open_id" ]; then
  [ -n "$open_name" ] && [ -n "$host_name" ] && [ -n "$host_key" ] \
    || die "--open 需要同时给出 --name、--host-name、--host-key"
fi

say "1/7 系统依赖 · packages"
need=""
command -v sshd >/dev/null 2>&1 || [ -x /usr/sbin/sshd ] || need="$need openssh-server"
command -v git >/dev/null 2>&1 || need="$need git"
command -v ssh-keygen >/dev/null 2>&1 || need="$need openssh-client"
if [ "$from_source" -eq 1 ]; then
  command -v cc >/dev/null 2>&1 || need="$need build-essential"
  command -v curl >/dev/null 2>&1 || need="$need curl ca-certificates"
fi
if [ -n "$need" ]; then
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -qq
  # shellcheck disable=SC2086
  apt-get install -y -qq --no-install-recommends $need >/dev/null
  ok "installed:$need"
else
  ok "already present"
fi

say "2/7 Relay 程序 · binary"
if [ "$from_source" -eq 1 ]; then
  [ -f "$HERE/../Cargo.toml" ] || die "--from-source 需要在仓库里运行（deploy/install.sh）"
  if ! command -v cargo >/dev/null 2>&1; then
    curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal >/dev/null
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
  fi
  (cd "$HERE/.." && cargo build --release -p teahouse-relay --locked)
  binary="$HERE/../target/release/teahouse-relay"
fi
[ -f "$binary" ] || die "找不到程序 $binary"
if [ "$(readlink -f "$binary")" != "$BIN" ]; then
  install -o root -g root -m 0755 "$binary" "$BIN.new"
  "$BIN.new" version >/dev/null || die "$binary 无法运行（架构不对？）"
  mv -f "$BIN.new" "$BIN"
fi
ok "$("$BIN" version) → $BIN"

say "3/7 系统用户 · user $USER_NAME"
if ! id "$USER_NAME" >/dev/null 2>&1; then
  useradd --system --home-dir "$ROOT" --create-home --shell /bin/sh "$USER_NAME"
  ok "created"
else
  ok "exists"
fi
# sshd refuses an account whose password is locked with `!` under some PAM
# setups; `*` means "no password, not locked".
usermod -p '*' "$USER_NAME" 2>/dev/null || true
install -d -o "$USER_NAME" -g "$USER_NAME" -m 0750 "$ROOT"

say "4/7 初始化 · init"
as_relay "$BIN" --root "$ROOT" init --binary "$BIN" >/dev/null
ok "$ROOT"

say "5/7 OpenSSH"
install -d -m 0755 /etc/ssh/sshd_config.d
if ! grep -qsE '^[[:space:]]*Include[[:space:]]+/etc/ssh/sshd_config\.d/' /etc/ssh/sshd_config; then
  printf '\nInclude /etc/ssh/sshd_config.d/*.conf\n' >> /etc/ssh/sshd_config
fi
conf_src="$HERE/sshd/teahouse.conf"
if [ -f "$conf_src" ]; then
  install -m 0644 "$conf_src" /etc/ssh/sshd_config.d/teahouse.conf
else
  die "缺少 $conf_src（请在仓库的 deploy/ 目录里运行）"
fi
[ -f /etc/ssh/ssh_host_ed25519_key ] || ssh-keygen -A >/dev/null
sshd_bin=$(command -v sshd || echo /usr/sbin/sshd)
# Ubuntu 24.04 starts sshd by socket activation, so /run/sshd may not exist
# yet; `sshd -t` refuses to run without it.
install -d -m 0755 /run/sshd
"$sshd_bin" -t || die "sshd 配置检查失败 · sshd -t failed"
if command -v systemctl >/dev/null 2>&1 && [ -d /run/systemd/system ]; then
  systemctl enable --now ssh >/dev/null 2>&1 || systemctl enable --now sshd >/dev/null 2>&1 || true
  systemctl reload ssh 2>/dev/null || systemctl reload sshd 2>/dev/null || systemctl restart ssh
  ok "sshd reloaded"
else
  ok "sshd configured (no systemd here — start it yourself: $sshd_bin)"
fi

say "6/7 每日检查 · daily check"
if command -v systemctl >/dev/null 2>&1 && [ -d /run/systemd/system ]; then
  install -m 0644 "$HERE/systemd/teahouse-check.service" "$HERE/systemd/teahouse-check.timer" /etc/systemd/system/
  systemctl daemon-reload
  systemctl enable --now teahouse-check.timer >/dev/null 2>&1
  ok "teahouse-check.timer"
else
  ok "skipped (no systemd)"
fi

say "7/7 茶馆 · teahouse"
if [ -n "$open_id" ]; then
  if as_relay "$BIN" --root "$ROOT" members "$open_id" >/dev/null 2>&1; then
    ok "$open_id already open"
  else
    [ -n "$host_id" ] || host_id=$(printf '%s' "$host_name" | tr 'A-Z' 'a-z' | tr -cs 'a-z0-9' '-' | sed 's/^-*//; s/-*$//')
    [ -n "$host_id" ] || host_id=host
    case "$host_key" in
      @*) key_arg=$(head -n 1 "${host_key#@}") || die "读不到 ${host_key#@}" ;;
      *)  key_arg="$host_key" ;;
    esac
    set -- --root "$ROOT" open "$open_id" --name "$open_name" --host-id "$host_id" --host-name "$host_name" --host-key "$key_arg"
    [ -n "$open_sign" ] && set -- "$@" --signboard "$open_sign"
    as_relay "$BIN" "$@"
  fi
else
  ok "none opened (use --open …, or later: sudo -u teahouse $BIN --root $ROOT open …)"
fi

addr=$(hostname -I 2>/dev/null | awk '{print $1}')
port=$(sshd -T 2>/dev/null | awk '/^port /{print $2; exit}')
printf '\n\033[1m茶馆 Relay 装好了 · The relay is ready.\033[0m\n'
printf '  地址 · address:  teahouse@%s%s\n' "${addr:-<this-server>}" "$( [ "${port:-22}" != 22 ] && printf ':%s' "$port" )"
printf '  检查 · check:    sudo -u teahouse %s --root %s check\n' "$BIN" "$ROOT"
printf '  日志 · audit:    %s/log/audit.log\n' "$ROOT"
