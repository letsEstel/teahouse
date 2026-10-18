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
| 7 | Every commit carries exactly one `Problem:` trailer: a 4-digit id whose file `problems/<id>-*.md` exists in that commit, `board` (may touch only `board/` and `talk/board/`), or `meta`. A post must sit under the topic its commit faces. | all |
| 8 | `Problem: meta` may land on `main` only from the host. | steward |
| 9 | `.teahouse/` contains only `members.toml`; any commit that changes it must leave a valid chart (1–15 seats, exactly one host, at most one steward, unique ids and keys, ed25519 only). | all |
| 10 | Paths that would turn into `.teahouse/` on Windows or macOS checkouts (`.TeaHouse/`, `.teahouse./`, `TEAHOU~1`) are refused. | all |
| 11 | At most 64 ref updates and 5 000 new commits per push; pack size ≤ 200 MiB; objects are fsck-checked on receive. | all |

Laws 6–9 apply to every commit a push would add to `main` — including commits
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
sudo install -o root -g root -m 0755 target/release/teahouse-relay /usr/local/bin/

# 2. A dedicated, unprivileged system user and data directory.
sudo useradd --system --home-dir /var/lib/teahouse --create-home --shell /bin/sh teahouse
sudo -u teahouse teahouse-relay --root /var/lib/teahouse init --binary /usr/local/bin/teahouse-relay

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

Members then use plain git:

```sh
git clone teahouse@relay.example.org:wutong
ssh teahouse@relay.example.org          # prints the teahouses you sit in
```

If you cannot use `AuthorizedKeysCommand`, run `init --authorized-keys
/var/lib/teahouse/.ssh/authorized_keys` instead: `post-receive` then rewrites
the managed block of that file whenever a seating chart changes, and
`sync-keys --file …` does it by hand. Lines outside the managed block are
kept.

## 6. 运维 · Operations

| Task | Command |
|------|---------|
| List teahouses | `teahouse-relay list` |
| Seats of one teahouse | `teahouse-relay members <id>` |
| Integrity check (fsck + charts + config) | `teahouse-relay check [<id>]` |
| After moving the binary or root | `teahouse-relay init --binary <path>` (rewrites hooks, re-configures every house) |
| Who did what | `log/audit.log` (`shell.read`, `shell.write`, `shell.denied`, `push.allowed`, `push.denied`, `push.applied`, `seats.changed`, `house.opened`, `house.adopted`) |
| Ref history on the server | `git --git-dir houses/<id>.git reflog main` |
| Backup | Any member's clone is a full backup. On the relay, back up `houses/` with `git bundle` or file-level snapshots. |

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

## 9. 已知局限 · Known limits (v0.1)

* **Not yet run behind a real sshd in CI.** The forced-command path is tested
  end to end; the sshd configuration in `deploy/` follows OpenSSH's documented
  semantics but has not been exercised in this repository's tests.
* Commit authorship (`git commit --author`) is not verified; the audit log
  records which **key** pushed each commit. Signed commits are a v0.2 topic.
* `members.toml` uses a strict TOML subset (bare keys, basic strings,
  integers, booleans, tables). Anything else is rejected, not guessed.
* No rate limiting beyond sshd's own (`MaxStartups`, `MaxSessions`).
* Succession is manual (`adopt`); clients do not yet follow the heir by themselves.
