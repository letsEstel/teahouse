# 茶馆 · 前端（M0 视觉原型）

只有前端，数据全部是假数据（`src/data/`，三间茶馆：梧桐里三号院、康德读书会、爸妈的老房子）。M1 会在这个目录里加上 `src-tauri/`。

```bash
npm install
npm run dev          # http://localhost:5173
npm run check        # 类型检查
```

路由：`#/<茶馆>/<界面>`，茶馆是 `wutong` / `kant` / `family`。

| 界面 | 路由 |
|---|---|
| 茶馆首页 · 座位 | `#/wutong/house` |
| 公告板 | `#/wutong/board` |
| 问题列表 / 看板 | `#/wutong/problems` / `#/wutong/problems/kanban` |
| 问题详情 | `#/wutong/problem/0001` |
| 提案列表（窄窗口是单独一页） | `#/wutong/proposals` |
| 提案审阅 | `#/wutong/proposal/p1` |
| 发起提案（茶客） | `#/kant/propose`、`#/kant/propose/0001`、`#/kant/propose/new` |

URL 参数：`?lang=zh|en|de` 指定语言（否则用上次的选择或系统语言；德语暂时只是附带，缺的词条退到英语）；`?theme=light|dark|auto` 指定外观（默认跟随系统）；`?tour=off` 关掉首次引导，`?tour=1|2|3` 直接显示某一步。

三间茶馆里"你"的角色不同：梧桐里是老板，康德读书会是茶客，爸妈的老房子是伙计，用来检查每种角色看到的主要动作。

## 目录

- `src/i18n/` 翻译：`zh.ts` / `en.ts` / `de.ts`，界面文字一律用 `t('键')`；日期时间用 `when()` / `date()` 本地化。
- `src/styles/base.css` 颜色语义表、断点、按钮样式。
- `src/lib/fit.ts` 超大标题按容器宽度缩小，不换行。

## 截图（每轮评审都用）

```bash
npm run shoot -- 02-公告与提案
```

构建 → 本地预览 → 截图写到 `/mnt/c/Users/andmi/Documents/Elysium/茶馆/截图/<轮次>/<界面>_<宽度>_<语言>.png`。
默认只保存 1366×768 中文一组（含夜间模式，不超过 20 张）；溢出检查照样对 3 种尺寸 × 中英全部组合运行，结果写进同目录的 `检查.md`。
`--full` 保存全套，`--de` 把德语也纳入检查，`--only=wutong-house,kant-propose` 只跑一部分，`--dpr=2` 出高清图。

WSL 里浏览器缺 `libnss3` / `libnspr4` 又没有 sudo 时，先运行一次 `scripts/setup-browser-libs.sh`（只解压到 `~/.cache/teahouse/chrome-libs`，不改系统）。
