# 茶馆 · 前端（M0 视觉原型）

只有前端，数据全部是假数据（`src/data/fake.ts`）。M1 会在这个目录里加上 `src-tauri/`。

```bash
npm install
npm run dev          # http://localhost:5173
npm run check        # 类型检查
```

四个界面：`#/house` 座位 · `#/problems` 问题列表 · `#/problem/0001` 问题详情 · `#/proposal/p1` 提案审阅。

截图（1440×900 @2x）：先启动 dev，再运行 `npm run shoot`，图片输出到 `screenshots/`。
如果 Playwright 自带的浏览器版本对不上，可以用 `CHROME_PATH=/path/to/chrome` 指定。
