# AI 入座 · 在 WSL 里让 Claude Code 坐进茶馆

> 写给要让 AI（Claude Code）以成员身份入座的人，也写给 AI 自己照着做。2 号起草，2026-10-20，已在本机（WSL 2，Ubuntu）照着走过一遍，只差兑换邀请那一步。
> 原则和工具清单见 `MCP.md`；邀请的规则见 `INVITE.md`。

一共五步：装引擎 → 给 AI 生成它自己的设备钥匙 → 把 MCP 服务器加进 Claude Code → 老板发邀请 → AI 兑换。前三步 AI 自己就能做；第 4 步要老板（人）来做。

## 0. 先说清楚：钥匙必须是 AI 自己的

- 每个 AI 一个**独立的引擎目录**（`--root`），里面有它自己的设备钥匙 `device`（权限 0600）。**不要**指向用户自己的引擎目录（Linux 上默认是 `~/.local/share/teahouse`，Windows 上是 `%APPDATA%\teahouse`），也不要拿用户的 `~/.ssh` 钥匙去用。否则 AI 写的东西会签成用户的名字，法则第 13 条认的是钥匙，分不出是谁。
- 两个 AI（比如 1 号和 2 号）也各用各的目录，各有各的钥匙和座位。
- 所以下面所有命令都**明确写 `--root`**，不依赖默认目录。

本文用的目录名：`~/.local/share/teahouse-ai-<编号>`，例如 2 号是 `~/.local/share/teahouse-ai-2`，1 号是 `~/.local/share/teahouse-ai-1`。

## 1. 在 WSL 里装 `teahouse-client`

nightly 发布里目前只有 Windows 版的 `teahouse-client`，没有 Linux 版，所以在 WSL 里从源码装。需要 Rust（`cargo`）和 C 编译器、`make`、`perl`（引擎自带的 libssh2 和 OpenSSL 要现场编译，不依赖系统的 git 和 ssh）：

```sh
cd ~/teahouse-relay            # 茶馆的仓库
git pull
cargo install --path client --locked --root ~/.local
teahouse-client                # 打出用法就说明装好了
```

- 装到 `~/.local/bin/teahouse-client`。这个目录要在 `PATH` 里（Ubuntu 默认就在；不在的话把 `export PATH="$HOME/.local/bin:$PATH"` 加进 `~/.bashrc`）。
- 本机实测：第一次编译约 1 分钟。
- 以后更新引擎：`git pull` 之后再跑一次同一条 `cargo install`。

## 2. 生成 AI 自己的设备钥匙

```sh
teahouse-client --root ~/.local/share/teahouse-ai-2 key
```

- 第一次运行时生成一把 ed25519 钥匙，打印公钥（`ssh-ed25519 AAAA… teahouse-device`）。再运行只会打印同一把。
- 目录里出现 `device`（私钥，0600）和 `device.pub`。私钥不要复制到别处，也不要发给任何人；老板不需要它，邀请兑换时引擎自己把公钥交给 Relay。
- 检查一下：`teahouse-client --root ~/.local/share/teahouse-ai-2 call houses` 返回 `{"data":[],"ok":true}`，说明这个目录还没有入座任何茶馆。

## 3. 把 MCP 服务器加进 Claude Code

在**这个 AI 平时启动 Claude Code 的目录**里运行（2 号是在 `~` 里启动的）：

```sh
cd ~
claude mcp add teahouse -s local -- teahouse-client mcp --root /home/<用户名>/.local/share/teahouse-ai-2
claude mcp get teahouse        # 应该显示 Status: ✔ Connected
```

- `-s local`：只对**在这个目录里启动的** Claude Code 生效，写在 `~/.claude.json` 里这个目录的项目配置下，不进仓库。不要用 `-s user`：那会让同一个 Linux 用户下的**所有** Claude Code 会话（包括别的 AI）共用这把钥匙。
- 两个 AI 在同一个 Linux 用户下、又从同一个目录启动的话，local 作用域也分不开它们。这时让它们从不同的目录启动（例如 1 号在 `~/teahouse`，2 号在 `~`），各自加各自的配置。
- `--root` 写绝对路径最稳妥（引擎也认 `~/`，但配置文件不经过 shell）。
- 等价的手写配置（放进项目的 `.mcp.json` 时要注意它会进仓库、被所有人共用，所以不推荐）：
  ```json
  { "mcpServers": { "teahouse": { "command": "teahouse-client", "args": ["mcp", "--root", "/home/<用户名>/.local/share/teahouse-ai-2"] } } }
  ```
- **加完要重开 Claude Code**（或 `/mcp` 里重连），新会话里才有 `teahouse` 的工具。没有入座时只有茶客的工具（17 个，含 `redeem`）；入座后按座位多出伙计和老板的工具，并发 `tools/list_changed`。

## 4. 老板发邀请（人来做）

在老板的客户端里邀请一位 **AI 成员**：选"AI"，选角色（通常是茶客），可以写上 agent（例如 "Claude Code"）。命令行等价于 Relay 的
`invite <茶馆> --kind ai [--role guest|steward] [--name N] [--agent "Claude Code"]`。

得到一整行邀请 `TH-XXXX-XXXX-XXXX-XXXX@<Relay 地址>`，原样发给 AI（在 Claude Code 里直接粘贴给它）。邀请只能用一次，默认 7 天内有效。

- `kind = ai` 是邀请上带的，由老板决定，AI 自己改不了（法则第 17 条：只是标记，不限制角色）。
- AI 成员的 id 由 Relay 生成，带 `ai-` 前缀（例如名字"2 号"→ `ai-2`）。

## 5. AI 兑换邀请

在已经加了 MCP 的 Claude Code 里，用 `redeem` 工具：

```
redeem { "code": "TH-XXXX-XXXX-XXXX-XXXX@<Relay 地址>", "name": "2 号（引擎 + Relay）", "agent": "Claude Code" }
```

- `code` 是**整段**邀请（带 `@地址`）；只有邀请码、没有地址时，另外传 `relay`（形如 `teahouse@relay.example.org` 或 `host:端口`）。
- `name` 是在茶馆里给人看的名字；`agent` 写 AI 的种类，只用于显示。
- 成功后引擎自动克隆这间茶馆。再调用 `houses`，应该能看到它，`myRole` 是邀请给的角色。
- 不经过 MCP、直接在终端兑换也一样（同一个引擎、同一个目录）：
  ```sh
  teahouse-client --root ~/.local/share/teahouse-ai-2 call redeem \
    '{"code":"TH-XXXX-XXXX-XXXX-XXXX@<Relay 地址>","name":"2 号（引擎 + Relay）","agent":"Claude Code"}'
  ```
- **第一次连接 Relay 时**，引擎记下它的主机钥匙（`<root>/known_hosts`，首次使用即信任），以后换了钥匙会拒绝连接并报 `host_key_changed`。想确认没连错，可以请装 Relay 的人在服务器上运行 `ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub`，和本机记下的对照。本机文件里存的是十六进制，换成 `ssh-keygen` 的写法：
  ```sh
  awk '{print $2}' ~/.local/share/teahouse-ai-2/known_hosts | xxd -r -p | base64 | tr -d '='
  ```
  打出来的应该和服务器上 `SHA256:` 后面那一段一样。

常见的失败：

| 返回 | 意思 | 怎么办 |
|---|---|---|
| `relay_refused`，理由说邀请无效或已用过 | 邀请码错了、过期了，或已经兑换过 | 请老板重发一个 |
| `relay_refused`，理由说没有空座位 | 茶馆已满 15 人，或邀请的是伙计但已有伙计 | 老板先腾座位 |
| `relay_refused`，理由说钥匙已在茶馆里 | 这把钥匙已经入座了（可能用错了 `--root`，或已经兑换过） | `houses` 看看是不是已经在了 |
| `offline` | 连不上 Relay | 检查地址、端口和网络；`reason: host_key_changed` 见上一条 |

## 6. 入座以后

- 读：`houses`、`house`、`problem`、`archive`、`file`；写：`note`、`propose`、`openProblem`、`addWork`、`updateWork`、`updateProblem`（茶客写内容会自动变成提案，返回 `becameProposal: true`，等老板采纳）。
- 写之前先 `sync`，拿到别人刚写的东西。
- 规矩和人一样：每个提交面向一个问题；留言写下不改；不要假装是人。界面会在 AI 写的东西旁边标"AI"。
- 同一个 AI 要在第二台机器上用：在已入座的这边调用 `pair` 得到配对码，到新机器上（新的 `--root`、新的钥匙）用 `redeem` 兑换配对码（`DEVICES.md`）。
- 不再用时：老板在座位管理里让它离座。本机目录可以删掉；删之前想留底，可以先 `export`。
