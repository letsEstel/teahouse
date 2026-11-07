// README 上的五张图（用户 2026-11-07 定：全用示例数据，三张电脑、两张手机，提案那张是夜间模式）。
// 三种语言各拍一套，每份 README 用自己语言的那一套：docs/screenshots/<zh|en|de>/。
//
//   node scripts/readme-shots.mjs [输出目录]     默认 ../docs/screenshots
//
// 界面改了以后跑一次就能全部重拍。电脑图 1366×768 只拍第一屏，手机图 390×844 也只拍第一屏
// （README 上看的是“打开时的样子”，不是整页）。存成 JPEG，仓库里不放大图。
import { build, preview } from 'vite'
import { chromium } from 'playwright'
import { existsSync, mkdirSync, readdirSync } from 'node:fs'
import { homedir } from 'node:os'
import { join, resolve } from 'node:path'

const out = resolve(process.argv[2] ?? '../docs/screenshots')
mkdirSync(out, { recursive: true })

const langs = ['zh', 'en', 'de']
const shots = [
  { file: '1-seats', hash: '#/wutong/house', w: 1366, h: 768 },
  { file: '2-notices', hash: '#/kant/board', w: 1366, h: 768 },
  { file: '3-proposal', hash: '#/wutong/proposal/p1', w: 1366, h: 768, theme: 'dark' },
  { file: '4-phone-band', hash: '#/band/house', w: 390, h: 844 },
  { file: '5-phone-lab', hash: '#/lab/problem/0001', w: 390, h: 844 },
]

await build({ logLevel: 'warn' })
const server = await preview({ preview: { port: 4321, strictPort: false }, logLevel: 'warn' })
const base = server.resolvedUrls.local[0]
const libs = join(homedir(), '.cache/teahouse/chrome-libs')
const env = existsSync(libs) ? { ...process.env, LD_LIBRARY_PATH: libs } : undefined
async function launch() {
  try { return await chromium.launch({ env }) } catch (e) {
    const cache = join(homedir(), '.cache/ms-playwright')
    const dir = existsSync(cache) && readdirSync(cache).filter((d) => d.startsWith('chromium_headless_shell-')).sort().pop()
    if (!dir) throw e
    return chromium.launch({ executablePath: join(cache, dir, 'chrome-headless-shell-linux64/chrome-headless-shell'), env })
  }
}
const browser = await launch()
for (const lang of langs) for (const s of shots) {
  mkdirSync(join(out, lang), { recursive: true })
  const page = await browser.newPage({ viewport: { width: s.w, height: s.h }, deviceScaleFactor: 2, reducedMotion: 'reduce' })
  await page.goto(`${base}?lang=${lang}&tour=off&theme=${s.theme ?? 'light'}${s.hash}`)
  await page.evaluate(() => document.fonts.ready)
  await page.waitForTimeout(500)
  await page.screenshot({ path: join(out, lang, `${s.file}.jpg`), type: 'jpeg', quality: 80 })
  console.log(`${lang}/${s.file}.jpg`)
  await page.close()
}
await browser.close()
server.httpServer.close()
