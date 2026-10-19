# 茶馆客户端引擎 · 接口约定（v0.1 草案）

> 状态：评审方起草，2026-10-19。这是界面（1 号）和客户端引擎（2 号）之间的合同。
> 两边都按这份文档开发；需要改动时，先在 PR 里改这份文档，再改代码。
> 茶馆的"法"仍以 `docs/RELAY.md` 第 3 节为准，本文只规定**内容格式**和**引擎接口**。

---

## 0. 一张图

```
┌──────────── 界面 app/ (Svelte) ────────────┐
│  views ──▶ Backend 接口 ──┬─ FakeBackend    │  假数据（现在的 data/*.ts）
│                          └─ LocalBackend ──┼──▶ HTTP JSON 127.0.0.1:7377   （开发期）
└────────────────────────────────────────────┘      或 Tauri invoke（打包后，同一组命令）
                                                       │
                              ┌────────── 引擎 client/ (Rust, teahouse-client) ──────────┐
                              │ 本地副本 ~/.local/share/teahouse/houses/<id>/  （普通 git 仓库）│
                              │ 读：git 历史 → JSON 快照      写：文件 + 提交 → 推送          │
                              │ 推送前用 teahouse-core 的规则预检，被 Relay 拒绝时原样转达理由   │
                              └──────────────────────────────┬──────────────────────────┘
                                                             │ git over ssh
                                                       teahouse-relay
```

**原则**
1. 引擎是唯一碰 git 的地方。界面永远看不到 commit、hash、branch、路径。
2. 同一组命令，两种传输：开发期是本机 HTTP（方便 Vite 热更新和截图脚本），打包后是 Tauri `invoke`。命令名、参数、返回值完全一致。
3. 引擎在推送前先用 `teahouse-core` 预检；预检能拦住的，就不让用户白跑一趟 Relay。最终裁决仍在 Relay。
4. 离线可写：提交先落在本地副本，联网后再推送。界面显示"未同步"。

---

## 1. 茶馆仓库里的内容格式

Relay 只管路径和权限；下面这些格式是客户端之间的约定，由引擎读写。所有文本都是 UTF-8，换行用 `\n`。

### 1.1 问题 `problems/<id>-<slug>.md`

```markdown
---
title: 小区停车位不好找
status: active            # open（待认领）| active（进行中）| solved（已办成）
owner: wang               # 认领人的成员 id，可省略
---
## 现状
院内共 42 个车位……

## 目标
年底前让每户至少有一个……

## 约束
不新增地面硬化……
```

- `id` 是四位数字，由引擎分配：取现有最大编号加一。两个人同时新开问题，后推送的一方会因为不是快进而被拒绝；引擎此时拉取最新状态，重新编号，然后重试一次。
- `slug` 只是为了文件名好认，引擎生成后不再改动。标题以 frontmatter 里的 `title` 为准。
- 正文的每个 `##` 小节，对应界面上的一个分段（`body[]`）。

### 1.2 成果 `works/<问题id>/<文件名>`

| 类型 `kind` | 文件 | 说明 |
|---|---|---|
| `doc` 文档 | `*.md` | frontmatter 里写 `title`，可选写 `kind: plan` 表示"方案" |
| `plan` 方案 | `*.md` | 同上，`kind: plan` |
| `table` 表格 | `*.csv` | 第一行是表头。标题放在同名的 `*.csv.meta.md` 的 frontmatter 里；没有这个文件时，用文件名当标题 |
| `image` 图片 | `*.png` `*.jpg` `*.webp` `*.svg` | 标题规则同表格（`*.png.meta.md`）|

- 其他扩展名一律当作"附件"：档案里能看到、能下载，但不能预览。
- 单个成果文件不超过 20 MiB（Relay 对整个推送包的上限是 200 MiB）。

### 1.3 公告和留言（Relay 已经强制执行，见 RELAY.md）

- 公告 `board/<公告名>.md`：frontmatter 里写 `title`，正文就是公告内容。
- 留言 `talk/<问题id>/<YYYYMMDDTHHMMSSZ>-<成员id>.md`，公告的回应放在 `talk/board/<公告名>/…`。文件内容就是留言正文，没有 frontmatter。
- **显示顺序 = 进入 main 的先后**，按 RELAY.md "显示顺序"一节给出的 git 命令计算，不按文件名里的时间戳排序。

### 1.4 提交说明（commit message）

```
<一句话，也就是界面上显示的动作描述>

<可选的正文：理由、说明>

Problem: 0001
Act: add-work
```

除了 Relay 要求的 `Problem:` trailer，引擎还会写一个 `Act:` trailer，供界面生成"经过"和"最近"：

| Act | 含义 | 由谁产生 |
|---|---|---|
| `open-problem` | 提出问题 | 新增 `problems/*.md` |
| `set-status` | 改变问题状态（另写 `Status: active` 等） | 修改 frontmatter 里的 status |
| `add-work` / `update-work` | 添加或更新成果 | 新增或修改 `works/` 下的文件 |
| `note` | 留言 | 新增 `talk/` 下的文件 |
| `notice` | 张贴公告 | 新增 `board/*.md` |
| `adopt` | 采纳提案（合并提交，另写 `Proposal: <成员id>/<提案名>`） | 老板 |
| `decline` | 不采纳提案（空提交，另写 `Proposal: …`，正文写理由） | 老板 |
| `seat` | 座位变更 | 修改 `members.toml` |

**没有 `Act:` 的提交**（比如有人直接用 git 提交）照样能显示：引擎会从改动的路径推断动作，推断不出来时显示为 `update`。

### 1.5 提案

- 分支名：`refs/proposals/<作者id>/<提案名>`，提案名由引擎从标题生成（ASCII slug 加短随机后缀）。
- 提案的标题和理由，就是这条分支上**第一个提交**的说明第一行和正文。
- **采纳**：老板一侧，引擎在本地把提案分支用 `--no-ff` 合并进 main，提交说明为 `采纳提案：<标题>`，带 `Act: adopt` 和 `Proposal:` 两个 trailer，然后推送 main，并在同一次推送里删除这条提案分支。
- **不采纳**：先推送一个空提交到 main（`Act: decline`），再删除提案分支。这样"已处理"列表永远可以从历史里还原，不依赖 Relay 的审计日志。

---

## 2. 身份与连接

**原则：身份属于人，不属于 Relay。** 成员的身份是自己设备上的密钥，Relay 只核对公钥。所以 Relay 被关停、伙计在别处重开茶馆时，每个人的身份原样有效，继承才成立。账号密码方案做不到这一点：身份会锁死在某一台 Relay 的数据库里。

**普通用户永远看不到"密钥"这个词。**
- 第一次运行时，引擎自动在本机生成一把 ed25519 设备密钥（私钥存进系统钥匙串，Windows 用凭据管理器）。界面上只问"你叫什么名字"。
- **入座靠邀请码**：老板在"座位管理"里点"邀请"，得到一个一次性邀请码（或链接）发给对方；对方在客户端里粘贴邀请码、填名字，引擎把设备公钥和邀请码一起交给 Relay，入座完成。（需要 Relay 新增"邀请"功能，见第 8 节。）
- **换设备、多设备**：在已登录的设备上点"添加一台设备"，得到一个短时有效的配对码，在新设备上输入即可。新设备拿到的是自己的新密钥，旧设备的密钥不需要导出。
- **高级选项**：懂行的用户可以在设置里改用已有的 ssh 密钥（如 `~/.ssh/id_ed25519`）。
- 引擎通过系统的 `ssh` 让 git 连接 Relay。
- 引擎的配置文件是 `~/.config/teahouse/client.toml`，记录每间茶馆的 Relay 地址和使用哪把密钥。
- "我是谁"由引擎判断：拿本机公钥和 `members.toml` 比对。找不到对应的座位，说明你已经离座，界面切换成只读并给出提示。
- 开发期用 Relay 的 `local-ssh`（见 RELAY.md 第 7 节），由 `scripts/dev-stack.sh` 一键配置。

---

## 3. 数据模型（引擎返回的 JSON）

字段名使用 camelCase，时间使用 RFC 3339 UTC，**用户写的内容都是普通字符串**（不分语言）；界面自身的文字仍然走 i18n。
角色使用 Relay 的名字：`host | steward | guest`。界面现在的 `boss | heir | guest` 在 `LocalBackend` 里做一次映射，以后统一改掉。

```ts
type Role = 'host' | 'steward' | 'guest'
type Status = 'open' | 'active' | 'solved'
type WorkKind = 'doc' | 'plan' | 'table' | 'image' | 'file'
type Act = 'open-problem' | 'set-status' | 'add-work' | 'update-work' | 'note' | 'notice'
         | 'propose' | 'adopt' | 'decline' | 'seat' | 'update'

interface HouseSummary {
  id: string; name: string; me: string; myRole: Role
  seats: number                 // 已坐的人数，上限固定 15
  pendingProposals: number      // 待审提案数（老板看到的徽标）
  sync: SyncState
}
interface SyncState {
  state: 'synced' | 'ahead' | 'behind' | 'offline' | 'error'
  unpushed: number              // 本地已提交、尚未推送的数量
  lastSync?: string
  message?: string              // 出错时给人看的一句话（来自 Relay 时原样转达）
}

interface Member { id: string; name: string; role: Role; joined: string }
interface Event  { by: string; act: Act; title: string; when: string; problem?: string; status?: Status }
interface Note   { by: string; text: string; when: string; writtenAt: string }  // when=进入茶馆的时间，writtenAt=文件名里的时间戳
interface Notice { id: string; by: string; title: string; text: string; when: string; replies: Note[] }

interface Work {
  id: string                    // 稳定 id，就是仓库路径的编码（界面不显示）
  kind: WorkKind; title: string; by: string; when: string
  size: number
  text?: string                 // doc/plan 的正文，table 的 CSV 原文；图片和附件不带
}

interface ProblemSummary {
  id: string; title: string; summary: string; status: Status
  by: string; owner?: string; opened: string; updated: string
  workCount: number; pendingProposals: number
}
interface Problem extends ProblemSummary {
  body: { heading: string; text: string }[]
  works: Work[]
  events: Event[]               // 经过：按时间顺序
  notes: Note[]                 // 讨论：按进入茶馆的先后
}

interface DiffLine { kind: 'same' | 'add' | 'del'; text: string }
interface FileChange { path: string; title: string; kind: WorkKind; change: 'add' | 'modify' | 'delete'; lines?: DiffLine[] }
interface Proposal { id: string; title: string; by: string; problem: string; when: string; reason?: string; files: FileChange[] }
interface Decided  { id: string; title: string; by: string; result: 'adopted' | 'declined'; when: string; reason?: string }

interface House {
  id: string; name: string; signboard?: string; founded: string; capacity: 15
  me: string; members: Member[]
  problems: ProblemSummary[]; recent: Event[]
  proposals: Proposal[]; decided: Decided[]; board: Notice[]
  sync: SyncState
}

// 档案
interface ArchiveNode {
  id: string; label: string     // label 用人话（"成果 / 0001 · 小区停车位不好找"）
  kind: 'dir' | 'problem' | WorkKind | 'notice' | 'note' | 'members'
  problem?: string; size?: number
  children?: ArchiveNode[]
}
interface HistoryEntry { by: string; when: string; act: Act; problem: string; title: string; diff?: DiffLine[] }
```

---

## 4. 命令

传输方式：开发期 `POST http://127.0.0.1:7377/v1/<命令名>`，请求体和返回体都是 JSON；打包后用 `invoke('<命令名>', 参数)`。
所有命令都返回 `{ ok: true, data }` 或 `{ ok: false, error }`（见第 5 节）。

### 读取（只读本地副本，不连网，必须快）

| 命令 | 参数 | 返回 |
|---|---|---|
| `houses` | — | `HouseSummary[]` |
| `house` | `{ house }` | `House` |
| `problem` | `{ house, problem }` | `Problem` |
| `archive` | `{ house, view: 'dirs' \| 'problems' }` | `ArchiveNode` |
| `file` | `{ house, id }` | `{ node: ArchiveNode, text?: string, history: HistoryEntry[] }` |
| `download` | `{ house, id }` | 原始字节（HTTP 直接返回文件；Tauri 返回本地路径） |

### 同步

| 命令 | 参数 | 说明 |
|---|---|---|
| `sync` | `{ house? }` | 先拉取，再推送本地积压的提交；返回新的 `SyncState`。引擎也会定时在后台同步（默认 60 秒一次，失败后退避重试） |
| `join` | `{ relay, house }` | 用本机密钥克隆一间茶馆。`relay` 形如 `teahouse@relay.example.org` |

### 写入（都是"先写本地、再尝试推送"）

| 命令 | 参数 | 谁能用 | 产生的提交 |
|---|---|---|---|
| `openProblem` | `{ house, title, body[] }` | 老板、伙计（茶客自动转为提案） | `open-problem` |
| `setStatus` | `{ house, problem, status, owner? }` | 老板、伙计 | `set-status` |
| `addWork` | `{ house, problem, kind, title, text? \| fromPath? }` | 老板、伙计（茶客自动转为提案） | `add-work` |
| `updateWork` | `{ house, id, text? \| fromPath? }` | 同上 | `update-work` |
| `note` | `{ house, problem \| notice, text }` | 所有人 | `note` |
| `postNotice` | `{ house, title, text }` | 老板、伙计（茶客自动转为提案） | `notice` |
| `propose` | `{ house, problem, title, reason?, changes: {id?, kind, title, text? \| fromPath?, delete?}[] }` | 所有人 | 提案分支 |
| `withdraw` | `{ house, proposal }` | 提案作者 | 删除提案分支 |
| `adopt` | `{ house, proposal }` | 老板 | `adopt`，同时删除提案分支 |
| `decline` | `{ house, proposal, reason? }` | 老板 | `decline`，同时删除提案分支 |
| `seat` | `{ house, add?: Member&{key}, remove?: id, role?: {id, role} }` | 老板 | `seat` |

**界面按角色分开设计，引擎再兜一层底。**
- 界面以 `House.me` 的角色为准，**按角色显示不同的界面**：老板有"座位管理"和"提案审阅"，别人看不到这两样；茶客看到的按钮直接写成"提议添加成果""提议一条公告"，点之前就知道这会交给老板。
- 引擎的"自动转为提案"只是兜底：如果茶客的请求还是走到了老板或伙计才能用的**内容类**写命令（`openProblem`、`addWork`、`updateWork`、`postNotice`），引擎不报错，而是改为发起提案，返回值带上 `{ becameProposal: true }`。这种情况通常只发生在角色刚变动、界面还没刷新的时候。
- **管理类命令绝不自动转换**：`seat`、`adopt`、`decline`、`setStatus` 只有对应角色能用，其他人调用一律返回 `forbidden`。

写命令的返回值：`{ commit: 'local' | 'pushed', sync: SyncState, becameProposal?: boolean, id?: string }`。

### 事件推送（可选，M2 之后再做）

`GET /v1/events`（SSE）：茶馆在后台同步后有变化时，推送 `{ house }`，界面收到后重新读取数据。

---

## 5. 错误

```ts
interface ApiError {
  code: 'not_found' | 'not_a_member' | 'forbidden' | 'invalid' | 'conflict'
      | 'offline' | 'relay_refused' | 'too_large' | 'internal'
  message: string              // 给人看的一句话（中英双语，和 Relay 的风格一致）
  reasons?: string[]           // relay_refused 时，Relay 给出的逐条理由（去掉 "remote:" 前缀）
}
```

- `forbidden`：引擎预检就拦下了（例如伙计想改 `members.toml`）。没有提交，什么都没发生。
- `relay_refused`：提交已经在本地，但 Relay 拒绝了推送。引擎把这次本地提交撤回到推送前的状态，**用户写的内容保留在草稿里**，不会丢。
- `conflict`：别人先推送了。引擎会自动拉取并在最新状态上重放一次；只有重放仍然失败时才返回这个错误。

---

## 6. 开发与测试环境

- `scripts/dev-stack.sh`：一条命令起整套环境
  1. 在临时目录里初始化一个本地 Relay（`local-ssh`）
  2. 用 `fixtures/` 里的示例数据，把六间示例茶馆**种成真的 git 仓库**：按时间顺序逐条提交，提交时间用示例数据里的时间，作者用示例成员
  3. 为每个示例成员生成密钥；"我"用哪个成员登录，由环境变量 `TEAHOUSE_DEV_ME=<成员id>` 决定
  4. 启动 `teahouse-client serve --port 7377`
  5. 启动 `npm run dev`，并设置 `VITE_BACKEND=local`
- `fixtures/houses.zh.json`：由 1 号从现有的假数据导出中文版（`npm run export-fixtures`）。种仓库用中文；英文和德文的示例内容只保留给 FakeBackend 截图用。
- 测试：引擎自己的测试覆盖每一条命令；再加一组端到端测试，在真实的 Relay 和本地副本之间跑完"开问题 → 添加成果 → 茶客提案 → 老板采纳 → 留言"整个流程。

---

## 7. 分工与顺序

| 步骤 | 1 号（界面） | 2 号（引擎） |
|---|---|---|
| 1 | 定义 `Backend` 接口（第 4 节），现有假数据放到 `FakeBackend` 后面；界面视觉不变；导出 `fixtures/houses.zh.json` | 新建 `client/` crate：读取命令（第 4 节"读取"）和第 1 节的格式解析 |
| 2 | 实现 `LocalBackend`（HTTP），用 `VITE_BACKEND=local` 切换；`boss/heir` 映射成 `host/steward` | 写入命令、预检、推送、撤回、重放；`serve` 子命令 |
| 3 | 处理"未同步""已离座""被 Relay 拒绝"这三种状态的界面 | `dev-stack.sh`、种仓库、端到端测试 |
| 4 | 联调：在 `dev-stack` 上把六间茶馆全部走一遍，截图 | 联调：修复问题 |

**依赖库**：引擎可以使用第三方库，但每加一个都要在 PR 里写明理由。建议：`serde`/`serde_json`（JSON），一个小型 HTTP 服务器（`tiny_http` 或 `axum` 二选一），git 操作**继续调用系统的 git**，不引入 libgit2，以便和 Relay 的行为保持一致。

---

## 8. 待定（需要用户或评审拍板）

1. **已定方向（用户 10-19）**：首次运行自动生成设备密钥，用户不接触密钥；保留"使用已有密钥"的高级选项。**待定**：要不要额外提供"邮箱 + 密码"登录。评审建议：不做账号密码，用"邀请码 + 设备配对码"达到同样的易用性，理由见第 2 节开头。
2. 图片成果的缩略图由引擎生成还是由界面生成（建议：界面直接显示原图，M2 之后再考虑缩略图）。
3. 茶客离线时写好的提案，在已经离座之后才同步：直接丢弃，还是导出成文件交给用户（建议：导出，绝不静默丢弃）。
4. **Relay 新增"邀请"功能**（属于"法"，需要用户和评审定稿）：老板生成一次性邀请码 → 新人持码提交公钥和名字 → 自动写入 `members.toml`（以老板的名义提交，`Act: seat`）。邀请码的有效期、能不能指定角色，待定。
