// 线上预览：构建前端，把 dist/ 全部内容复制到共享文件夹的 预览/（评审把它发布成私密网页，供用户在手机上看）。
//
//   npm run preview:publish
//
// 每轮结束都运行一次。PREVIEW_DIR=/path 可以改目标目录。
import { build } from 'vite'
import { cpSync, existsSync, mkdirSync, readdirSync, rmSync, writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'

const target = process.env.PREVIEW_DIR ?? '/mnt/c/Users/andmi/Documents/Elysium/茶馆/预览'
const parent = join(target, '..')
if (!existsSync(parent)) {
  console.error(`找不到 ${parent}。共享文件夹没挂载？`)
  process.exit(1)
}
await build({ logLevel: 'warn', mode: 'preview', build: { outDir: 'dist-preview', emptyOutDir: true } })
const dist = fileURLToPath(new URL('../dist-preview/', import.meta.url))
mkdirSync(target, { recursive: true })
// 先清空旧的预览，避免留下上一轮的旧文件
for (const f of readdirSync(target)) rmSync(join(target, f), { recursive: true, force: true })
cpSync(dist, target, { recursive: true })
writeFileSync(
  join(target, '说明.txt'),
  `茶馆界面预览（M0，假数据）。用浏览器打开 index.html。\n生成于 ${new Date().toISOString()}。\n` +
    `入口：index.html#/wutong/house（老板视角）、#/kant/house（茶客视角）、#/family/house（伙计视角）。\n`,
)
console.log(`预览已更新：${target}（${readdirSync(target).length} 项）`)
