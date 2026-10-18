// 每轮评审用的截图脚本（说明书第十三节）。
//
//   npm run shoot -- <轮次编号>-<简述>        例如 npm run shoot -- 01-M0评审
//
// 1. 构建前端并在本地起一个预览服务；
// 2. 用 Playwright 打开每个界面，覆盖 1920×1080、1366×768、900×1000 三种尺寸和 zh / en / de 三种语言；
// 3. 截图存到 茶馆/截图/<轮次>/，文件名 <界面>_<宽度>_<语言>.png；
// 4. 顺带检查横向滚动和文字溢出，结果写进同目录的 检查.md。
//
// 可选：--only=wutong-house,kant-problem 只拍部分界面；--dpr=2 高清；
//      SHOTS_DIR=/path 改输出根目录；CHROME_PATH=/path/to/chrome 指定浏览器。
// 浏览器缺 libnss3 / libnspr4 时，先运行一次 scripts/setup-browser-libs.sh。
import { build, preview } from 'vite'
import { chromium } from 'playwright'
import { existsSync, mkdirSync, readdirSync, writeFileSync } from 'node:fs'
import { homedir } from 'node:os'
import { join } from 'node:path'

const args = process.argv.slice(2)
const round = args.find((a) => !a.startsWith('--'))
if (!round) {
  console.error('用法：npm run shoot -- <轮次编号>-<简述>')
  process.exit(1)
}
const opt = (k) => args.find((a) => a.startsWith(`--${k}=`))?.split('=')[1]
const only = opt('only')?.split(',')
const dpr = Number(opt('dpr') ?? 1)

const shared = '/mnt/c/Users/andmi/Documents/Elysium/茶馆/截图'
// 说明书第十三节：截图必须存到共享目录（绝对路径），不能存在 ~/teahouse 下。SHOTS_DIR 只用于临时试拍。
const root = process.env.SHOTS_DIR ?? shared
if (!existsSync(root)) {
  console.error(`找不到截图目录 ${root}。共享文件夹没挂载？`)
  process.exit(1)
}
const out = join(root, round)
mkdirSync(out, { recursive: true })

const sizes = [
  [1920, 1080],
  [1366, 768],
  [900, 1000],
]
const langs = ['zh', 'en', 'de']

// 每个界面：名字、路由、可选的额外操作、只在哪些宽度拍
const screens = []
for (const h of ['wutong', 'kant', 'startup']) {
  screens.push(
    { name: `${h}-house`, hash: `#/${h}/house` },
    { name: `${h}-problems`, hash: `#/${h}/problems` },
    { name: `${h}-problem`, hash: `#/${h}/problem/0001` },
    { name: `${h}-proposal`, hash: `#/${h}/proposal/p1` },
    // 宽屏上提案列表和提案详情是同一屏，只在窄窗口单独拍列表
    { name: `${h}-proposals`, hash: `#/${h}/proposals`, widths: [900] },
  )
}
screens.push(
  { name: 'wutong-proposal-merged', hash: '#/wutong/proposal/p1', act: (p) => p.click('button.adopt') },
  { name: 'wutong-proposal-rejected', hash: '#/wutong/proposal/p1', act: (p) => p.click('button.reject'), widths: [1366] },
  { name: 'wutong-tour-1', hash: '#/wutong/house', tour: 1, full: false },
  { name: 'wutong-tour-2', hash: '#/wutong/house', tour: 2, full: false },
  { name: 'wutong-tour-3', hash: '#/wutong/house', tour: 3, full: false },
  { name: 'kant-tour-3', hash: '#/kant/house', tour: 3, full: false, widths: [1366] },
)

// 在页面里运行：找横向滚动、超出视口的元素、溢出自身盒子的文字
function audit() {
  const vw = document.documentElement.clientWidth
  const issues = []
  const desc = (el) => {
    const c = typeof el.className === 'string' && el.className ? '.' + el.className.split(/\s+/).filter((x) => !x.startsWith('svelte-')).join('.') : ''
    return `${el.tagName.toLowerCase()}${c}「${(el.textContent ?? '').trim().slice(0, 24)}」`
  }
  if (document.documentElement.scrollWidth > vw) issues.push(`横向滚动：页面宽 ${document.documentElement.scrollWidth}px > 视口 ${vw}px`)
  for (const el of document.querySelectorAll('body *')) {
    if (el.closest('svg') && el.tagName.toLowerCase() !== 'svg') continue
    const cs = getComputedStyle(el)
    if (cs.display === 'none' || cs.visibility === 'hidden' || cs.position === 'fixed') continue
    const r = el.getBoundingClientRect()
    if (!r.width) continue
    if (r.right > vw + 1 && !el.closest('[style*="position: fixed"]')) issues.push(`超出视口 ${Math.round(r.right - vw)}px：${desc(el)}`)
    const ownText = [...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim())
    if (!ownText || cs.display.startsWith('inline') && cs.display !== 'inline-flex' && cs.display !== 'inline-block') continue
    if (el.scrollWidth > el.clientWidth + 1) {
      if (cs.textOverflow === 'ellipsis') issues.push(`被省略号截断：${desc(el)}`)
      else if (cs.overflowX === 'visible') issues.push(`文字溢出 ${el.scrollWidth - el.clientWidth}px：${desc(el)}`)
    }
  }
  // 文字超出所在的栏、卡片、按钮（例如大标题挤进隔壁一栏）
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT)
  const range = document.createRange()
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    if (!n.textContent.trim()) continue
    const box = n.parentElement.closest('section, aside, article, li, header, nav, figure, blockquote, button, a, [role=dialog]')
    if (!box || getComputedStyle(n.parentElement).position === 'fixed') continue
    range.selectNodeContents(n)
    const t = range.getBoundingClientRect()
    const b = box.getBoundingClientRect()
    if (t.width && (t.right > b.right + 1 || t.left < b.left - 1)) issues.push(`文字超出所在区域 ${Math.round(Math.max(t.right - b.right, b.left - t.left))}px：${desc(box)}「${n.textContent.trim().slice(0, 24)}」`)
  }
  return [...new Set(issues)]
}

console.log('构建前端…')
await build({ logLevel: 'warn' })
const server = await preview({ preview: { port: 4317, strictPort: false }, logLevel: 'warn' })
const base = server.resolvedUrls.local[0]

// Playwright 升级后自带浏览器版本可能对不上：先试默认，再找本机缓存里已有的任一版 headless shell
// 缺系统库时用 scripts/setup-browser-libs.sh 解压出来的库
const libs = join(homedir(), '.cache/teahouse/chrome-libs')
const env = existsSync(libs) ? { ...process.env, LD_LIBRARY_PATH: [libs, process.env.LD_LIBRARY_PATH].filter(Boolean).join(':') } : undefined
async function launch() {
  if (process.env.CHROME_PATH) return chromium.launch({ executablePath: process.env.CHROME_PATH, env })
  try {
    return await chromium.launch({ env })
  } catch (e) {
    const cache = join(homedir(), '.cache/ms-playwright')
    const dir = existsSync(cache) && readdirSync(cache).filter((d) => d.startsWith('chromium_headless_shell-')).sort().pop()
    const exe = dir && join(cache, dir, 'chrome-headless-shell-linux64/chrome-headless-shell')
    if (!exe || !existsSync(exe)) throw e
    console.log(`改用本机已有的浏览器：${dir}`)
    return chromium.launch({ executablePath: exe, env })
  }
}
const browser = await launch()
const report = []
let count = 0
try {
  for (const [w, h] of sizes) {
    const ctx = await browser.newContext({ viewport: { width: w, height: h }, deviceScaleFactor: dpr, reducedMotion: 'reduce' })
    const page = await ctx.newPage()
    page.on('pageerror', (e) => report.push({ file: `(${w})`, issues: [`页面报错：${e.message}`] }))
    for (const lang of langs) {
      for (const s of screens) {
        if (only && !only.includes(s.name)) continue
        if (s.widths && !s.widths.includes(w)) continue
        const file = `${s.name}_${w}_${lang}.png`
        await page.goto('about:blank')
        await page.goto(`${base}?lang=${lang}&tour=${s.tour ?? 'off'}${s.hash}`)
        await page.evaluate(() => document.fonts.ready)
        await page.waitForTimeout(250)
        if (s.act) { await s.act(page); await page.waitForTimeout(250) }
        if (s.tour) await page.waitForSelector('[role=dialog]')
        await page.waitForTimeout(150)
        const issues = await page.evaluate(audit)
        if (issues.length) report.push({ file, issues })
        // 整页截图：把视口拉到整页高度再拍，这样固定的侧栏也是整页高（Playwright 的 fullPage 不会拉伸它）
        const full = s.full !== false && (await page.evaluate(() => document.documentElement.scrollHeight))
        if (full && full > h) { await page.setViewportSize({ width: w, height: full }); await page.waitForTimeout(150) }
        await page.screenshot({ path: join(out, file) })
        if (full && full > h) await page.setViewportSize({ width: w, height: h })
        count++
        if (count % 20 === 0) console.log(`已拍 ${count} 张`)
      }
    }
    await ctx.close()
  }
} finally {
  await browser.close()
  await new Promise((r) => server.httpServer.close(r))
}

const lines = [`# 截图检查：${round}`, '', `共 ${count} 张。尺寸 ${sizes.map((s) => s.join('×')).join('、')}；语言 ${langs.join(' / ')}。`, '']
if (!report.length) lines.push('没有发现横向滚动、超出视口或文字溢出。')
for (const r of report) lines.push(`## ${r.file}`, ...r.issues.map((i) => `- ${i}`), '')
writeFileSync(join(out, '检查.md'), lines.join('\n') + '\n')
console.log(`\n截图在 ${out}`)
console.log(report.length ? `有 ${report.length} 张存在问题，见 检查.md` : '检查通过：无横向滚动、无文字溢出')
