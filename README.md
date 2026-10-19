# 茶馆 · Teahouse

不超过 15 人的小团体把事情办成的地方。闲聊产生情绪，茶馆产生成果。
每一次提交都面向一个问题；每一间茶馆都可以被关停，但杀不死。

| 目录 | 内容 |
|------|------|
| `core/` | `teahouse-core`：成员、角色、密钥、规则。零依赖、无 I/O，供 relay 和以后的桌面客户端共用。 |
| `relay/` | `teahouse-relay`：OpenSSH 后面的守门人。只有它决定谁能进门、谁能写什么。 |
| `client/` | `teahouse-client`：客户端引擎（`docs/CLIENT-API.md`）。每间茶馆一份本地副本，git 用进程内的 libgit2（含 libssh2），不调用 git 或 ssh 程序，可以直接编译到 Android。读写命令、提交签名、同步：`teahouse-client --root <目录> call <命令> '<json>'`；开发用 HTTP 接口：`teahouse-client serve`（`127.0.0.1:7377`）。 |
| `app/` | 界面（Tauri + Web 前端，开发中）。 |
| `docs/RELAY.md` | Relay 的设计、茶馆的法、部署与运维。 |
| `deploy/` | sshd 配置片段、systemd 定时检查。 |
| `scripts/demo.sh` | 一键演示：开茶馆、入座、提案、被拒、继承。 |
| `scripts/sshd-test.sh` | 真实 sshd 集成测试（非 root，两种接法）。 |

## 文档

- [使用说明](docs/USER-GUIDE.md)：给所有人
- [开一个 Relay](docs/DEPLOY.md)：给老板和运维
- [Relay 与茶馆的法](docs/RELAY.md)、[接口约定](docs/CLIENT-API.md)、[邀请](docs/INVITE.md)：给开发者

## 快速开始

```sh
cargo test               # 全部测试（需要 git）
./scripts/demo.sh        # 演示完整流程
./scripts/sshd-test.sh   # 在真实的 OpenSSH 后面验证（不需要 root，约 15 秒）
```

`sshd-test.sh` 在 `127.0.0.1:2222` 起一个非 root 的 sshd，用 `deploy/sshd/teahouse.conf`
生成配置，把 AuthorizedKeysCommand 和 authorized_keys 文件两种接法各测一遍。
测试内容包括：进门、读写权限、留言、直接登录、端口转发和审计日志；
还有客户端引擎经真实 sshd 的入座、签名推送、无座位被拒和主机钥匙变更（E1–E4）。
没装 sshd 的话，它会把 Ubuntu 的 openssh-server 解压到 `~/.cache/teahouse/sshd`。
常用参数：`./scripts/sshd-test.sh A` 或 `B` 只测一种接法；`PORT=2299` 换端口；
`KEEP=1` 保留临时目录，里面有生成的 sshd 配置和日志。全部通过时退出码为 0。

## 角色

| | 老板 host | 伙计 steward | 茶客 guest |
|---|---|---|---|
| 读取全部 | ✓ | ✓ | ✓ |
| 直接写 main | 全部 | `problems/` `works/` `board/` `talk/` | 只能新增自己的留言（`talk/`） |
| 提案（`refs/proposals/<id>/…`） | ✓ | ✓ | ✓ |
| 修改座位表 / `Problem: meta` | ✓ | ✗ | 只能提案 |
| 继承茶馆 | — | ✓ | ✗ |
