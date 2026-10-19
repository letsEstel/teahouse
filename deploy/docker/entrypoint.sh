#!/bin/sh
# Container start: make sure the volume is initialised, then run sshd in the
# foreground. Safe on every start (init is idempotent).
set -eu
ROOT=/var/lib/teahouse
chown teahouse:teahouse "$ROOT"
install -d -m 0700 -o root -g root "$ROOT/.sshd"
[ -f "$ROOT/.sshd/ssh_host_ed25519_key" ] || ssh-keygen -q -t ed25519 -N '' -f "$ROOT/.sshd/ssh_host_ed25519_key"
runuser -u teahouse -- /usr/local/bin/teahouse-relay --root "$ROOT" init --binary /usr/local/bin/teahouse-relay >/dev/null
mkdir -p /run/sshd
/usr/sbin/sshd -t
echo "teahouse relay ready; host key: $(ssh-keygen -lf "$ROOT/.sshd/ssh_host_ed25519_key.pub")"
exec /usr/sbin/sshd -D -e
