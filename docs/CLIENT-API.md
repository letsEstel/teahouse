# 茶馆客户端引擎 · 接口约定（v0.1 草案）

> 状态：评审方起草，2026-10-19，**第三版**。用户已定：**客户端优先（本地优先，像 Obsidian）**，Relay 是会合点和执法者；网页渲染（公开分享）和网页窗口以后做。这是界面（1 号）和客户端引擎（2 号）之间的合同。
> 两边都按这份文档开发；需要改动时，先在 PR 里改这份文档，再改代码。
> 茶馆的"法"仍以 `docs/RELAY.md` 第 3 节为准，本文只规定**内容格式**和**引擎接口**。

---

## 0. 一张图

```
┌──────────── 茶馆客户端（Tauri：Windows / macOS / Linux，以后手机）────────────┐
│ 界面 app/（Svelte，现在这套）──invoke──▶ 引擎 client/（Rust，teahouse-client）  │
│                                          · 整间茶馆的完整副本（普通 git 仓库）   │
│                                          · 图片和附件的本地缓存                 │
│                                          · 设备钥匙（系统钥匙串）               │
└──────────────────────────────────────────────┬──────────────────────────────┘
                                               │ git over ssh + 附件传输（同一条 ssh 通道）
                                               ▼
┌──────────────── Relay（一台 VPS，或者茶馆托管服务）────────────────┐
│ 正本：houses/<id>.git      法：pre-receive（含签名检查）            │
│ 附件仓库：blobs/<sha256>   邀请码兑换、审计日志                      │
│ 以后：公开页面渲染（像 Obsidian Publish）、网页窗口                  │
└──────────────────────────────────────────────────────────────────┘
```

**原则**
1. **本地优先。** 每个客户端都有整间茶馆的完整副本，断网照常读写，联网后自动同步。读取永远只读本地，所以必须快。
2. **只有进了正本的才算数。** 法只在 Relay 执行；引擎推送前用 `teahouse-core` 预检，给出更早、更友好的提示。
3. **界面永远看不到 git。** 界面只调用第 4 节的命令。
4. **同一组命令，两种传输。** 打包后用 Tauri `invoke`；开发期引擎同时提供本机 HTTP 接口（`127.0.0.1:7377`），方便 Vite 热更新和截图脚本。命令名、参数、返回值完全一致。
5. **谁也冒充不了谁。** 每个提交都用作者的设备钥匙签名（git 自带的 ssh 签名），Relay 只接受座位表上有对应钥匙的签名，连 Relay 自己也伪造不了。

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
- frontmatter 之后、第一个 `##` 之前的那段文字是**摘要**（`summary`）；每个 `##` 小节对应界面上的一个分段（`body[]`）。
- **frontmatter 语法**（全文通用）：只有一层 `key: value`，不支持嵌套；` #` 之后是注释；值可以加双引号，引号里按 JSON 转义。引擎写入时，值里出现 `#`、`:` 或首尾有空格，一律加引号。不使用 YAML 库。
- **编号唯一**：main 上每个编号最多对应一个 `problems/<id>-*.md`（Relay 检查，见第 8 节第 2 条）。

### 1.2 成果 `works/<问题id>/<文件名>`

| 类型 `kind` | 文件 | 说明 |
|---|---|---|
| `doc` 文档 | `*.md` | frontmatter 里写 `title`；可选 `kind: plan` 表示"方案"；可选 `note:` 一句话说明（成果卡片上标题下面那一行） |
| `plan` 方案 | `*.md` | 同上，`kind: plan` |
| `table` 表格 | `*.csv` | 第一行是表头。标题放在同名的 `*.csv.meta.md` 的 frontmatter 里；没有这个文件时，用文件名当标题 |
| `image` 图片 | 附件引用（见 1.6） | 标题写在引用文件里 |
| `file` 附件 | 附件引用（见 1.6） | 同上 |

### 1.3 公告和留言（Relay 已经强制执行，见 RELAY.md）

- 公告 `board/<公告名>.md`：frontmatter 里写 `title`，正文就是公告内容。
- 留言 `talk/<问题id>/<YYYYMMDDTHHMMSSZ>-<成员id>.md`，公告的回应放在 `talk/board/<公告名>/…`。文件内容就是留言正文，没有 frontmatter。
- **显示顺序 = 进入 main 的先后**，按 RELAY.md "显示顺序"一节给出的 git 命令计算，不按文件名里的时间戳排序。

**`works/` 下文件的分类，按优先级判断**：`*.ref.md` → 附件（`type` 为 `image/*` 时是 `image`，否则是 `file`）；`*.csv.meta.md` → 只作为表格的标题和说明，不单独列出；`*.md` → `plan`（frontmatter 写了 `kind: plan`）或 `doc`；`*.csv` → `table`；其他文件（1.6 节之前直接放进 git 的）→ `file`，带 `size`，没有 `blob`。

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
| `remove-note` | 删除留言 | 老板、伙计 |

采纳和不采纳的提交同样必须带 `Problem:`：取提案所针对的问题；提案改的是公告时写 `board`。
| `seat` | 座位变更 | 修改 `members.toml` |

**没有 `Act:` 的提交**（比如有人直接用 git 提交）照样能显示：引擎会从改动的路径推断动作，推断不出来时显示为 `update`。

### 1.5 提案

- 分支名：`refs/proposals/<作者id>/<提案名>`，提案名由引擎从标题生成（ASCII slug 加短随机后缀）。
- 提案的标题和理由，就是这条分支上**第一个提交**的说明第一行和正文。
- **采纳**：老板一侧，引擎在本地把提案分支用 `--no-ff` 合并进 main，提交说明为 `采纳提案：<标题>`，带 `Act: adopt` 和 `Proposal:` 两个 trailer，然后推送 main，并在同一次推送里删除这条提案分支。
- **新问题的编号撞车**：提案里新建了 `problems/0007-…`，但采纳时 0007 已被占用，引擎就在**合并提交本身**里把文件（连同 `talk/0007/…`）改到新编号，并在提交说明里写 `Renumbered: 0007 -> 0009`。合并提交由老板签名，茶客原来的提交不动。
- **决定不能撤回**：采纳和不采纳推送之后就是历史。以后如果要"改主意"，做法是一次新的提交（例如重新打开提案）。
- **不采纳**：先推送一个空提交到 main（`Act: decline`），再删除提案分支。这样"已处理"列表永远可以从历史里还原，不依赖 Relay 的审计日志。

### 1.6 图片和附件：不进 git，只进 git 一行引用

二进制文件（图片、PDF、音频……）**不存进 git 历史**，因为历史不能改写，误传的隐私照片会永远删不掉，仓库也会越来越大。做法：

- 文件本身存在**附件仓库**里，按内容哈希命名：`blobs/<sha256>`（Relay 上一份，每个客户端本地缓存一份）。
- git 里放一个很小的**引用文件**，例如 `works/0001/车位照片.png.ref.md`：
  ```markdown
  ---
  title: 车位照片
  blob: sha256:9f2c…e41a
  size: 482113
  type: image/png
  ---
  可选的说明文字
  ```
- 渲染时，界面按引用去本地缓存取文件；本地没有就向 Relay 下载，下载后校验哈希。
- **删除**：老板执行 `purgeBlob`：老板的引擎先把引用标成 `purged: <时间> <老板id>`，作为老板签名的提交推送；推送成功后 Relay 删除文件本身；各客户端同步后删除本地缓存。历史依然完整、诚实，但内容被移除了。（实话：一个刻意不听话的客户端可以保留自己已经下载的那份。本地优先的软件都有这个限制。）
- 正文里也可以直接贴外部网址，只当普通链接，不做校验、不缓存。
- 单个附件不超过 100 MiB；每间茶馆的附件总量上限由 Relay 配置。

---

## 2. 身份与连接

**原则：身份属于人，不属于 Relay。** 成员的身份是自己设备上的钥匙，Relay 只核对公钥、不保存密码。Relay 被关停、伙计在别处重开茶馆时，每个人的身份原样有效，继承才成立。

**普通用户永远看不到"密钥"这个词。**
- 第一次运行：引擎自动生成一把 ed25519 设备钥匙（私钥存进系统钥匙串，Windows 用凭据管理器），界面上只问"你叫什么名字"。
- **入座**：粘贴老板发来的邀请码。引擎通过 ssh 连到 Relay，执行兑换（`docs/INVITE.md`），把设备公钥和名字交上去。
- **多设备**：在已入座的设备上点"添加一台设备"，得到一个配对码，在新设备上输入。新设备生成自己的钥匙，旧钥匙不需要导出。（需要座位表支持"一人多钥匙"，见第 8 节。）
- **高级选项**：懂行的用户可以改用已有的 ssh 密钥。
- **签名**：引擎对每个提交做 git 的 ssh 签名（`gpg.format=ssh`）。签名用的就是设备钥匙。
- 传输：**引擎内置 git 库，不依赖系统里的 git 程序**（用 `git2` crate，也就是 libgit2 加 libssh2；ssh 认证和提交签名都在进程内完成）。这样同一个引擎可以直接编译到 Windows、macOS、Linux 和 **Android**，打包时也不用附带 git。（用户 10-19：手机是安卓，要尽快在手机上用，所以从第一天起就这样做。）
- 配置：`<应用数据目录>/teahouse/client.toml`，记录每间茶馆的 Relay 地址。
- 本地目录（`<root>` 即应用数据目录下的 `teahouse/`）：`houses/<id>/` 是各茶馆的 git 副本；`blobs/<sha256>` 是附件缓存；`drafts/<id>/` 是被拒绝或离线时保存的草稿；设备钥匙在第 4 步打包前暂存为 `<root>/device`（权限 0600），打包时移入系统钥匙串。
- "我是谁"：拿本机公钥和 `members.toml` 比对。找不到座位，说明已经离座，界面切换成只读并给出提示，本地副本保留。
- 开发期用 Relay 的 `local-ssh`（RELAY.md 第 7 节），由 `scripts/dev-stack.sh` 一键配置。

---

## 3. 数据模型（引擎返回的 JSON）

字段名使用 camelCase，时间使用 RFC 3339 UTC，**用户写的内容都是普通字符串**（不分语言）；界面自身的文字仍然走 i18n。
角色使用 Relay 的名字：`host | steward | guest`（界面内部也已统一）。
**已离座**：`House.me` / `HouseSummary.me` 为空字符串，表示本机钥匙不在座位表上；此时 `myRole` 没有意义，界面整间只读，但本地副本可以看、可以导出。
**给人看的固定文字由界面翻译**：引擎只返回代码（目录种类、错误代码及参数），界面按中英德显示；只有用户自己写的内容（标题、正文、成员名）是原样字符串。
**时间**：`when`（进入茶馆的时间）取把它带进 main 的那个第一父提交的 committer 时间，仅供显示，可能有设备时钟误差；**顺序**以第一父链为准，这是可靠的。可信的服务器时间以后由 Relay 另行记录。
`Member.joined` 取第一次加入这个 id 的提交时间，`House.founded` 取根提交时间。

```ts
type Role = 'host' | 'steward' | 'guest'
type Status = 'open' | 'active' | 'solved'
type WorkKind = 'doc' | 'plan' | 'table' | 'image' | 'file'   // image、file 是附件引用（1.6）
type Act = 'open-problem' | 'set-status' | 'add-work' | 'update-work' | 'note' | 'remove-note' | 'notice'
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

interface Member { id: string; name: string; role: Role; joined: string; kind: 'human' | 'ai'; agent?: string; devices?: DeviceInfo[] }  // kind 只是标记，不限制角色（法第 17 条）；界面在 AI 成员旁显示"AI"标记；devices 只出现在自己的座位上
interface DeviceInfo { name: string; fingerprint: string; added: string; thisDevice: boolean }  // name 为空 = 第一台设备；fingerprint 形如 SHA256:…（和 ssh-keygen -l 一致）
interface Event  { by: string; act: Act; title: string; when: string; problem?: string; status?: Status }
interface Note   { id: string; by: string; text: string; when: string; writtenAt: string }  // id=留言文件的仓库路径  // when=进入茶馆的时间，writtenAt=文件名里的时间戳
interface Notice { id: string; by: string; title: string; text: string; when: string; replies: Note[] }

interface Work {
  id: string                    // 稳定 id，就是仓库路径的编码（界面不显示）
  kind: WorkKind; title: string; by: string; when: string
  size: number
  note?: string                 // 一句话说明（frontmatter 的 note:）
  text?: string                 // doc/plan 的正文，table 的 CSV 原文；图片和附件不带
  blob?: { hash: string; type: string; cached: boolean; purged?: boolean }
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
  id: string                    // 就是仓库路径，界面放进地址栏时用 encodeURIComponent
  dir?: 'problems' | 'works' | 'board' | 'talk' | 'meta'   // 目录种类，由界面翻译
  label?: string                // 只放用户写的内容（问题标题、公告标题、成员名）
  kind: 'dir' | 'problem' | WorkKind | 'notice' | 'note' | 'members'
  problem?: string; size?: number
  children?: ArchiveNode[]
}
interface HistoryEntry { by: string; when: string; act: Act; problem: string; title: string; status?: Status; diff?: DiffLine[] }
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
| `blob` | `{ house, id }` | 附件的本地路径（没有缓存时先下载并校验）。界面显示时：Tauri 里用 `convertFileSrc`；开发期用 `serve` 提供的只读地址 `GET /v1/blob?house=<id>&id=<附件id>`，只返回该茶馆附件缓存里的文件 |
| `export` | `{ house, to }` | 把整间茶馆（git bundle + 附件 + 一份可直接阅读的文件夹）导出到 `to` |

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
| `addWork` | `{ house, problem, kind, title, text? \| fromPath? }`（图片和附件用 `fromPath`，引擎负责上传附件并写引用） | 老板、伙计（茶客自动转为提案） | `add-work` |
| `updateWork` | `{ house, id, text? \| fromPath? }` | 同上 | `update-work` |
| `note` | `{ house, problem \| notice, text }` | 所有人 | `note` |
| `postNotice` | `{ house, title, text }` | 老板、伙计（茶客自动转为提案） | `notice` |
| `propose` | `{ house, problem, title, reason?, changes: {id?, kind, title, text? \| fromPath?, delete?}[] }` | 所有人 | 提案分支 |
| `withdraw` | `{ house, proposal }` | 提案作者 | 删除提案分支 |
| `adopt` | `{ house, proposal }` | 老板 | `adopt`，同时删除提案分支 |
| `decline` | `{ house, proposal, reason? }` | 老板 | `decline`，同时删除提案分支 |
| `seat` | `{ house, remove?: id, role?: {id, role} }` | 老板 | `seat`（加人只走邀请） |
| `invite` | `{ house, role?, name?, days? }` | 老板 | 无提交；返回**完整邀请** `TH-XXXX-XXXX-XXXX-XXXX@<relay>[:端口]`（`@` 后是 Relay 地址，ssh 用户默认 `teahouse`），界面整段复制给对方 |
| `redeem` | `{ relay, code, name }` | 尚未入座的人，或要加入的新设备 | Relay 代写 `seat`；完成后自动克隆茶馆。界面从粘贴的整段文字里认出 `TH-…@地址` 或 `teahouse://地址/…`；只有邀请码时才多问地址。**同一个入口也接受配对码**：配对码覆盖几间茶馆，就加入几间 |
| `pair` | `{ house?, name? }` | 所有人（AI 也可以） | 无提交；返回配对码 `TH-…@<relay>`，10 分钟内有效、只能用一次，对这台设备在该 Relay 上入座的**所有茶馆**有效。`name` 是新设备的名字（如"手机"）。界面在"添加一台设备"里整段显示，供新设备粘贴（见 DEVICES.md） |
| `devices` | `{ house }` | 所有人 | 无提交；返回 `DeviceInfo[]`（我的全部设备） |
| `unpair` | `{ house?, device }` | 所有人（只能移除自己的另一台设备） | Relay 代写座位变更；`device` 是设备名或 `SHA256:` 指纹。不指定 `house` 时先同步，再在所有有这台设备的茶馆里移除。正在用的这台、最后一台都不能移除。返回移除了的茶馆 id |
| `drafts` | `{ house }` | 所有人 | 无提交；返回 `{ id, kind, title, text, when, reason? }[]`（被拒绝或离线时保存的草稿） |
| `discardDraft` | `{ house, id }` | 所有人 | 无提交；删除一份草稿 |
| `purgeBlob` | `{ house, id }` | 老板 | 删除附件内容，引用标成 purged |
| `removeNote` | `{ house, id }` | 老板、伙计 | `remove-note` |

`propose` 的 `title` 由界面自动生成（"修改《…》""添加《…》"），茶客只需要写理由。

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
  params?: Record<string, string>   // 例如 { path: 'members' }，界面据此拼出本地化的句子
  message: string              // 后备：界面不认识这个 code 时才显示
  reasons?: string[]           // relay_refused 时，Relay 给出的逐条理由（原样显示，去掉 "remote:" 前缀）
}
```

- `bad_signature`（并入 `relay_refused`）：签名不对，通常说明设备钥匙和座位表不一致。
- `forbidden`：引擎预检就拦下了（例如伙计想改 `members.toml`）。没有提交，什么都没发生。
- `relay_refused`：提交已经在本地，但 Relay 拒绝了推送。引擎把这次本地提交撤回到推送前的状态，**用户写的内容保留在草稿里**，不会丢。
- `conflict`：别人先推送了。引擎会自动拉取并在最新状态上重放一次；只有重放仍然失败时才返回这个错误。

---

## 6. 开发与测试环境

- **打包**：用 GitHub Actions 自动构建 Windows（.msi）、macOS（.dmg）和 Linux（.AppImage）安装包，不依赖任何人的电脑。
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

| 步骤 | 1 号（界面） | 2 号（引擎 + Relay） |
|---|---|---|
| 1 | 定义 `Backend` 接口（第 4 节），现有假数据放到 `FakeBackend` 后面，视觉不变；导出 `fixtures/houses.zh.json` | 新建 `client/` crate：读取命令和第 1 节的格式解析（直接读本地 git 副本） |
| 2 | 实现 `HttpBackend`（开发期，本机 7377 端口）；把 `boss/heir` 映射成 `host/steward` | 写入、签名、同步、预检、被拒后撤回；`serve` 子命令 |
| 3 | 入座界面（只问名字、粘贴邀请码）；"未同步""已离座""被拒绝"三种状态 | Relay：签名检查、附件仓库、邀请码（INVITE.md）；`dev-stack.sh`；端到端测试 |
| 4 | Tauri 外壳和 `TauriBackend`；**桌面版和 Android 一起打包**（Tauri 2 mobile）；CI 产出 Windows 安装包和 Android APK | 联调、修复问题；一人多设备（座位表支持多把钥匙、配对码） |

**依赖库**：引擎可以用第三方库，每加一个都要在 PR 里写明理由。git 操作用 `git2`（libgit2），**禁止调用外部 git 程序**，否则 Android 上无法运行；同时要加一组对照测试，确认引擎写出的提交，能被 Relay（系统 git）原样接受。签名用 `ssh-key` crate 生成 SSHSIG 格式，与 git 的 ssh 签名兼容。`core/` 保持零依赖。

**Relay 部署要尽量简单（用户要求）**：
- 目标：一台最便宜的 VPS，**一条命令装好**：`curl -fsSL <地址>/install.sh | sh`，或者 `docker run -d -p 2222:22 -v teahouse:/var/lib/teahouse teahouse/relay`。
- 安装脚本负责全部事情：建系统用户、装 sshd 配置、systemd 服务、每日检查，最后打印出第一个"开茶馆"的邀请码。
- 升级：`teahouse-relay upgrade`。备份：`teahouse-relay backup --to <路径或 S3>`。
- 以后提供"茶馆托管"（像 Obsidian Sync）：不想自己运维的人，直接用别人运营的 Relay，随时可以带着正本迁走。

---

## 8. 待定（需要用户或评审拍板）

1. **已定（用户 10-19）**：客户端优先；Relay 是会合点和执法者；用户不接触密钥；靠邀请码入座；不做账号密码；图片和附件不进 git，只放引用；提交签名；Relay 部署要尽量简单。
2. **"法"的修订，评审方起草后交用户确认**：
   - 签名检查：Relay 只接受作者是座位表成员、并且用该成员钥匙签名的提交（Relay 自己代写的提交，由 Relay 的钥匙签名，并注明"代写"）。
   - 附件：`*.ref.md` 引用文件的格式检查；git 里禁止出现超过 1 MiB 的二进制文件。
   - **编号唯一**（2 号提议，评审同意）：main 上每个编号最多对应一个 `problems/<id>-*.md`。
   - 座位表支持一人多把钥匙（多设备）。**优先级提前**：手机和电脑同步需要它，随第 4 步一起做。
3. **以后**：公开页面渲染（老板把某个问题或成果标成"公开"，Relay 把它渲染成网页，可以发链接给任何人看，像 Obsidian Publish）；网页窗口；手机 App。
4. 图片缩略图：先不做，直接显示原图。
5. 茶客离线时写好的提案，同步时发现自己已经离座了：导出成文件交还给他，绝不悄悄丢掉。
