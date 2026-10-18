// 用无头浏览器给 M0 的四个界面截图：node scripts/shoot.mjs [baseURL]
import { chromium } from 'playwright'
import { mkdirSync } from 'node:fs'

const base = process.argv[2] ?? 'http://localhost:5173/'
const out = new URL('../screenshots/', import.meta.url).pathname
mkdirSync(out, { recursive: true })

const shots = [
  ['01-house', '#/house'],
  ['02-problems', '#/problems'],
  ['03-problem', '#/problem/0001'],
  ['04-proposal', '#/proposal/p1'],
]

const browser = await chromium.launch(
  process.env.CHROME_PATH ? { executablePath: process.env.CHROME_PATH } : {},
)
const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 })
for (const [name, hash] of shots) {
  await page.goto(base + hash)
  await page.evaluate(() => document.fonts.ready)
  await page.waitForTimeout(1200)
  await page.screenshot({ path: `${out}${name}.png`, fullPage: process.argv.includes('--full') })
  console.log('saved', name)
}
// 合并后的状态
await page.goto(base + '#/proposal/p1')
await page.waitForTimeout(800)
await page.click('button.merge')
await page.waitForTimeout(900)
await page.screenshot({ path: `${out}05-proposal-merged.png` })
console.log('saved 05-proposal-merged')
await browser.close()
