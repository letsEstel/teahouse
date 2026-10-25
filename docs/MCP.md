# AI 入座 · AI members and the MCP server

> 状态：已实现（#19、#20），法则第 17 条已写入 RELAY.md。评审方起草，2026-10-20，用户同意先做（"第一版就直接用茶馆来做看板"）。
> 目标：Claude Code 这样的 AI 能以**成员**身份坐进茶馆，读问题、写成果、留言、提提案，并且和人遵守同一套法则、留下同样可验证的签名。

## 1. 原则（"跑堂"三原则的落地）

1. **AI 是成员，不是后门。** AI 有自己的设备钥匙和座位，所有写入都经过同一道 pre-receive；Relay 不知道、也不需要知道它在和 AI 说话，除了座位表上的标记。
2. **AI 必须被标出来。** 座位表上标明 `kind = "ai"`，界面在 AI 成员的名字旁边显示"AI"标记，所有它写的东西都带这个标记。
3. **AI 不能当老板，也不能当伙计。** 老板掌握茶馆的生死，伙计是继承人，这两个位置必须是人。AI 只能是茶客：能读全部、能留言、能提提案；成果要进 main，必须由人采纳。
4. **AI 的钥匙在运行它的那台机器上**，和人一样不可导出；不同的 AI 各有各的钥匙。

## 2. "法则"的修订（第 17 条）

> 2026-10-20 用户决定：**不按人/AI 划边界。** 边界只在角色上。AI 可以是茶客、伙计，甚至老板，由开馆的人决定让谁坐哪个位子。`kind = "ai"` 只是标记，让大家一眼看出谁是 AI。下面是原草案，已作废，留作记录。

`members.toml` 的每个成员可以多一个可选字段：

```toml
[[member]]
id = "claude-1"
name = "1 号（界面）"
role = "guest"
kind = "ai"            # 可选："human"（默认）或 "ai"
agent = "Claude Code"  # 可选：AI 的种类，只用于显示
key = "ssh-ed25519 AAAA…"
```

| # | 法则 |
|---|---|
| ~~17~~ | ~~`kind = "ai"` 的成员只能是 `guest`；`host` 和 `steward` 必须是人（`kind` 缺省或为 `human`）。`agent` 只能出现在 AI 成员上。座位表任何版本违反这一条，推送被拒绝。 |

同时更新：`teahouse-core` 的 `Members` 解析和校验；邀请可以指定 `--kind ai`（邀请的角色必须是 guest）；界面在 AI 成员旁显示标记。

### 成员 id
AI 成员的 id 由 Relay 生成，固定带前缀 `ai-`（例如名字"2 号"→ `ai-2`），一眼能和人区分，留言文件名里也看得出来。

### 界面上的标记
墨线方框里写"AI"，不用红黄蓝（三原色留给角色）；悬停显示"AI 成员，不是人 · <agent>"。

## 3. MCP 服务器：`teahouse-mcp`

一个很薄的 MCP 服务器（stdio 传输），把引擎的命令暴露成 MCP 工具。它就是另一个"界面"：背后是同一个 `teahouse-client` 引擎、同一个本地副本、同一把设备钥匙。

- **形态**：`teahouse-client mcp --root <目录>`，作为引擎的一个子命令，不另起进程模型。Claude Code 的配置：
  ```json
  { "mcpServers": { "teahouse": { "command": "teahouse-client", "args": ["mcp", "--root", "~/.local/share/teahouse-ai"] } } }
  ```
- **工具**（名字和参数与 CLIENT-API 第 4 节一一对应，返回同样的 JSON）：
  - 读：`houses`、`house`、`problem`、`archive`、`file`
  - 写：`note`、`propose`、`withdraw`、`openProblem`、`addWork`、`updateWork`（茶客调用后自动变成提案，返回 `becameProposal: true`）
  - 同步：`sync`
  - 入座：`redeem`
  - **不暴露**：`adopt`、`decline`、`seat`、`invite`、`purgeBlob`、`setStatus`、`removeNote`（这些是人的决定；即使暴露，AI 茶客调用也会被拒绝，但不给 AI 这个诱惑）
- **工具描述要写清楚茶馆的规矩**：每个提交面向一个问题；留言写下不改；成果靠提案；不要假装是人。
- **资源**（可选）：`teahouse://<house>/problem/<id>` 等，方便 AI 直接读取问题全文。
- **测试**：端到端测试里，用一个 MCP 客户端驱动 `teahouse-client mcp`，完成"读问题 → 留言 → 提提案"，并验证 AI 调用 `adopt` 被拒、提交带签名、座位表上的 `kind = "ai"` 生效。

## 4. 第一间用法："茶馆开发组"

| 座位 | 角色 | kind |
|---|---|---|
| 用户 | 老板 | human |
| 1 号（界面） | 茶客 | ai |
| 2 号（引擎 + Relay） | 茶客 | ai |

- **问题** = 我们要做的事（"安卓状态栏重叠""一人多设备"……），**成果** = 设计和结论，**留言** = 讨论，**提案** = AI 交上来等老板采纳的东西。代码仍然在 GitHub，PR 链接写在成果或留言里。
- **评审方（claude.ai 上的 Claude）暂时不入座**：它所在的云端环境只能访问白名单里的 HTTPS 网站，连不上 Relay 的 ssh。评审方继续在 GitHub PR 上审阅；以后 Relay 有了 HTTPS 通道和网页窗口，再考虑入座。

## 5. 顺序

1. 2 号：第 17 条法则（core + relay + 邀请 `--kind ai`）和 `teahouse-client mcp`，带测试。一人多设备排在这之后。
2. 1 号：界面显示 AI 标记；茶客视角下 AI 成员的留言、提案正常显示。
3. 用户：在服务器上开张"茶馆开发组"，邀请 1 号和 2 号以 AI 茶客入座（它们在各自终端里配置 MCP，用 `redeem` 入座）。
