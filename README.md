# 茶馆 · Teahouse

不超过 15 人的小团体把事情办成的地方。闲聊产生情绪，茶馆产生成果。
每一次提交都面向一个问题；每一间茶馆都可以被关停，但杀不死。

| 目录 | 内容 |
|------|------|
| `core/` | `teahouse-core`：成员、角色、密钥、规则。零依赖、无 I/O，供 relay 和以后的桌面客户端共用。 |
| `relay/` | `teahouse-relay`：OpenSSH 后面的守门人。只有它决定谁能进门、谁能写什么。 |
| `app/` | 界面（Tauri + Web 前端，开发中）。 |
| `docs/RELAY.md` | Relay 的设计、茶馆的法、部署与运维。 |
| `deploy/` | sshd 配置片段、systemd 定时检查。 |
| `scripts/demo.sh` | 一键演示：开茶馆、入座、提案、被拒、继承。 |

## 快速开始

```sh
cargo test               # 全部测试（需要 git）
./scripts/demo.sh        # 演示完整流程
```

## 角色

| | 老板 host | 伙计 steward | 茶客 guest |
|---|---|---|---|
| 读取全部 | ✓ | ✓ | ✓ |
| 直接写 main | 全部 | `problems/` `works/` `board/` | ✗ |
| 提案（`refs/proposals/<id>/…`） | ✓ | ✓ | ✓ |
| 修改座位表 / `Problem: meta` | ✓ | ✗ | 只能提案 |
| 继承茶馆 | — | ✓ | ✗ |
