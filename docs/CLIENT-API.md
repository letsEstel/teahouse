# 茶馆客户端引擎 · 接口约定（v0.1 草案）

> 状态：评审方起草，2026-10-19；**第二版**按用户的原始架构修订：引擎和 Relay、git 一起部署在服务器上，客户端只是界面。这是界面（1 号）和客户端引擎（2 号）之间的合同。
> 两边都按这份文档开发；需要改动时，先在 PR 里改这份文档，再改代码。
> 茶馆的"法"仍以 `docs/RELAY.md` 第 3 节为准，本文只规定**内容格式**和**引擎接口**。

---

## 0. 一张图

```
 电脑浏览器 / 手机浏览器 / （以后）Tauri 壳里的同一个网页界面
        │  HTTPS + JSON，设备钥匙签名登录
        ▼
┌──────────────────────── 服务器（一台 VPS）────────────────────────┐
│  teahouse-server（引擎 + HTTP 接口，Rust）                          │
│    读：直接读裸仓库的历史 → JSON                                     │
│    写：在临时工作区里生成提交 → 以该成员的身份推入 → pre-receive 检查  │
│        （和外部 git 推送走同一道关，"法"只在一处执行）                  │
│  teahouse-relay（守门：法、审计）＋ houses/<id>.git（裸仓库）           │
│  sshd：只给"镜像备份"用（老板、伙计在自己电脑上保留完整副本）           │
└────────────────────────────────────────────────────────────────────┘
```

**原则**
1. **客户端不碰 git。** 用户的设备上只有界面；git 只在服务器上。界面永远看不到 commit、hash、branch、路径。（这是茶馆最初定义里的第五条：Relay 是唯一连接 git 和用户的地方。）
2. **"法"只在一处执行。** 引擎替用户写入时，不绕过 Relay：它把提交以该成员的身份推入裸仓库，走同一个 pre-receive 钩子。引擎自己只做预检，目的是给出更早、更友好的提示。
3. **一套界面，处处可用。** 手机浏览器打开网址就能用，不需要安装 App。以后要做 App，就用 Tauri 包一层壳，界面代码完全相同。
4. **杀不死靠两样东西**：任何成员都能"带走整间茶馆"（导出完整历史）；老板和伙计在自己的设备上保留完整副本（现在用"镜像备份"，以后由原生客户端自带），所以伙计随时可以凭这份副本继承茶馆（`teahouse-relay adopt`）。

### 两种客户端（用户 10-19 定）

| | 网页版（现在做） | 原生客户端（以后） |
|---|---|---|
| 形态 | 浏览器打开网址，电脑和手机都能用 | 桌面或手机 App（Tauri） |
| git | 不碰 git，所有读写都经过服务器 | **设备里内置 git**，保存整间茶馆的完整副本 |
| 离线 | 不支持（断网时只把草稿存在本机） | 支持：离线照常读写，联网后自动同步 |
| 带走数据 | "带走整间茶馆"：手动导出下载 | 本来就在设备上，随时完整 |
| 适合 | 所有人，尤其是只用手机的人 | 老板、伙计，以及想要完整副本的人；**继承靠它** |

两者说同一种"茶馆语言"（第 1 节的内容格式），法都由 Relay 执行，所以可以混着用：王阿姨用手机网页，伙计用带完整副本的桌面客户端。原生客户端里的引擎，复用服务器引擎的同一份代码（`server/` 的核心逻辑会拆成一个库），只是把"推入本机的裸仓库"换成"提交到本机副本，再推送到 Relay"。在原生客户端出来之前，"镜像备份"（第 8 节第 5 条）先起到这个作用。

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

## 2. 身份与登录

**原则：身份属于人，不属于服务器。** 每台设备有一把 ed25519 **设备钥匙**，公钥写在 `members.toml` 里。服务器只核对公钥，不保存任何密码。所以服务器被关停、伙计在别处重开茶馆以后，每个人的身份原样有效，继承才成立。

**普通用户永远看不到"密钥"这个词。**
- **第一次打开网页或 App**：浏览器用 WebCrypto 生成一把**不可导出**的设备钥匙，存在本机（浏览器里是 IndexedDB，App 里是系统钥匙串）。界面上只问"你叫什么名字"。
  - 注：Ed25519 在 WebCrypto 里的支持已经比较普及；如果遇到不支持的浏览器，回退到 ECDSA P-256。`members.toml` 因此需要能接受 P-256 公钥，这属于"法"的改动，和多设备一起定稿（见第 8 节）。
- **入座**：粘贴老板发来的邀请码（见 `docs/INVITE.md`）。
- **登录**：挑战-签名。客户端请求 `POST /v1/auth/challenge`，得到一个随机数；用设备钥匙签名后交回 `POST /v1/auth/login`，换到一个会话令牌（HttpOnly Cookie，12 小时有效）。之后每个请求都带上这个令牌。
- **换设备、多设备**：在已登录的设备上点"添加一台设备"，得到一个短时有效的配对码，在新设备上输入。新设备生成自己的钥匙，旧设备的钥匙不需要导出。
- **高级选项**：懂行的用户可以直接用已有的 ssh 密钥做镜像备份（`git clone teahouse@relay:<house>`，只能读）。

---

## 3. 数据模型（服务器返回的 JSON）

字段名使用 camelCase，时间使用 RFC 3339 UTC，**用户写的内容都是普通字符串**（不分语言）；界面自身的文字仍然走 i18n。
角色使用 Relay 的名字：`host | steward | guest`。界面现在的 `boss | heir | guest` 在 `HttpBackend` 里做一次映射，以后统一改掉。

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
  version: string               // 当前 main 的版本号（不透明字符串），写入时用来防止覆盖别人的改动
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

传输：`POST https://<服务器>/v1/<命令名>`，请求体和返回体都是 JSON，需要先登录（第 2 节）。以后包成 App 时，仍然调用同一组 HTTP 接口。
所有命令都返回 `{ ok: true, data }` 或 `{ ok: false, error }`（见第 5 节）。图片和附件用 `upload`：先 `POST /v1/upload`（multipart）拿到一个上传编号，再把编号放进写命令。

### 读取

| 命令 | 参数 | 返回 |
|---|---|---|
| `houses` | — | `HouseSummary[]`（我有座位的茶馆） |
| `house` | `{ house }` | `House` |
| `problem` | `{ house, problem }` | `Problem` |
| `archive` | `{ house, view: 'dirs' \| 'problems' }` | `ArchiveNode` |
| `file` | `{ house, id }` | `{ node: ArchiveNode, text?: string, history: HistoryEntry[] }` |
| `download` | `{ house, id }` | 原始字节 |
| `export` | `{ house }` | 整间茶馆的 git bundle 加一份可直接阅读的 zip（"带走整间茶馆"） |

### 变化通知

`GET /v1/events`（SSE）：茶馆的 main 或提案有变化时，推送 `{ house, version }`，界面收到后重新读取。多人同时在线时，靠它让大家看到的内容保持最新。

### 写入（服务器生成提交，并通过同一道 pre-receive 检查）

| 命令 | 参数 | 谁能用 | 产生的提交 |
|---|---|---|---|
| `openProblem` | `{ house, title, body[] }` | 老板、伙计（茶客自动转为提案） | `open-problem` |
| `setStatus` | `{ house, problem, status, owner? }` | 老板、伙计 | `set-status` |
| `addWork` | `{ house, problem, kind, title, text? \| upload? }` | 老板、伙计（茶客自动转为提案） | `add-work` |
| `updateWork` | `{ house, id, text? \| upload? }` | 同上 | `update-work` |
| `note` | `{ house, problem \| notice, text }` | 所有人 | `note` |
| `postNotice` | `{ house, title, text }` | 老板、伙计（茶客自动转为提案） | `notice` |
| `propose` | `{ house, problem, title, reason?, changes: {id?, kind, title, text? \| upload?, delete?}[] }` | 所有人 | 提案分支 |
| `withdraw` | `{ house, proposal }` | 提案作者 | 删除提案分支 |
| `adopt` | `{ house, proposal }` | 老板 | `adopt`，同时删除提案分支 |
| `decline` | `{ house, proposal, reason? }` | 老板 | `decline`，同时删除提案分支 |
| `seat` | `{ house, add?: Member&{key}, remove?: id, role?: {id, role} }` | 老板 | `seat` |

**界面按角色分开设计，引擎再兜一层底。**
- 界面以 `House.me` 的角色为准，**按角色显示不同的界面**：老板有"座位管理"和"提案审阅"，别人看不到这两样；茶客看到的按钮直接写成"提议添加成果""提议一条公告"，点之前就知道这会交给老板。
- 引擎的"自动转为提案"只是兜底：如果茶客的请求还是走到了老板或伙计才能用的**内容类**写命令（`openProblem`、`addWork`、`updateWork`、`postNotice`），引擎不报错，而是改为发起提案，返回值带上 `{ becameProposal: true }`。这种情况通常只发生在角色刚变动、界面还没刷新的时候。
- **管理类命令绝不自动转换**：`seat`、`adopt`、`decline`、`setStatus` 只有对应角色能用，其他人调用一律返回 `forbidden`。

- 每个写命令都可以带上 `baseVersion`（读取时拿到的 `version`）。内容类修改（`updateWork` 等）如果期间有人改过同一个文件，返回 `conflict`，并附上对方改了什么，由用户决定怎么合并；不涉及同一个文件的写入，服务器会自动在最新版本上完成。
- 写命令的返回值：`{ version: string, becameProposal?: boolean, id?: string }`。

## 5. 错误

```ts
interface ApiError {
  code: 'unauthenticated' | 'not_found' | 'not_a_member' | 'forbidden' | 'invalid'
      | 'conflict' | 'relay_refused' | 'too_large' | 'rate_limited' | 'internal'
  message: string              // 给人看的一句话（中英双语，和 Relay 的风格一致）
  reasons?: string[]           // relay_refused 时，pre-receive 给出的逐条理由
  theirs?: DiffLine[]          // conflict 时，对方的改动
}
```

- `forbidden`：预检就拦下了，什么都没写入。
- `relay_refused`：预检没拦住、pre-receive 拦住了（说明两边规则不一致，应当当作程序缺陷修复）。什么都没写入；用户填写的内容由界面保留在草稿里。
- **网络断了**：界面把正在写的内容存进本机草稿，恢复联网后提示用户重新提交。v0.1 不做离线写入队列。

---

## 6. 开发与测试环境

- `scripts/dev-stack.sh`：一条命令启动整套环境
  1. 在临时目录里初始化 Relay
  2. 用 `fixtures/houses.zh.json` 把六间示例茶馆**种成真的 git 仓库**：按时间顺序逐条提交，提交时间和作者都照搬示例数据
  3. 启动 `teahouse-server --dev --port 7377`。`--dev` 模式下，用 `TEAHOUSE_DEV_ME=<成员id>` 直接以某个成员的身份登录，跳过签名
  4. 启动 `npm run dev`，设置 `VITE_BACKEND=http://127.0.0.1:7377`
- 测试：服务器对每个命令都有测试；再加一组端到端测试，在真实的 Relay 上跑完"开问题 → 添加成果 → 茶客提案 → 老板采纳 → 留言 → 导出"的完整流程。另外要专门测试：服务器写入的每一类提交，都能通过 pre-receive；伪造会话、越权调用全部被拒绝。

---

## 7. 分工与顺序

| 步骤 | 1 号（界面） | 2 号（服务器） |
|---|---|---|
| 1 | 定义 `Backend` 接口（第 4 节），现有假数据放到 `FakeBackend` 后面，视觉不变；导出 `fixtures/houses.zh.json` | 新建 `server/` crate（`teahouse-server`）：读取命令和第 1 节的格式解析；`--dev` 登录 |
| 2 | 实现 `HttpBackend`，用 `VITE_BACKEND` 切换；把 `boss/heir` 映射成 `host/steward` | 写入命令（临时工作区 → 以成员身份推入 → pre-receive）、`export`、SSE |
| 3 | 登录和入座界面（只问名字、粘贴邀请码），加上"已离座""被拒绝""网络断了"三种状态 | 挑战-签名登录、`dev-stack.sh`、种仓库、端到端测试 |
| 4 | 联调：在 `dev-stack` 上把六间茶馆全部走一遍，截图 | 联调：修复问题 |

**依赖库**：服务器可以使用第三方库，每加一个都要在 PR 里写明理由。建议：`axum` 加 `tokio`（HTTP 和 SSE），`serde`/`serde_json`，`ed25519-dalek` 和 `p256`（验签）。git 操作**继续调用系统的 git**，与 Relay 保持一致。`core/` 仍然保持零依赖。

**部署**：服务器和 Relay 同机部署，用同一个系统用户，配一个 systemd 服务（`teahouse-server.service`，沙箱化，只监听 127.0.0.1），前面用 Caddy 或 nginx 提供 HTTPS。部署文档写进 `docs/RELAY.md` 第 5 节。

---

## 8. 待定（需要用户或评审拍板）

1. **已定（用户 10-19）**：用户不接触密钥；入座靠邀请码；不做账号密码（理由见第 2 节）。
2. 图片缩略图：先不做，直接显示原图。
3. **"法"的修订，需要单独定稿**：`members.toml` 支持**一个成员拥有多把公钥**（多设备），以及接受 **P-256 公钥**（给不支持 Ed25519 的浏览器用）。
4. 邀请功能见 `docs/INVITE.md`（PR #2）。由于登录改走 HTTP，兑换邀请码也改为 HTTP 接口 `POST /v1/invite/redeem`；sshd 只留给镜像备份。
5. 镜像备份：老板和伙计在电脑上用什么方式开启（一个小工具，还是"镜像"页面上给出一条命令）。建议 v0.1 先给一条命令，以后再做成按钮。
