# 网页版可行性 · Can the teahouse run in a browser?

> 状态：可行性分析，2 号，2026-10-20。**只有文档，不写代码。**
> 回答四个问题：git 在浏览器里怎么跑；Relay 的 HTTPS 入口怎么认人、怎么保证和 SSH 走同一套法；浏览器存储会不会被清掉；要不要因此换掉 libgit2。
> 第 1、4 节里关于 gitoxide 的判断查过资料（见文末来源）；标着"待验证"的地方，需要先做一个原型（第 5 节）才能确定。

## 结论先说

| 问题 | 建议 |
|---|---|
| 浏览器里的 git | **用 isomorphic-git**，引擎的其余部分（格式解析、视图、同步规则、预检）照样用 Rust，编译成 wasm，跑在 Web Worker 里。**不把引擎迁到 gitoxide**：gitoxide 到现在还不能 push。 |
| HTTPS 身份 | **签名挑战**：设备钥匙签 Relay 给的随机数，换一个短期令牌。之后的 git 请求进入的是**同一个 `shell` 授权和同一个 pre-receive**，法只写一遍。 |
| 存储被清 | **会**：Safari 在用户 7 天没访问时清掉网页存储；所有浏览器在空间紧张时都可能清。所以浏览器里的副本只当缓存，正本在 Relay；写完立刻推；钥匙丢了用配对码补回。网页版只能做"轻量窗口"，不能当主力设备。 |
| 换掉 libgit2 | **现在不换。** 只在引擎里收一道"git 后端"接口：本机用 libgit2，网页用 isomorphic-git。等 gitoxide 能 push 了，再评估要不要让所有平台共用它。 |

## 1. 浏览器里跑 git：isomorphic-git，还是 gitoxide 编译成 wasm

**gitoxide（gix）**：纯 Rust，原则上能编译成 wasm，但：
- **还不能 push。** 它自己的进度表把 "push and self-contained clone/fetch over file:// and ssh://" 列为还缺的功能；第三方评测和最近的项目讨论也都说 push 没做。茶馆的每一次写入最终都是 push，没有 push 就不能用。
- 对 wasm 的支持只有零星的早期工作（2023 年给 git-pack 加过初步的 wasm32 支持），没有官方说明。网络层还得自己接到浏览器的 `fetch` 上。
- 签名提交（"create signed commits and tags"）也列在进度表上，没看到已完成的说明（待验证）。

**isomorphic-git**：纯 JS，浏览器里成熟的 git 实现：
- clone、fetch、push 都走 **smart HTTP**，所以需要 Relay 提供 HTTPS（第 2 节）。不支持 ssh。
- 存储可以放在 IndexedDB（LightningFS、ZenFS）或 OPFS 上。
- `commit` 和 `merge` 有 `onSign` 回调，可以接我们自己的 SSHSIG 签名：Relay 用 `git verify-commit` 判断，只认格式，不管是谁生成的（待验证：签名所覆盖的提交内容，要和 git 的算法逐字节一致。我们的 libgit2 实现已经验证过这一点，isomorphic-git 要在原型里验证）。
- 跨域：网页和 Relay 同源时不需要代理。所以网页应该由 Relay 自己提供，或者 Relay 加上 CORS 头。

**建议的结构**（Web Worker 里跑，不卡界面）：
```
界面（Svelte，同一套）──postMessage──▶ Worker
                                       ├─ 引擎（Rust → wasm）：格式解析、视图、同步规则、预检、签名
                                       └─ git 后端（isomorphic-git + OPFS）：对象、引用、fetch、push
```
- 引擎今天只有三处用到 libgit2：`git.rs`（读）、`commit.rs`（写）、`sync.rs`（同步）。在这三处收一个 `GitBackend` 接口：本机实现用 libgit2，网页实现调用 isomorphic-git。
- **最难的一点**：isomorphic-git 是异步的（Promise），而引擎现在是同步的。网页版要么把引擎这一层改成 async，要么在 Worker 里用同步的 OPFS 访问句柄、自己读对象。原型要先回答这个问题（第 5 节）。
- **设备钥匙**：浏览器的 WebCrypto 可以生成**不可导出**的 Ed25519 钥匙，存进 IndexedDB。"钥匙不出设备"在浏览器里也做得到。SSHSIG 只是在 Ed25519 签名外面套一层固定格式，我们自己拼（待验证：各浏览器对 WebCrypto Ed25519 的支持程度）。

## 2. Relay 的 HTTPS 入口：怎么认人，怎么保证同一套法

**认人：签名挑战**（不用密码，不用账号，也不用浏览器证书）：
1. 网页 `POST /auth/challenge`，Relay 返回一个随机数（一次性，60 秒内有效）。
2. 网页用设备钥匙对它签名（SSHSIG，命名空间 `teahouse-auth`，和提交签名分开），连同公钥一起交回 `POST /auth/token`。
3. Relay 校验签名（和第 13 条用同一套 ssh-keygen 检查），确认这把钥匙在某间茶馆有座位，发一个**短期令牌**（例如 10 分钟，只绑定这把钥匙），以后的请求就带着它。
4. 令牌只代表"这把钥匙"，不代表角色。角色每次都按当时的座位表现查，和 SSH 一样：离座立刻生效。

为什么不用别的办法：
- 密码：身份属于人、不能被 Relay 掌握（第 2 节原则）。
- WebAuthn / 通行密钥：它签的是另一种格式的数据，换不成 git 能验证的提交签名；设备钥匙还是得有一把。
- 客户端 TLS 证书：浏览器里体验很差，手机上几乎没法用。

**同一套法**：HTTPS 入口不另写一套检查，只做两件事：认出钥匙，然后把请求交给现有的代码。
- git 请求交给 `git http-backend`，环境变量和 SSH 时完全一样：`TEAHOUSE_KEY`、`TEAHOUSE_HOUSE` 由入口设置，客户端无法自己设置。push 时 git 运行的是**同一个 pre-receive hook**，法一行都不用重写。
- 进门检查（有没有座位、读还是写）调用 `shell.rs` 现有的函数，不复制。
- 附件和邀请（`blob-*`、`invite`、`redeem`）做成 1:1 的 HTTPS 接口，背后是同一组函数（`blobs.rs`、`invites.rs`）。
- 审计日志里，同一个事件加一个字段 `via: ssh | https`。
- 部署：`teahouse-relay http --listen 127.0.0.1:8080`，前面放 Caddy 自动签发证书。安装脚本可以加一个 `--https <域名>` 选项。
- 端到端测试：现有的推送测试（签名、编号、留言、提案……）参数化一次，同一组用例经 SSH 和 HTTPS 各跑一遍，结果必须完全一样。

**新的风险**：HTTPS 是公开的网页入口，比 SSH 多一类攻击面（CSRF、XSS 偷令牌）。
- 令牌只放在 Worker 的内存里，不放 cookie，也不放 localStorage。
- 设置严格的 CSP，网页和 Relay 同源。
- 写入接口只接受 `Content-Type: application/x-git-*` 和 JSON（和现在 `serve` 的做法一样）。

## 3. 浏览器存储会不会被清掉

**会。** 具体是：
- **默认是"尽力而为"（best-effort）**：磁盘空间紧张时，Chromium 会先清最久没用的网站。调用 `navigator.storage.persist()` 可以申请"持久"，持久的数据只在用户手动清除时才删；Chrome 和 Safari 根据用户和网站的互动自动批准或拒绝，不弹窗；Firefox 会问用户。
- **Safari 的 7 天规则**（ITP）：在 Safari 里**连续 7 个"使用 Safari 的日子"**没有访问某个网站，这个网站的 IndexedDB、localStorage、Service Worker 等就会被清掉。**加到主屏幕的网页应用另算**，苹果说不会这样清它们的数据。
- **Safari 17** 提高了配额（浏览器里最多可用磁盘的 60%），持久模式的网站不会被清。但有报告说 Safari 17 里 `persist()` 总是返回 false（WebKit bug 271401，有没有修好待验证）。

**对茶馆意味着什么**：
1. **只有进了正本的才算数**（CLIENT-API 第 0 节第 2 条）在网页上更重要：浏览器里的副本只是缓存，被清了就重新克隆。
2. **没推送的写入有丢失风险**：网页版每写一条都立刻推送；离线时明确提示"还没送出"；草稿另外存一份。
3. **设备钥匙也可能被清**：清了，这台浏览器就是一台"新设备"，用配对码重新加入（DEVICES.md），不需要找老板。**所以一人多设备是网页版的前提。**
4. 结论：网页版做成**轻量窗口**（看、留言、提提案、老板审批），主力设备仍然是桌面版和手机 App。在手机上，建议把网页加到主屏幕（不受 7 天规则影响）或者直接用 App。

## 4. 要不要换掉 libgit2，让所有平台共用一个引擎

**现在不换。**
- libgit2 已经在 Windows、Linux、Android 上跑通（CI 的打包和真实 sshd 测试），macOS 也可以。签名、push、ssh 都已验证。
- gitoxide 不能 push，连 file:// 和 ssh:// 上自包含的 clone、fetch 都还在进度表上。现在换，等于自己去写 send-pack 和 ssh 传输，风险最大的部分正好落在我们身上。
- 网页版本来就要另一个 git 后端（isomorphic-git 走 HTTPS），换成 gitoxide 也不会让网页版更早上线。

**现在就该做的一步**：在引擎里收一道 `GitBackend` 接口（第 1 节）。这本身是好的结构，也给以后留了门：
- 网页：isomorphic-git 实现。
- 将来：gitoxide 的 push 和 ssh 成熟以后，可以先在测试里拿它和 libgit2 跑同一组对照测试（像现在 libgit2 对系统 git 那样），通过了再决定是否全面替换。换掉的好处是不用再附带 C 代码和 vendored OpenSSL（安卓和 Windows 打包会更简单）。

## 5. 下一步：先做一个原型，验证三件事

只做验证，不进主分支：
1. **签名兼容**：isomorphic-git 在浏览器里 clone 一间真实的茶馆（经一个临时的 HTTPS 入口），用 WebCrypto 的 Ed25519 生成 SSHSIG 签名提交，push 上去，被 Relay 的第 13 条接受。
2. **同步模型**：在 Worker 里，用 wasm 引擎加异步的 git 后端完成一次"读问题 → 留言 → 推送"，确定是把引擎改成 async，还是走同步的 OPFS。
3. **存储**：在 Chrome、Firefox、Safari（包括 iOS 主屏幕应用）上记录 `persist()` 的结果。

## 来源
- gitoxide 进度表（push、签名提交）：https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md
- gitoxide 评测（push 未完成）：https://hysenlabs.com/projects/gitoxidelabs-gitoxide
- 近期项目讨论（gitoxide 0.87 仍无 push）：https://github.com/jwh3times/causet/issues/286
- gitoxide 早期 wasm32 工作：https://github.com/GitoxideLabs/gitoxide/pull/735
- isomorphic-git 签名回调（onSign）：https://tessl.io/registry/tessl/npm-isomorphic-git/files/docs/commit-operations.md
- isomorphic-git 的文件系统（ZenFS、LightningFS）：https://isomorphic-git.org/docs/en/next/fs
- isomorphic-git 在 OPFS 上运行：https://cdn.jsdelivr.net/npm/memfs@4.57.2/demo/git-opfs/README.md
- isomorphic-git 与 CORS：https://isomorphic-git.org/docs/en/gitRemoteHTTP 、https://isomorphic-git.org/blog/2018/04/07/gogs-adds-cors-headers-for-isomorphic-git
- Safari 7 天规则：https://mjtsai.com/blog/2020/03/26/safari-13-1-third-party-cookie-blocking-and-7-day-script-writeable-storage/ 、https://searchengineland.com/what-safaris-7-day-cap-on-script-writeable-storage-means-for-pwa-developers-332519
- WebKit 存储策略（Safari 17）：https://webkit.org/?p=14403
- Safari 17 的 persist() 问题：https://bugs.webkit.org/show_bug.cgi?id=271401
- 配额与清除规则：https://developer.mozilla.org/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria 、https://web.dev/articles/storage-for-the-web
