# 公开围观 · Public read-only teahouses（提案）

> 状态：**用户已同意方案（2026-10-26），第 3 节的法则文字待评审方审定**。2 号起草，2026-10-26。决定见第 8 节。
> 背景：用户问"一间很重要的茶馆，上千人围观，要每个人都看到茶馆里的每一步，怎么办"，并决定：**先只做一个一键开关**，细的选项以后再拆。

## 1. 要满足的事

1. **做事的人和看的人分开。** 写入照旧只有座位上的人（≤ 15），法则一条不变。围观的人只读、人数不限、不用装软件、不用钥匙，用浏览器打开一个网址就能看。
2. **一个开关，含义简单：打开 = 这间茶馆的 `main` 公开。** main 上有什么就公开什么：问题、成果、留言、公告、每一步的历史和签名、没被删除的附件，名字按座位表显示。提案不在 main 上，所以不公开，采纳后自然出现。
3. **开关本身也是历史里的一步。** 只有老板能开关；所有成员都看得到是谁、在什么时候打开的；Relay 自己打不开，也关不掉。
4. **上千人看，服务器不吃力。** 围观的人只取静态文件，可以放在任何静态服务器或 CDN 后面；写入用的 ssh 那条路不受影响。
5. **每一步都可以验证。** 公开的内容带着原来的签名；谁都可以把完整历史克隆走，自己核对、自己留一份。

## 2. 开关：座位表里的一行

```toml
version = 1
[teahouse]
name = "茶馆开发组"
public = true          # 公开围观；没有这一行 = 不公开
```

- 改座位表本来就只有老板能做（法则第 8 条），所以开关天然只属于老板，提交由老板的设备钥匙签名。
- 引擎新命令 `setPublic { house, public: true | false }`（只有老板能用）：在**同一次推送**里写两个提交：
  1. 座位表加上或去掉 `public = true`（`Problem: meta`，`Act: seat`）；
  2. 在公告板贴一条"本茶馆已公开围观 / 已停止公开"（`Problem: board`，`Act: notice`），让每个成员都看到。
- 界面（1 号）：老板的"座位管理"里放一个开关；打开前确认一次，写明"打开后，任何拿到网址的人都能看到 main 上的全部内容和历史；关掉只能停止以后的公开，已经被看过、克隆走的内容收不回来"。打开后，每个人的界面顶部都显示"公开围观中"。
- **兼容**：旧版本的客户端不认识 `public` 这一行，会拒绝整张座位表（和 `kind = "machine"` 一样）。所以第一次打开之前，所有成员的客户端都要先更新。

## 3. 法则（拟加入规则表，第 20 条；第 1 条加一句）

| # | 法则 |
|---|---|
| 1（补充） | ……没有座位的人看来，不存在的茶馆和没有自己座位的茶馆完全一样，**公开围观的茶馆除外**：它的 `main` 任何人都能经 HTTPS 读取（第 20 条）。 |
| 20 | 座位表 `[teahouse]` 里写着 `public = true` 的茶馆是**公开围观**的（只有老板能改，第 8 条）。此后每一次被接受的推送之后，Relay 都会经 HTTPS 只读地发布：`main` 的完整历史（一个任何人都能克隆的 git 仓库），`main` 引用的、没被删除的附件，以及供网页显示的同一份数据。提案、邀请、配对码、审计日志和其他一切照旧不公开。`public` 被去掉后，Relay 撤下已经发布的内容。写入的规矩不变。 |

英文（RELAY.md 用）：
> 20 | A teahouse whose chart says `public = true` under `[teahouse]` is **public** (only the host changes it, law 8). After every accepted push the relay then publishes, read-only over HTTPS: `main` with its whole history (a git repository anyone may clone), the attachments `main` references that are not purged, and the same data as static files for the web page. Proposals, invitations, pairing codes, the audit log and everything else stay private. When `public` is removed, the relay withdraws what it published. Writing is unchanged.

## 4. Relay 发布什么

只在 `post-receive` 里做：只有通过了 pre-receive 全部法则的提交才会被发布。

```
<relay root>/public/<茶馆 id>/
  git/                    只含 main 的裸仓库（git update-server-info，"dumb HTTP"：
                          静态文件就能 git clone，CDN 能缓存）
  blobs/<sha256>          main 引用的附件（purged 的不在）
  data/house.json         和引擎的 house 命令同样格式（CLIENT-API 第 3 节）
  data/problems/<id>.json 和 problem 命令同样格式
  data/archive.json       和 archive 命令同样格式
  head                    最新一步：提交 id 和时间（几十字节，围观页每 10 秒读一次）
```

- `data/` 由 `teahouse-client publish --from <public git> --to <data 目录>` 生成：用的就是引擎自己的读取代码，数据格式和界面拿到的完全一样，Relay 不另写一套。Relay 的发布包里本来就可以带上这个程序。
- 每次推送只重新生成有变化的问题（看这次推送改了哪些路径），所以问题再多，一次发布也很快。
- **撤下**：座位表里没有 `public = true` 了，post-receive 就删掉整个 `public/<茶馆 id>/`。附件被 `purgeBlob` 删除时，公开目录里的那份同时删掉。
- 公开目录里**永远不会有**：`refs/proposals/*`、邀请、配对码、审计日志、Relay 自己的钥匙。

## 5. 围观的人怎么看

- **网址**：`https://<域名>/<茶馆 id>/`。
- **网页**：1 号的界面做一个"围观模式"的静态构建：只读，没有任何写入按钮，数据从 `data/*.json` 读。看到的和成员看到的一样：问题、成果、留言、每一步的经过。
- **实时**：网页每 10 秒读一次 `head`，有新的一步就刷新那一页。一千人围观大约是每秒 100 次几十字节的请求，对静态服务器不算负担。
- **完整历史**：`git clone https://<域名>/<茶馆 id>/git` 能拿到全部历史和签名，可以用 `git log --show-signature` 自己核对。
- **以后（不在这次范围）**：浏览器里验签（WEB.md 的 wasm 引擎），每一步旁边显示"签名有效：王阿姨"；镜像；更细的公开选项（不公开留言、附件、用化名……）。

## 6. 部署

- 公开目录是纯静态文件，由服务器上现有的网页服务器提供。
- **茶馆开发组的服务器（122.10.118.71，香港，不需要 ICP 备案）**：上面已经有 Apache 2.4 在 80/443 端口上提供 `jiankangzhao.com` 的 HTTPS 网站，DNS 在 Cloudflare。所以：
  1. 在 Cloudflare 加一条 DNS 记录：`teahouse.jiankangzhao.com` → A → `122.10.118.71`；
  2. Apache 加一个虚拟主机 `teahouse.jiankangzhao.com`，`DocumentRoot` 指向 `<relay root>/public/`，只读，不执行任何脚本；
  3. 给子域名签证书（certbot，和主站一样）；
  4. 可选：Cloudflare 的代理（橙色云朵）打开后，静态文件由 Cloudflare 缓存，围观的人再多也基本不到服务器。`head` 文件要设成不缓存，或者最多缓存几秒，否则实时刷新会变慢。
- `install.sh --public <域名>`：检测到已有 Apache 时，写入上面的虚拟主机配置并重新加载 Apache；没有网页服务器时才装 Caddy（自动签发证书）。不加这个选项时，Relay 照样生成公开目录，只是没有对外的网址。
- 这条路只读、没有登录、没有令牌，所以不会像 WEB.md 里的网页写入那样多出 CSRF、偷令牌这类风险。

## 7. 测试计划

- core：`public = true` 的解析和校验；只有老板的提交能改它（沿用第 8 条的测试）。
- Relay 端到端测试：
  - 打开后推送 → 公开目录里有 main、附件和 `head`，没有提案；
  - `git clone` 公开仓库，历史和签名与 Relay 上的一致；
  - 删除附件 → 公开目录里的那份也没了；
  - 关掉 → 公开目录被删除；
  - 伙计或茶客改 `public` → 被拒（第 8 条）。
- 引擎：`setPublic` 的两个提交在同一次推送里；茶客、伙计调用返回 `forbidden`（管理类命令，不会自动转成提案）。
- `sshd-test`：经真实 sshd 打开开关，然后经 HTTP 克隆公开仓库。

## 8. 已定的事（用户，2026-10-26）

1. **开关放在座位表里**（`[teahouse] public = true`），由老板签名；第一次打开之前，所有成员的客户端先更新。
2. **`data/` 由 Relay 调用 `teahouse-client publish` 生成**，数据格式和界面完全一致；Relay 的发布包里一起带上这个程序。
3. **域名：`teahouse.jiankangzhao.com`**（用户的个人域名下的子域名，2026-10-26）。服务器在香港，不需要 ICP 备案；用现有的 Apache 和证书方式（第 6 节）。teahouse.com、.app、.io、.org、.dev 都已被注册，以后有了正式域名再换；公开网址发出去以后最好不再换，所以正式公开前要定下来。

**还要评审方审定**：第 3 节第 1 条的补充和第 20 条的文字。
