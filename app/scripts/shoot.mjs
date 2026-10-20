// 每轮评审用的截图脚本（说明书第十三节）。
//
//   npm run shoot -- <两位轮次>-<简述>         例如 npm run shoot -- 02-公告与提案
//
// 1. 构建前端并在本地起一个预览服务；
// 2. 默认只保存 1366×768 中文一组（含夜间模式），整轮不超过 20 张；
//    --full 才保存 3 种尺寸 × 全部语言的全套；
// 3. 不管存不存图，溢出检查都对全部组合运行（1920×1080、1366×768、900×1000 × zh / en / de），
//    结果写进 检查.md；
// 4. 截图存到 茶馆/截图/<轮次>/，文件名 <界面>_<宽度>_<语言>.png。
//
// 可选：--only=wutong-house,kant-problem 只跑部分界面；--dpr=2 高清；
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
const fullSet = args.includes('--full')

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
  [390, 844],   // 手机：只检查，不存图（用户会在手机上看预览）
  // 安卓 Pixel Fold：外壳把状态栏（32）和手势条（24）让出去之后，网页实际拿到的大小
  [411, 741],   // 折叠（外屏 411×797）
  [930, 719],   // 展开（内屏 930×775，< 1024，平板布局）
]
// 安卓外框截图：假状态栏 + 手势条 + 中间的应用，就是 MainActivity.kt 加内边距后的样子
const android = { bar: 32, nav: 24, devices: [['fold', 411, 797], ['unfold', 930, 775]] }
const androidShots = [
  { name: 'wutong-house', hash: '#/wutong/house' },
  { name: 'wutong-problem', hash: '#/wutong/problem/0001' },
  { name: 'wutong-problem-dark', hash: '#/wutong/problem/0001', theme: 'dark' },
]
function androidFrame(src, w, h, dark) {
  const paper = dark ? '#161514' : '#F1ECE2'
  const ink = dark ? '#F1ECE2' : '#141414'
  return `<!doctype html><meta charset=utf-8><style>
  html,body{margin:0;background:${paper};color:${ink};font:600 14px/1 Roboto,system-ui,sans-serif}
  .bar{height:${android.bar}px;display:flex;align-items:center;justify-content:space-between;padding:0 16px}
  .icons{display:flex;gap:6px;align-items:center}
  .sig{width:0;height:0;border-left:12px solid transparent;border-bottom:12px solid ${ink}}
  .bat{width:20px;height:10px;border:1.5px solid ${ink};box-sizing:border-box;padding:1px}
  .bat i{display:block;height:100%;width:70%;background:${ink}}
  iframe{display:block;border:0;width:${w}px;height:${h - android.bar - android.nav}px}
  .nav{height:${android.nav}px;display:flex;align-items:center;justify-content:center}
  .nav i{width:108px;height:4px;background:${ink};opacity:.6}
</style><div class=bar><span>9:41</span><span class=icons><span class=sig></span><span class=bat><i></i></span></span></div>
<iframe src="${src}"></iframe><div class=nav><i></i></div>`
}
const langs = ['zh', 'en', 'de']
// 默认保存的一组：标了 save 的界面，存 1366 中文；saveDe 另存 1366 德文；saveAt 指定别的宽度（中文）
const keep = (s, w, lang) =>
  fullSet || (s.save && ((w === (s.saveAt ?? 1366) && lang === 'zh') || (s.saveDe && w === 1366 && lang === 'de')))

// 康德读书会：茶客修改术语表，然后写理由
const reasons = {
  zh: '我们以李秋零译本为准，统一用“物自身”，“本体”留给 Noumenon。',
  en: 'We read Li Qiuling’s translation, so use 物自身 throughout and keep 本体 for Noumenon.',
  de: 'Wir lesen Li Qiuling, also durchgehend 物自身; 本体 bleibt für Noumenon.',
}
async function fillProposal(p, lang) {
  await p.click('[data-work*="/k2-"]')
  const body = p.locator('textarea[name=body]')
  const v = await body.inputValue()
  await body.fill(v.replace('Ding an sich — 物自体', 'Ding an sich — 物自身').replace(/(Noumenon — 本体)[、,] ?物自体/, '$1'))
  await p.fill('textarea[name=reason]', reasons[lang])
}

// 每个界面：名字、路由、可选操作 act(page, lang)、只在哪些宽度跑（widths）、theme、
// save（本轮要保存的图）。所有界面都参加溢出检查，只有 save 的才存图。
const houses = ['wutong', 'kant', 'family', 'startup', 'band', 'lab']
const screens = []
for (const h of houses) {
  screens.push(
    { name: `${h}-house`, hash: `#/${h}/house` },
    { name: `${h}-board`, hash: `#/${h}/board` },
    { name: `${h}-problems`, hash: `#/${h}/problems` },
    { name: `${h}-problem`, hash: `#/${h}/problem/0001` },
    { name: `${h}-proposal`, hash: `#/${h}/proposal/p1` },
    { name: `${h}-archive`, hash: `#/${h}/archive` },
  )
}
screens.push(
  { name: 'wutong-kanban', hash: '#/wutong/problems/kanban' },
  { name: 'wutong-proposal-merged', hash: '#/wutong/proposal/p1', act: (p) => p.click('button.adopt') },
  { name: 'wutong-proposals', hash: '#/wutong/proposals', widths: [900] },
  { name: 'kant-propose', hash: '#/kant/propose/0001', act: fillProposal },
  { name: 'kant-propose-new', hash: '#/kant/propose/new' },
  { name: 'kant-propose-board', hash: '#/kant/propose/board' },
  { name: 'wutong-tour-1', hash: '#/wutong/house', tour: 1, full: false },
  // 侧栏：茶馆多于 4 间时折叠；展开后可滚动；窄屏下拉
  { name: 'wutong-rail-open', hash: '#/wutong/house', act: (p) => p.click('button.more'), widths: [1920, 1366] },
  { name: 'wutong-switch', hash: '#/wutong/house', act: (p) => p.click('button.switch'), widths: [900], full: false },
)
// 档案（第 6 轮）
screens.push(
  { name: 'kant-archive-history', hash: '#/kant/archive/' + encodeURIComponent('works/0001/k2-glossary-of-terms.md'), act: (p) => p.click('.entry') },
  { name: 'wutong-archive-byproblem', hash: '#/wutong/archive/' + encodeURIComponent('works/0001/w1-parking-map-measured.png.ref.md'), act: (p) => p.click('.seg button:nth-child(2)'), widths: [1920, 1366] },
  { name: 'wutong-archive-list', hash: '#/wutong/archive', widths: [900, 390] },
)
// 入座和三种状态（CLIENT-API 第 7 节第 3 步）
const invite = 'TH-7KQ2-M9XF-3RDP-WC5L@relay.teahouse.example'
screens.push(
  { name: 'join', hash: '#/join' },
  { name: 'join-filled', hash: '#/join', act: async (p) => { await p.fill('textarea[name=invite]', `邀请你坐进茶馆「梧桐里三号院」。打开茶馆，选“加入一间茶馆”，把这段话整段贴进去：${invite}`); await p.fill('input[name=name]', '小周') } },
  { name: 'join-seated', hash: '#/join', act: async (p) => { await p.fill('textarea[name=invite]', invite); await p.fill('input[name=name]', '小周'); await p.click('button[type=submit]'); await p.waitForTimeout(400) } },
  { name: 'wutong-invite', hash: '#/wutong/house', act: async (p) => { await p.click('[data-tour=action] button.primary'); await p.click('.invite .primary') } },
  { name: 'wutong-unsynced', hash: '#/wutong/house', query: '&fake=unsynced' },
  { name: 'wutong-offline', hash: '#/wutong/problem/0001', query: '&fake=offline' },
  { name: 'kant-unseated', hash: '#/kant/house', query: '&fake=unseated' },
  { name: 'kant-unseated-problem', hash: '#/kant/problem/0001', query: '&fake=unseated' },
  { name: 'wutong-refused', hash: '#/wutong/problem/0001', query: '&fake=refuse', act: async (p) => { await p.fill('.talk-write textarea', '我同意，周六我也来帮忙喷编号。'); await p.click('.talk-write button.submit'); await p.waitForTimeout(300) } },
)
// AI 成员的标记（docs/MCP.md）：三人创业团队里的 Claude
const aiNote = (p) => p.click('a[data-file$="-claude.md"]')
screens.push(
  { name: 'startup-proposal-ai', hash: '#/startup/proposal/p2' },
  { name: 'startup-archive-ai', hash: '#/startup/archive', act: aiNote, widths: [1920, 1366] },
  { name: 'startup-house-dark', hash: '#/startup/house', theme: 'dark', only1366: true },
)
// 本轮保存的图（第 9 轮：AI 标记）
const save = {
  'startup-house': {}, 'startup-problem': {}, 'startup-proposal-ai': {}, 'startup-archive-ai': {}, 'startup-house-dark': {},
}
for (const sc of screens) if (save[sc.name]) Object.assign(sc, { save: true }, save[sc.name])
// 手机
screens.push(
  { name: 'join-phone', hash: '#/join', widths: [390], act: async (p) => { await p.fill('textarea[name=invite]', invite); await p.fill('input[name=name]', '小周') } },
  { name: 'kant-unseated-phone', hash: '#/kant/problem/0001', query: '&fake=unseated', widths: [390] },
)

// 夜间模式：布局不变，只拍默认组
screens.push(
  { name: 'wutong-refused-dark', hash: '#/wutong/problem/0001', query: '&fake=refuse', theme: 'dark', only1366: true, act: async (p) => { await p.fill('.talk-write textarea', '我同意，周六我也来帮忙喷编号。'); await p.click('.talk-write button.submit'); await p.waitForTimeout(300) } },
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
let checked = 0
try {
  for (const [w, h] of sizes) {
    const ctx = await browser.newContext({ viewport: { width: w, height: h }, deviceScaleFactor: dpr, reducedMotion: 'reduce' })
    const page = await ctx.newPage()
    page.on('pageerror', (e) => report.push({ file: `(${w})`, issues: [`页面报错：${e.message}`] }))
    for (const lang of langs) {
      for (const s of screens) {
        if (only && !only.includes(s.name)) continue
        if (s.widths && !s.widths.includes(w)) continue
        if (s.only1366 && !(w === 1366 && lang === 'zh')) continue
        const file = `${s.name}_${w}_${lang}.png`
        await page.goto('about:blank')
        await page.goto(`${base}?lang=${lang}&tour=${s.tour ?? 'off'}&theme=${s.theme ?? 'light'}${s.query ?? ''}${s.hash}`)
        await page.evaluate(() => document.fonts.ready)
        await page.waitForTimeout(250)
        if (s.act) { await s.act(page, lang); await page.waitForTimeout(250) }
        if (s.tour) await page.waitForSelector('[role=dialog]')
        await page.waitForTimeout(150)
        const issues = await page.evaluate(audit)
        if (issues.length) report.push({ file, issues })
        checked++
        if (!keep(s, w, lang)) continue
        // 整页截图：把视口拉到整页高度再拍，这样固定的侧栏也是整页高（Playwright 的 fullPage 不会拉伸它）
        if (s.full !== false) await page.evaluate(() => window.scrollTo(0, 0))
        const full = s.full !== false && (await page.evaluate(() => document.documentElement.scrollHeight))
        if (full && full > h) { await page.setViewportSize({ width: w, height: full }); await page.waitForTimeout(150) }
        await page.screenshot({ path: join(out, file) })
        if (full && full > h) await page.setViewportSize({ width: w, height: h })
        count++
      }
    }
    await ctx.close()
  }
  for (const [dev, w, h] of android.devices) {
    const ctx = await browser.newContext({ viewport: { width: w, height: h }, deviceScaleFactor: dpr, reducedMotion: 'reduce' })
    const page = await ctx.newPage()
    for (const s of androidShots) {
      if (only && !only.includes(s.name)) continue
      const dark = s.theme === 'dark'
      await page.setContent(androidFrame(`${base}?lang=zh&tour=off&theme=${s.theme ?? 'light'}${s.hash}`, w, h, dark))
      const frame = page.frames()[1]
      await frame.waitForLoadState('load')
      await frame.evaluate(() => document.fonts.ready)
      await page.waitForTimeout(400)
      await page.screenshot({ path: join(out, `android-${dev}-${s.name}_${w}_zh.png`) })
      count++
    }
    await ctx.close()
  }
} finally {
  await browser.close()
  await new Promise((r) => server.httpServer.close(r))
}

const lines = [`# 截图检查：${round}`, '', `检查了 ${checked} 个组合（尺寸 ${sizes.map((s) => s.join('×')).join('、')}；语言 ${langs.join(' / ')}），保存 ${count} 张截图。`, '']
if (!report.length) lines.push('没有发现横向滚动、超出视口或文字溢出。')
for (const r of report) lines.push(`## ${r.file}`, ...r.issues.map((i) => `- ${i}`), '')
writeFileSync(join(out, '检查.md'), lines.join('\n') + '\n')
console.log(`检查 ${checked} 个组合，保存 ${count} 张，截图在 ${out}`)
console.log(report.length ? `有 ${report.length} 张存在问题，见 检查.md` : '检查通过：无横向滚动、无文字溢出')
