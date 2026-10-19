# 茶馆 · 前端（M0 视觉原型）

只有前端，数据全部是假数据（`src/data/`，六间茶馆：梧桐里三号院、康德读书会、爸妈的老房子、三人创业团队、旧鞋带乐队、液滴论文小组）。M1 会在这个目录里加上 `src-tauri/`。

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

（`scripts/engine-bridge.mjs`：`/v1/<命令>` → `teahouse-client --root <根目录> call <命令> <参数>`；附件经 `/v1/__file` 读取，只允许读根目录里面的文件。）
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
