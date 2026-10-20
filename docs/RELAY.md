# 茶馆 Relay · Teahouse Relay

The relay is the only way into a teahouse. It hosts the root git repository
of every teahouse and enforces the seating chart: who may read, who may write
what, and which history is legitimate. This document is the reference for how
it works, why it is built this way, and how to run it.

---

## 1. 设计原则 · Design principles

1. **不造轮子，尤其不造加密的轮子。** Authentication is OpenSSH; transport
   is git's own `upload-pack` / `receive-pack`. Both have decades of hostile
   traffic behind them. The relay adds exactly one thing: *authorisation by
   role*. This is the architecture gitolite has run on for fifteen years.
2. **授权只来自当前的 main。** The seating chart in force is
   `.teahouse/members.toml` on the relay's current `main`. A push is judged
   against the chart *before* it, never by the chart it carries, so no push
   can promote the person making it.
3. **失败即关闭。** Anything unexpected — a missing environment variable, an
   unreadable chart, an unknown ref, a malformed file — is a refusal, not a
   pass. The pre-receive hook refuses pushes that did not come through the
   relay shell.
4. **全有或全无。** git's pre-receive hook sees all ref updates of a push at
   once; if any of them is refused, none is applied.
5. **历史只增不改。** Nobody — not even the host — can force-push or delete
   `main`. Server reflogs are kept forever (`core.logAllRefUpdates=always`,
   `gc.reflogExpire=never`).
6. **零依赖。** `teahouse-core` and `teahouse-relay` use only the Rust
   standard library. Every line that decides who gets in is in this
   repository and can be read in an afternoon.

## 2. 架构 · Architecture

```
member's git ──ssh──▶ sshd ──(publickey ok)──▶ teahouse-relay shell <key>
                       │                              │ checks: valid request? seat in this house?
                       │ AuthorizedKeysCommand        ▼
                       └─▶ teahouse-relay authorized-keys --key %k
                                                exec git upload-pack | receive-pack
                                                              │ (push only)
                                                              ▼
                                               hooks/pre-receive → teahouse-relay hook pre-receive
                                                   judges every ref and every new commit
                                               hooks/post-receive → audit, refresh keys
```

There is **no long-running relay daemon**. sshd is the daemon; the relay is a
short-lived process per connection. Nothing to leak memory, nothing to
restart, nothing listening on a new port.

### On-disk layout

```
/var/lib/teahouse/
  relay.conf            binary = …, optional authorized_keys = …
  houses/<id>.git       one bare repository per teahouse
  hooks/                pre-receive, post-receive (shared via core.hooksPath)
  log/audit.log         append-only JSON lines
  blobs/<id>/<sha256>   attachments of each teahouse (CLIENT-API.md §1.6)
  invites/<id>/         pending invitations, by the SHA-256 of the code only
  relay_signing_key     the relay's own ed25519 key (made by `init`), for the
                        seat commits it writes when an invitation is redeemed
  tmp/                  scratch (staging new houses, private index files)
```

A new teahouse is prepared under `tmp/` and moved into `houses/` with one
`rename`, so a teahouse is either absent or complete.

## 3. 茶馆的法 · The laws the relay enforces

| # | Law | Applies to |
|---|-----|-----------|
| 1 | Only seated keys can connect to a teahouse; unknown teahouses and teahouses without your seat look identical. | read, write |
| 2 | Only `refs/heads/main` and `refs/proposals/<member-id>/<name>` exist. Tags and other branches are refused. | all |
| 3 | `main` is never created, deleted or rewritten through a push (fast-forward only, even for the host). | all |
| 4 | Guests (茶客) land on `main` only by **adding** their own discussion posts (`talk/…/<stamp>-<their-id>.md`): no edits, no deletions, no merges, nothing else. Everything else they push as a proposal to their own namespace. | guest |
| 5 | Anyone may create, rework (force) or delete proposals in **their own** namespace; only the host may delete someone else's. | all |
| 6 | The steward (伙计) may change only `problems/`, `works/`, `board/` and `talk/` (moderation) on `main`. Checked **per commit**, including what a merge resolution itself introduces (`git diff-tree -c`). | steward |
| 7 | Every commit carries exactly one `Problem:` trailer: a 4-digit id whose file `problems/<id>-*.md` exists in that commit, `board` (may touch only `board/` and `talk/board/`), or `meta`. A post must sit under the topic its commit faces, and that problem (`problems/<id>-*.md`) or notice (`board/<notice>.md`) must exist in the same commit. | all |
| 8 | `Problem: meta` may land on `main` only from the host. | steward |
| 9 | `.teahouse/` contains only `members.toml`; any commit that changes it must leave a valid chart (1–15 seats, exactly one host, at most one steward, unique ids and keys, ed25519 only). | all |
| 10 | Paths that would turn into `.teahouse/` on Windows or macOS checkouts (`.TeaHouse/`, `.teahouse./`, `TEAHOU~1`) are refused. | all |
| 11 | At most 64 ref updates and 5 000 new commits per push; pack size ≤ 200 MiB; objects are fsck-checked on receive. | all |
| 12 | Posts and notices (`talk/`, `board/`) are regular UTF-8 text files of at most 32 KiB, without control characters except tab and line breaks. A post's stamp is a real UTC time no more than 10 minutes ahead of the relay's clock; posts are shown in the order they entered `main`, never by stamp. | all |
| 13 | Every pushed commit is signed (git ssh signature) by the device key of its author `<member-id>@teahouse`, and that key appears in a version of this teahouse's `members.toml` on `main`. Who may push is still decided by the current chart. Commits the relay writes itself (opening, succession, redeemed invitations) do not arrive by push; they are signed with the relay's own key and carry committer `Teahouse Relay`. | all |
| 14 | On `main`, each problem number has at most one `problems/<id>-*.md`. (Proposals may collide; the host's engine renumbers on adoption.) | all |
| 15 | Attachments live outside git: every `*.ref.md` is well-formed (`blob: sha256:…`, `size` ≤ 100 MiB, `type`), and unless marked `purged`, its blob is already in the attachment store with that size. Binary files over 1 MiB are refused in git. | all |
| 16 | Invitations (`docs/INVITE.md` §2): only the host creates them; one-time, 1–30 days, stored hashed; redeeming needs a free seat (≤ 15), a free steward seat if the invitation is for a steward, and a key not yet in the house; rate-limited and audited. The relay writes the seat commit (author: the host). | host / anyone with a code |
| 17 | `kind = "ai"` marks a member as an AI; it is a label, not a rank. Any role may be held by an AI — the founder decides whom to seat. `agent` may appear only on AI members. | all |

Laws 6–9 and 12 apply to every commit a push would add to `main` — including commits
that came from someone else's proposal. A steward therefore cannot land a
guest's proposal that touches the seating chart; only the host can.

### Discussion layout (说明书 第十四节)

```
board/<notice>.md                                  notices (host, steward)
talk/<problem-id>/<YYYYMMDDTHHMMSSZ>-<member>.md   discussion under a problem
talk/board/<notice>/<YYYYMMDDTHHMMSSZ>-<member>.md replies to a notice
```

One post is one file with one author; it is written, not spoken, so it is
never edited afterwards. The steward and host may remove posts (moderation).

What the relay checks for every post or notice that a commit adds or
changes, on `main` and on proposals alike:

* **Text, at most 32 KiB** (32 768 bytes, about ten thousand Chinese
  characters). The file must be a regular file (not a symlink), valid UTF-8,
  and free of control characters other than tab, line feed and carriage
  return. Pictures and attachments go in `works/`. The same limit applies to
  every file under `board/`.
* **It answers something real.** `talk/<id>/…` needs `problems/<id>-*.md`,
  and `talk/board/<notice>/…` needs `board/<notice>.md`, both in that same
  commit. Taking a notice down later is moderation; its replies may stay.
* **The stamp is a real UTC date and time, at most 10 minutes ahead of the
  relay's clock** at the time of the push. Earlier stamps are fine: a post
  written offline is pushed later.

#### 显示顺序 · Display order

**Posts are shown in the order they entered the teahouse's `main`, never
sorted by their stamp.** The stamp only names the file and keeps two posts
apart, so changing it gains nothing. Every member's clone can compute this
order from history alone:

```sh
# For each step of main, oldest first: the posts that step added
# (compared with the step before it, i.e. the first parent).
git rev-list --first-parent --reverse main | while read c; do
  if git rev-parse -q --verify "$c^" >/dev/null; then
    git diff-tree -r --no-renames --diff-filter=A --name-only "$c^" "$c" -- talk/
  else
    git diff-tree -r --root --no-commit-id --no-renames --diff-filter=A --name-only "$c" -- talk/
  fi
done
```

Compare against the first parent explicitly. A plain `git diff-tree <merge>`
prints nothing for a merge and would lose every post that came in through
one. A post that arrives through a merged proposal enters when the merge
lands, not when the proposal was written. Within one step, order by file
name. Clients name new posts with `teahouse_core::rules::format_stamp(now)`.

### Signatures, numbers and attachments (laws 13–15)

These checks are enforced since step 3 of CLIENT-API.md §7. Their wording for
the table above is proposed to the reviewer and the user (CLIENT-API.md §8.2);
until it is settled, this section describes them.

* **Every pushed commit is signed by the member it names.** The author must
  be `<member-id>@teahouse`, and the commit must carry git's ssh signature
  (`gpg.format=ssh`) made with a key that has sat on this teahouse's chart.
  git checks it (`%G?`/`%GS` with an allowed-signers file built from every
  version of `members.toml` on `main`, verified by ssh-keygen). Who may
  *push* is still decided by the current chart. Commits the relay writes
  itself (opening, succession, redeemed invitations) are not pushed; the
  seat commits of invitations are signed with `relay_signing_key` and
  committed by `Teahouse Relay <relay@teahouse>`.
* **One problem file per number on `main`.** Two `problems/0007-*.md` on
  `main` are refused; on a proposal they are allowed, and the host's client
  renumbers when adopting (`Renumbered:`).
* **AI members are guests (法第 17 条, docs/MCP.md).** A member may carry
  `kind = "ai"` and, then only, `agent = "…"`. An AI member can only be a
  guest; the host and the steward are people. `teahouse-core` refuses any
  chart that breaks this, so every pushed chart, every seat commit of an
  invitation and every succession is held to it.
* **Attachments stay out of git.** A `*.ref.md` must name `blob:
  sha256:<64 hex>`, `size` (≤ 100 MiB) and `type`, and, unless `purged`, the
  attachment must already be in `blobs/<id>/` with that size. Any other file
  over 1 MiB that is binary (a NUL in its first 8000 bytes) is refused.

## 3a. 附件与邀请 · Attachments and invitations

Besides git, the forced command serves a fixed list of commands. Arguments
are words; single quotes quote (`--name '王阿姨'`); nothing is expanded.

| Command | Who | What |
|---|---|---|
| `blob-has <id> <sha256>` | seated | prints `yes` / `no` |
| `blob-put <id> <sha256> <size>` | seated | reads exactly `size` bytes, checks the hash, stores it (100 MiB each, `blob_quota_mib` per teahouse, default 2048) |
| `blob-get <id> <sha256>` | seated | writes the attachment |
| `blob-purge <id> <sha256>` | host | deletes it for good |
| `invite <id> [--role guest\|steward] [--name N] [--days 1–30] [--kind ai [--agent A]]` | host | prints a one-time code `TH-XXXX-XXXX-XXXX-XXXX` (80 bits, default 7 days); `--kind ai` seats an AI, as a guest only |
| `uninvite <id> <first four>` | host | revokes it |
| `redeem <code> --name N [--agent A]` | anyone, incl. a key with no seat | seats the key; prints `house <id>` and `member <id>`; `--agent` only for an AI's invitation |

**Founding invitations** open a teahouse instead of seating someone in one.
The operator runs `teahouse-relay found <id> --name N [--signboard S]
[--days D] [--address HOST[:PORT]]` (as the relay's user; `install.sh
--found` does it) and passes on the printed `TH-…@address`. Redeeming it
creates teahouse `<id>` exactly as `open` would, with the redeemer's key and
name as the host (the opening commit carries `Invite:`); until then nothing
exists. It fails if the teahouse exists by then, and of two founding
invitations for one id only the first redeemed counts.

**A key with no seat anywhere** gets, from `AuthorizedKeysCommand`, a line
forcing `teahouse-relay redeem-shell <key>`: the same entry, which accepts
`redeem` and nothing else (INVITE.md rule 6). With the authorized_keys-file
way, unknown keys cannot connect at all, so invitations need
`AuthorizedKeysCommand`.

Redeeming is all or nothing: the invitation must be valid, unused and
unexpired; the teahouse must have a free seat (15) and, for a steward
invitation, no steward; the key must not sit there yet. The member id comes
from the name's ASCII letters (`Wang Li` → `wang-li`) or is `m1`, `m2`, …
The seat commit is checked by the same code as a push (with the host as the
pusher) and lands with a compare-and-swap on `main`; a push racing it is
refused by git's ref lock and simply retried by the client. At most 5
failures per key per hour and 30 attempts per minute in all.

## 4. 继承 · Succession

Every member holds the full history, so a relay can be shut down but a
teahouse cannot be killed. The steward is the designated heir:

```sh
teahouse-relay --root /var/lib/teahouse adopt wutong --from ~/clones/wutong --key @~/.ssh/id_ed25519.pub
```

`adopt` verifies the key is the steward on that history, imports the history
**unchanged** (every commit keeps its hash), and adds one succession commit
that makes the steward host and gives the previous host a guest seat
(`Problem: meta`, `Succession-From: <old host>`). v0.2 will add verifiable
succession notices so members' clients follow the heir automatically.

## 5. 部署 · Deployment (Debian/Ubuntu)

```sh
# 1. Build and install the binary (owned by root; sshd requires that for AuthorizedKeysCommand).
cargo build --release
sudo install -d -m 0755 /usr/lib/teahouse && sudo install -o root -g root -m 0755 target/release/teahouse-relay /usr/lib/teahouse/ && sudo ln -sf /usr/lib/teahouse/teahouse-relay /usr/local/bin/

# 2. A dedicated, unprivileged system user and data directory.
sudo useradd --system --home-dir /var/lib/teahouse --create-home --shell /bin/sh teahouse
sudo -u teahouse teahouse-relay --root /var/lib/teahouse init --binary /usr/lib/teahouse/teahouse-relay

# 3. OpenSSH: look member keys up live from the seating charts.
sudo install -m 0644 deploy/sshd/teahouse.conf /etc/ssh/sshd_config.d/teahouse.conf
sudo sshd -t && sudo systemctl reload ssh

# 4. Daily integrity check.
sudo install -m 0644 deploy/systemd/teahouse-check.{service,timer} /etc/systemd/system/
sudo systemctl daemon-reload && sudo systemctl enable --now teahouse-check.timer

# 5. Open the first teahouse.
sudo -u teahouse teahouse-relay --root /var/lib/teahouse open wutong \
  --name "梧桐里三号院" --signboard "车位、垃圾、楼道灯，咱们自己管。" \
  --host-id chenli --host-name "陈立" --host-key "ssh-ed25519 AAAA… chenli@laptop"
```

Members normally use the teahouse client. Plain git works too, with every
commit signed by the member's own key:

```sh
git clone teahouse@relay.example.org:wutong && cd wutong
git config user.email <member-id>@teahouse
git config gpg.format ssh && git config user.signingkey ~/.ssh/id_ed25519
git config commit.gpgsign true
ssh -T teahouse@relay.example.org       # prints the teahouses you sit in
```

The relay checks signatures with `ssh-keygen`, which must be installed (it
comes with the OpenSSH client).

Tell members to use `ssh -T`. The relay never hands out a terminal, so a
plain `ssh` from an interactive shell first prints `PTY allocation request
failed on channel 0`. That line is harmless, but it looks like an error.
`ssh -tt` insists on a terminal and exits without printing anything.

If you cannot use `AuthorizedKeysCommand`, run `init --authorized-keys
/var/lib/teahouse/.ssh/authorized_keys` instead: `post-receive` then rewrites
the managed block of that file whenever a seating chart changes, and
`sync-keys --file …` does it by hand. Lines outside the managed block are
kept. Both ways are covered by `scripts/sshd-test.sh` (§8).

sshd runs an `AuthorizedKeysCommand` only if the program and every directory
above it are owned by root and writable by no one else; otherwise it logs
`Unsafe AuthorizedKeysCommand "…": bad ownership or modes` and refuses every
key. Some server images ship a group-writable `/usr/local/bin`, so the binary lives in `/usr/lib/teahouse/` (root-only all the way up) and is only *linked* into `/usr/local/bin`; `deploy/install.sh` checks the whole chain before touching sshd; a
binary under someone's home directory does not.

## 6. 运维 · Operations

| Task | Command |
|------|---------|
| List teahouses | `teahouse-relay list` |
| Seats of one teahouse | `teahouse-relay members <id>` |
| Integrity check (fsck + charts + config) | `teahouse-relay check [<id>]` |
| After moving the binary or root | `teahouse-relay init --binary <path>` (rewrites hooks, re-configures every house) |
| Who did what | `log/audit.log` (see below) and sshd's log (`journalctl -u ssh`) |
| Ref history on the server | `git --git-dir houses/<id>.git reflog main` |
| Backup | Any member's clone is a full backup. On the relay, back up `houses/` with `git bundle` or file-level snapshots. |

### 审计日志 · Audit log

`log/audit.log` is append-only JSON, one object per line. Every line has
`ts` (UTC), `event` and `pid`; the other fields depend on the event.

| Event | When | Fields |
|-------|------|--------|
| `shell.info` | A member logged in without a command (`ssh -T …`) | `key`, `houses` (comma-separated; empty if none) |
| `shell.read` | Fetch or clone let through to `git upload-pack` | `house`, `member`, `key`, `role` |
| `shell.write` | Push let through to `git receive-pack` (the hook judges it next) | `house`, `member`, `key`, `role` |
| `shell.denied` | A git request for a teahouse that does not exist, or where the key has no seat | `house`, `key`, `reason` |
| `shell.refused` | Anything that is not a git request: `bash`, `scp`, a malformed path… **Watch this one.** | `key`, `command`, `reason` |
| `push.allowed` | pre-receive accepted every ref update | `house`, `member`, `key`, `role`, `refs` |
| `push.denied` | pre-receive refused the push (all of it), or the pusher's seat was removed meanwhile | `house`, `member`, `key`, `role`, `refs`, `reasons` / `reason` |
| `push.applied` | post-receive: the refs have moved | `house`, `refs` |
| `seats.changed` | A push changed `members.toml` on `main` | `house` |
| `keys.synced` | `sync-keys` or post-receive rewrote the authorized_keys file | `file`, `keys` (count) |
| `house.opened` | `open` | `house`, `member` (host), `commit` |
| `house.adopted` | `adopt` (succession) | `house`, `member` (new host), `from`, `previous_host`, `commit` |
| `relay.init` | `init` | `binary` |
| `invite.created` | The host created an invitation | `house`, `member`, `invite` (first four), `role`, `days` |
| `invite.redeemed` | An invitation seated a key | `house`, `member` (new), `key`, `invite` |
| `invite.failed` | A redemption failed (bad code, expired, full, rate limited…) | `key`, `reason` |
| `invite.revoked` | The host revoked an invitation | `house`, `member`, `invite` |
| `blob.put` | An attachment was uploaded | `house`, `member`, `blob`, `size` |
| `blob.purged` | The host removed an attachment | `house`, `member`, `blob`, `existed` |

**What is *not* in the audit log.** sshd turns some requests away before
the relay ever runs. Those appear only in sshd's log:

| Attempt | sshd log line |
|---------|---------------|
| A key with no seat on any teahouse (authorized_keys file only; with `AuthorizedKeysCommand` it reaches `redeem-shell`, and anything but `redeem` is logged as `shell.refused`) | `Failed publickey for teahouse from … ssh2: ED25519 SHA256:…` |
| `-L` / `-W` port forwarding | `refused local port forward: originator …, target …` |
| `-R` remote forwarding | nothing at `LogLevel VERBOSE`. The client sees `remote port forwarding failed`. |

Set `LogLevel VERBOSE` in sshd for the key fingerprint on every accepted or
failed login. A key seated in one teahouse but asking for another passes
sshd and is logged by the relay as `shell.denied`.

## 7. 本地开发 · Local development

`local-ssh` replaces sshd so the whole path can run on a laptop (WSL is fine):

```sh
export TEAHOUSE_LOCAL_KEYS=~/teahouse-dev/keys        # contains <name>.pub
export GIT_SSH_COMMAND="teahouse-relay --root ~/teahouse-dev/relay local-ssh"
export GIT_SSH_VARIANT=simple
git clone ssh://chenli@local/wutong
```

It reads the key for `chenli` from `$TEAHOUSE_LOCAL_KEYS/chenli.pub` and runs
the exact forced command OpenSSH would. It proves nothing about identity:
**never expose it on a real relay**. `scripts/demo.sh` uses it for a guided tour.
Commits must still be signed (§5): the demo configures each clone.

**The whole stack** (relay, the six example teahouses as real signed
history, the client engine and the interface): `scripts/dev-stack.sh`
(`TEAHOUSE_DEV_ME=<member>` chooses who you are, `--fresh` rebuilds,
`NO_UI=1` skips the interface). The engine reaches the relay through its
forced command directly (`teahouse-local://`), with no sshd.

**A real sshd on a development machine:** use the authorized_keys way
(`init --authorized-keys <file>` + `sync-keys`, `AuthorizedKeysFile <file>`).
`AuthorizedKeysCommand` needs a root-owned binary (§5), which `target/release/`
is not. sshd also needs no root for this: an unprivileged sshd on a high port
works. Keep the authorized_keys file under your home directory, not `/tmp`.
With `StrictModes`, sshd refuses a file below a world-writable directory.
`scripts/sshd-test.sh` sets all of this up; run it with `KEEP=1` and copy the
generated `sshd-B.conf` as a starting point.

## 8. 测试 · Tests

```sh
cargo test            # unit tests + end-to-end tests with real git
```

The end-to-end suite (`relay/tests/e2e.rs`) drives real `git clone/fetch/push`
through the forced-command path and covers every law above, including
attacks: self-promotion, touch-and-revert of the chart, evil merges, landing a
guest's chart change through the steward, force pushes, deleting `main`,
smuggling a bad ref next to a good one, 16 seats, two hosts, look-alike paths,
outsiders, removed members, pushes that bypass the shell, concurrent pushes,
and succession on a fresh relay.

```sh
./scripts/sshd-test.sh        # real OpenSSH, no root needed (~15 s)
./scripts/sshd-test.sh B      # only the authorized_keys way
```

`scripts/sshd-test.sh` puts a real, unprivileged sshd on `127.0.0.1:2222`
(`PORT=…` to change) in front of throw-away relays. It builds its sshd
configs from `deploy/sshd/teahouse.conf` itself, so the file we ship is the
file under test. If the file changes shape, the script stops instead of
testing something else. It runs once with `AuthorizedKeysCommand` and once
with the authorized_keys file, and checks with real `ssh` and `git`:

* outsiders, removed members and the host, steward and guest paths;
* `talk/` posts (a guest's own new post lands; impersonation, edits,
  replies to a missing notice and posts over 32 KiB are refused);
* direct login, `bash`, path tricks, and `-L` / `-W` / `-R` forwarding;
* the audit events;
* the client engine through the same sshd (E1–E6): joining with the device
  key from memory (libssh2), signed pushes, no seat → `not_a_member`,
  invitations (a brand-new device redeems through `redeem-shell` with
  `AuthorizedKeysCommand`; with the file way it cannot), attachments up and
  down over ssh, and a changed host key refused;
* that the `restrict` on each key line holds even under an sshd_config with
  no hardening at all.

If no sshd is installed, it unpacks Ubuntu's `openssh-server` into
`~/.cache/teahouse/sshd` with `apt-get download` + `dpkg -x`.

## 9. 已知局限 · Known limits (v0.1)

* **Verified behind OpenSSH 9.6p1** (Ubuntu 24.04) with both wirings, by
  `scripts/sshd-test.sh`, which CI runs on every push after fmt, clippy and
  `cargo test`. Other OpenSSH versions are untested; 9.8+ splits sshd into
  `sshd-session`, which should not matter to the relay but has not been tried.
* As a non-root user, `AuthorizedKeysCommand` can only be tested through a
  root-owned launcher (`/usr/bin/env`). The script first checks that sshd
  refuses the binary when it is not root-owned. With a root-owned install
  (§5), the shipped line works as is.
* Signatures are checked for pushed commits only; history that came with
  `open`/`adopt` is taken as it is. Clients do not yet verify the relay's own
  signature on seat commits.
* `members.toml` uses a strict TOML subset (bare keys, basic strings,
  integers, booleans, tables). Anything else is rejected, not guessed.
* Rate limits exist for invitations only (5 failures per key per hour, 30
  attempts per minute); everything else relies on sshd (`MaxStartups`,
  `MaxSessions`). With `AuthorizedKeysCommand`, any key on the internet can
  open a `redeem-shell` session: keep `MaxStartups` at its default or lower.
* Device pairing (INVITE.md rule 8, several keys per member) is not done; it
  changes the chart's format and is planned with step 4.
* Succession is manual (`adopt`); clients do not yet follow the heir by themselves.
