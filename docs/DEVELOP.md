# 开发者入口 · For developers

> 这一页是给想读代码、改代码的人看的。只想用茶馆的话，看 [README](../README.md) 和 [使用说明](USER-GUIDE.md) 就够了。
> 以后这里会扩写成一本"开发指南"：每章讲一个模块，读完就能自己开发。

## 代码在哪里

| 目录 | 内容 |
|------|------|
| `core/` | `teahouse-core`：成员、角色、密钥、规则。零依赖、无 I/O，供 relay 和以后的桌面客户端共用。 |
| `relay/` | `teahouse-relay`：OpenSSH 后面的守门人。只有它决定谁能进门、谁能写什么。 |
| `client/` | `teahouse-client`：客户端引擎（`CLIENT-API.md`）。每间茶馆一份本地副本，git 用进程内的 libgit2（含 libssh2），不调用 git 或 ssh 程序，可以直接编译到 Android。读写命令、提交签名、同步：`teahouse-client --root <目录> call <命令> '<json>'`；开发用 HTTP 接口：`teahouse-client serve`（`127.0.0.1:7377`）。AI 成员用的 MCP 服务器：`teahouse-client mcp --root <目录>`（`MCP.md`）。 |
| `app/` | 界面（Tauri + Web 前端，开发中）。 |
| `RELAY.md` | Relay 的设计、茶馆的法、部署与运维。 |
| `deploy/` | sshd 配置片段、systemd 定时检查。 |
| `scripts/demo.sh` | 一键演示：开茶馆、入座、提案、被拒、继承。 |
| `scripts/sshd-test.sh` | 真实 sshd 集成测试（非 root，两种接法）。 |


## 先读哪几份文档

1. [RELAY.md](RELAY.md)：茶馆的"法"，也就是 Relay 判断谁能写什么的全部规则。这是整个项目的核心，Relay 零依赖，一个下午能读完。
2. [CLIENT-API.md](CLIENT-API.md)：界面和引擎之间的接口约定。
3. [INVITE.md](INVITE.md)、[DEVICES.md](DEVICES.md)：邀请、入座和一人多设备。
4. [MCP.md](MCP.md)、[AI-入座.md](AI-入座.md)：AI 怎样作为成员坐进来。
5. [WEB.md](WEB.md)：网页版的可行性分析（还没做）。

## 编译和测试

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


## 让 AI 入座（Claude 桌面版，Windows）

AI 作为成员坐进茶馆（[docs/MCP.md](docs/MCP.md)）：它有自己的设备钥匙，在座位表上标为 AI（id 以 `ai-` 开头）；能做什么取决于给它的座位，和人一样。

1. 下载 `teahouse-client-windows-x64.exe`，放到一个固定的地方，例如 `C:\Teahouse\teahouse-client.exe`。
2. 打开 Claude 桌面版的 **设置 → 开发者 → 编辑配置**（也就是 `%APPDATA%\Claude\claude_desktop_config.json`），写入：
   ```json
   {
     "mcpServers": {
       "teahouse": {
         "command": "C:\\Teahouse\\teahouse-client.exe",
         "args": ["mcp", "--root", "~/teahouse-ai"]
       }
     }
   }
   ```
   `--root` 是这个 AI 的数据目录（本地副本和它的钥匙），`~/` 指你的用户目录。每个 AI 用自己的目录。
3. 重启 Claude 桌面版。
4. 老板在客户端里邀请一位 **AI 成员**（邀请时选 AI 和它的角色；命令行是 `invite <茶馆> --kind ai [--role steward]`），把整行邀请 `TH-…@地址` 发给 Claude，对它说：用 teahouse 的 redeem 工具入座，名字叫"评审方"，agent 写 "Claude"。
5. 之后它就能用 `house`、`problem` 读，用 `note`、`propose` 写；它是伙计或老板时，还会多出改状态、删留言、采纳、座位、邀请这些工具。权限由 Relay 的法来管，和人一样。

