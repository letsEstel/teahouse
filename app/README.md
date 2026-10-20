# 茶馆 · 客户端（界面 + Tauri 外壳）

界面（Svelte）在 `src/`，Tauri 外壳在 `src-tauri/`，引擎是仓库根目录的 `client/`（`teahouse-client`）。
在浏览器里打开时用假数据（`src/data/`，六间示例茶馆：梧桐里三号院、康德读书会、爸妈的老房子、三人创业团队、旧鞋带乐队、液滴论文小组）；打包成应用后接真引擎。

## 安装包（Windows、安卓）

不用在自己电脑上编译：每次推送到 GitHub，`.github/workflows/app.yml` 会自动打包。
在仓库的 Actions → **app** → 最新一次运行 → Artifacts 里下载：

- `teahouse-windows`：Windows 安装程序（`Teahouse_0.1.0_x64-setup.exe`），双击安装。
- `teahouse-android`：安卓 APK（`teahouse-arm64.apk`，适用于 64 位 ARM 手机，几乎所有近几年的安卓手机都是）。
  传到手机上点开安装；第一次需要在系统设置里允许"安装未知来源的应用"。
  所有 APK（CI 的、nightly 的、本机 `scripts/build-android.sh` 打的）都用同一把固定的签名钥匙签，新版本直接覆盖安装。
  **不要卸载**：卸载会删掉这台设备的钥匙，也就丢了座位。钥匙放在仓库外（本机 `~/.config/teahouse/android.jks`，
  CI 里是仓库 Secrets `ANDROID_KEYSTORE_B64` / `ANDROID_KEYSTORE_PASSWORD`）；CI 没有这两个 Secrets 就直接失败，从不临时生成钥匙，并且会核对证书指纹。

应用里还没有茶馆时，打开就是入座页；没拿到邀请的话，可以点"先看看示例茶馆"，用假数据看看（不会同步，也不会离开这台设备），侧栏里随时可以退出示例。

自己编译（需要 Rust、Node 22，以及 Tauri 的系统依赖，见 https://tauri.app/start/prerequisites/）：

```bash
npm ci
npx tauri dev                    # 桌面窗口，接真引擎
npx tauri build --bundles nsis   # Windows 安装包
npx tauri android init && npx tauri android build --apk --target aarch64   # 安卓（需要 Android SDK 和 NDK）
```

```bash
npm install
npm run dev          # http://localhost:5173
npm run check        # 类型检查
```

路由：`#/<茶馆>/<界面>`，茶馆是 `wutong` / `kant` / `family` / `startup` / `band` / `lab`。

| 界面 | 路由 |
|---|---|
| 茶馆首页 · 座位 | `#/wutong/house` |
| 公告板 | `#/wutong/board` |
| 问题列表 / 看板 | `#/wutong/problems` / `#/wutong/problems/kanban` |
| 问题详情 | `#/wutong/problem/0001` |
| 提案列表（窄窗口是单独一页） | `#/wutong/proposals` |
| 提案审阅 | `#/wutong/proposal/p1` |
| 发起提案（茶客） | `#/kant/propose`、`#/kant/propose/0001`、`#/kant/propose/new` |

URL 参数：`?lang=zh|en|de` 指定语言（否则用上次的选择或系统语言；缺的词条退到英语）；`?theme=light|dark|auto` 指定外观（默认跟随系统）；`?tour=off` 关掉首次引导，`?tour=1|2|3` 直接显示某一步。

各间茶馆里"你"的角色不同（老板：梧桐里、旧鞋带乐队；伙计：爸妈的老房子、三人创业团队；茶客：康德读书会、液滴论文小组），用来检查每种角色看到的主要动作。

## 数据从哪里来（CLIENT-API 第 7 节第 1 步）

界面只通过 `src/api/backend.ts` 里的 `api(命令名, 参数)` 读写数据，命令和数据模型照 `docs/CLIENT-API.md`（类型在 `src/api/types.ts`）。
现在接的是 `FakeBackend`（`src/api/fake.ts`）：把 `src/data/` 里的六间示例茶馆包装成合同的形状，档案里的文件原文按第 1 节的格式生成。
第 4 步换成 `TauriBackend`；界面代码不用改。

**接真引擎（第 2 步，开发期）**：设 `VITE_BACKEND=local`，界面改用 `HttpBackend`，`POST http://127.0.0.1:7377/v1/<命令>`（引擎的 `serve`）。
引擎的 `serve` 还没做好时，可以让 Vite 开发服务器把请求转给引擎的命令行：

```bash
TEAHOUSE_ENGINE_CLI=<teahouse-client 的路径> TEAHOUSE_CLIENT_ROOT=<引擎根目录> npm run dev:engine
```

（`scripts/engine-bridge.mjs`：`/v1/<命令>` → `teahouse-client --root <根目录> call <命令> <参数>`；附件经 `GET /v1/blob?house=&id=` 读取（合同第 4 节），只返回根目录里面的文件。）
写入失败时，页面顶部出现一条按错误代码本地化的提示，输入框里写的内容保留；连不上引擎时显示"打不开这间茶馆"和重试按钮。

```bash
npm run export-fixtures   # 导出 ../fixtures/houses.zh.json，给引擎种真的 git 仓库
```

## 目录

- `src/api/` 合同类型、`Backend` 接口、FakeBackend、界面用的小工具（`view.ts`）。
- `src/data/` 示例茶馆的假数据（三种语言），只有 FakeBackend 读它。
- `src/i18n/` 翻译：`zh.ts` / `en.ts` / `de.ts`，界面文字一律用 `t('键')`；日期时间用 `when()` / `date()` 本地化。
- `src/styles/base.css` 颜色语义表、断点、按钮样式。
- `src/lib/fit.ts` 超大标题按容器宽度缩小，不换行。

## 线上预览（每轮结束都更新）

```bash
npm run preview:publish
```

构建一份精简的静态站点，复制到 `/mnt/c/Users/andmi/Documents/Elysium/茶馆/预览/`（约 4 MB）：
字体只保留界面里出现过的字，脚本是普通的 defer 脚本，所以直接打开 `index.html` 也能用。

## 截图（每轮评审都用）

```bash
npm run shoot -- 05-预览与表格
```

构建 → 本地预览 → 截图写到 `/mnt/c/Users/andmi/Documents/Elysium/茶馆/截图/<轮次>/<界面>_<宽度>_<语言>.png`。
默认只保存 1366×768 中文一组（含夜间模式和个别德文图，不超过 20 张）；溢出检查照样对 4 种尺寸（含手机 390 宽）× 中英德全部组合运行，结果写进同目录的 `检查.md`。
`--full` 保存全套，`--only=wutong-house,kant-propose` 只跑一部分，`--dpr=2` 出高清图。

WSL 里浏览器缺 `libnss3` / `libnspr4` 又没有 sudo 时，先运行一次 `scripts/setup-browser-libs.sh`（只解压到 `~/.cache/teahouse/chrome-libs`，不改系统）。
