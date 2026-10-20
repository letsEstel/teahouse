// 端到端：界面接真引擎（scripts/dev-stack.sh 起的 teahouse-client serve），把每一个写入按钮点一遍，
// 然后直接问引擎，确认真的写进去了——不只看界面。
//
//   NO_UI=1 scripts/dev-stack.sh --fresh          # 另一个终端：引擎（陈立，梧桐里的老板）在 :7377
//   GUEST_PORT=7378 …                              # 可选：茶客（赵一鸣）的引擎，测提案和撤回
//   node scripts/e2e-real.mjs [截图目录]
//
// 每一步打印 ok / FAIL；有 FAIL 时退出码 1。
import { build, preview } from 'vite'
import { chromium } from 'playwright'
import { existsSync, readdirSync, mkdirSync } from 'node:fs'
import { homedir } from 'node:os'
import { join } from 'node:path'

const HOST = Number(process.env.HOST_PORT ?? 7377)
const GUEST = process.env.GUEST_PORT ? Number(process.env.GUEST_PORT) : null
const shots = process.argv[2]
if (shots) mkdirSync(shots, { recursive: true })
const house = 'wutong'

let failed = 0
const ok = (what) => console.log(`ok    ${what}`)
const fail = (what, why) => { failed++; console.log(`FAIL  ${what}: ${why}`) }
async function engine(port, cmd, args) {
  const r = await (await fetch(`http://127.0.0.1:${port}/v1/${cmd}`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(args) })).json()
  if (!r.ok) throw new Error(`${cmd}: ${r.error.code} ${r.error.message}`)
  return r.data
}
async function step(what, fn) {
  try { await fn() } catch (e) { fail(what, e.message.split('\n')[0]) }
}

// 每个引擎一份构建：界面直接连那个端口
async function serve(port, outDir, previewPort) {
  process.env.VITE_BACKEND = 'local'
  process.env.VITE_BACKEND_URL = `http://127.0.0.1:${port}`
  await build({ logLevel: 'error', build: { outDir } })
  const s = await preview({ build: { outDir }, preview: { port: previewPort, strictPort: true }, logLevel: 'silent' })
  return { url: s.resolvedUrls.local[0], close: () => new Promise((r) => s.httpServer.close(r)) }
}

const libs = join(homedir(), '.cache/teahouse/chrome-libs')
const env = existsSync(libs) ? { ...process.env, LD_LIBRARY_PATH: libs } : undefined
const cache = join(homedir(), '.cache/ms-playwright')
const dir = readdirSync(cache).filter((d) => d.startsWith('chromium_headless_shell-')).sort().pop()
const browser = await chromium.launch({ executablePath: join(cache, dir, 'chrome-headless-shell-linux64/chrome-headless-shell'), env })

async function open(base, hash) {
  const page = await browser.newPage({ viewport: { width: 1366, height: 900 }, reducedMotion: 'reduce' })
  page.on('pageerror', (e) => fail('page error', e.message))
  await page.goto(`${base}?lang=zh&tour=off#/${house}/${hash}`)
  await page.waitForLoadState('networkidle')
  return page
}
const shot = async (page, name) => { if (shots) await page.screenshot({ path: join(shots, `${name}.png`), fullPage: true }) }
const stamp = Date.now().toString(36)

// ---------------- 老板 ----------------
const host = await serve(HOST, 'dist-e2e-host', 4401)
let problemId = null

await step('提出问题（openProblem）→ 跳到新问题', async () => {
  const before = (await engine(HOST, 'house', { house })).problems.length
  const p = await open(host.url, 'problems')
  await p.click('.action button.primary')
  await p.fill('#problem-title', `楼道里的自行车 ${stamp}`)
  await p.click('.compose .cm-content')
  await p.keyboard.type('先写一句摘要：楼道里停满了自行车。\n## 现状\n一楼到三楼一共 6 辆。\n## 目标\n楼道能走人。')
  await p.click('.compose button[type=submit]')
  await p.waitForURL(/#\/wutong\/problem\/\d{4}$/, { timeout: 30_000 })
  problemId = p.url().split('/').pop()
  const got = await engine(HOST, 'problem', { house, problem: problemId })
  const after = (await engine(HOST, 'house', { house })).problems.length
  if (!got.title.endsWith(stamp)) throw new Error(`title is ${got.title}`)
  if (after !== before + 1) throw new Error(`problems ${before} → ${after}`)
  const heads = got.body.map((b) => b.heading)
  if (!got.summary?.includes('楼道里停满了自行车') && !got.body.some((b) => b.text.includes('楼道里停满了自行车'))) throw new Error('summary missing')
  if (!heads.includes('现状') || !heads.includes('目标')) throw new Error(`sections ${JSON.stringify(heads)}`)
  await p.waitForSelector('h1, .title')
  await shot(p, '1-new-problem')
  ok(`提出问题 → #/wutong/problem/${problemId}，摘要和两节都在`)
  await p.close()
})

await step('写一份文档（addWork）', async () => {
  const p = await open(host.url, `problem/${problemId}`)
  await p.click('.action button.primary')
  await p.fill('#new-title', `分工 ${stamp}`)
  await p.click('.compose .cm-content')
  await p.keyboard.type('# 分工\n周六上午挪车。')
  await p.click('.compose .save .primary')
  await p.waitForSelector('.saved')
  const got = await engine(HOST, 'problem', { house, problem: problemId })
  if (!got.works.some((w) => w.title === `分工 ${stamp}`)) throw new Error('work not in engine')
  ok('写一份文档：引擎里有这份成果')
  await p.close()
})

await step('编辑成果（updateWork）', async () => {
  const w = (await engine(HOST, 'problem', { house, problem: problemId })).works.find((x) => x.title === `分工 ${stamp}`)
  const p = await open(host.url, `archive/${encodeURIComponent(w.id)}`)
  await p.click('.tools button.primary')
  await p.click('.editing .cm-content')
  await p.keyboard.press('Control+End')
  await p.keyboard.type('\n下雨改到周日。')
  await p.click('.editing .save .primary')
  await p.waitForSelector('.saved')
  const f = await engine(HOST, 'file', { house, id: w.id })
  if (!f.text.includes('下雨改到周日')) throw new Error('text not saved')
  ok('编辑成果：文件里有新加的一行')
  await p.close()
})

await step('写留言、删留言（note / removeNote）', async () => {
  const p = await open(host.url, `problem/${problemId}`)
  await p.fill('.talk-write textarea', `留言 ${stamp}`)
  await p.click('.talk-write button.submit')
  await p.waitForSelector(`text=留言 ${stamp}`)
  let got = await engine(HOST, 'problem', { house, problem: problemId })
  if (!got.notes.some((n) => n.text.includes(stamp))) throw new Error('note not in engine')
  ok('写留言：引擎里有')
  await p.click('.notes li:last-child .del')
  await p.click('.notes li:last-child .del.sure')
  await p.waitForTimeout(1500)
  got = await engine(HOST, 'problem', { house, problem: problemId })
  if (got.notes.some((n) => n.text.includes(stamp))) throw new Error('note still there')
  ok('删留言：引擎里没有了')
  await p.close()
})

await step('张贴公告（postNotice）', async () => {
  const before = (await engine(HOST, 'house', { house })).board.length
  const p = await open(host.url, 'board')
  await p.click('button.primary:has-text("张贴")').catch(() => p.click('.board button.primary'))
  await p.fill('.write input', `公告 ${stamp}`)
  await p.fill('.write textarea', '周六上午九点挪车。')
  await p.click('.write button.submit')
  await p.waitForTimeout(1500)
  const after = (await engine(HOST, 'house', { house })).board
  if (after.length !== before + 1 || !after.some((n) => n.title.includes(stamp))) throw new Error(`board ${before} → ${after.length}`)
  ok('张贴公告：引擎里有')
  await p.close()
})

await step('邀请（invite）', async () => {
  const p = await open(host.url, 'house')
  await p.click('[data-tour=action] button.primary')
  await p.click('.invite .primary')
  const code = await p.waitForSelector('text=/TH(-[0-9A-Z]{4}){4}@/')
  if (!code) throw new Error('no code shown')
  ok(`邀请：界面显示了邀请码（${(await code.textContent()).trim().slice(0, 9)}…）`)
  await p.close()
})

await step('每个页面都能在真数据上打开', async () => {
  const pages = ['house', 'board', 'problems', 'problems/kanban', `problem/${problemId}`, 'problem/0001', 'proposals', 'archive', 'propose']
  for (const hash of pages) {
    const p = await open(host.url, hash)
    await p.waitForTimeout(400)
    const blank = await p.evaluate(() => !document.querySelector('main .page')?.textContent?.trim())
    if (blank) throw new Error(`#/${house}/${hash} is empty`)
    await shot(p, `page-${hash.replace(/\//g, '-')}`)
    await p.close()
  }
  ok(`${pages.length} 个页面都有内容，没有页面报错`)
})

if (GUEST) {
  // ---------------- 茶客：提案、撤回；老板：采纳 ----------------
  const guest = await serve(GUEST, 'dist-e2e-guest', 4402)
  const proposalOf = async (title) => (await engine(HOST, 'house', { house })).proposals.find((x) => x.title.includes(title))
  for (const [what, keep] of [['撤回（withdraw）', false], ['采纳（adopt）', true]]) {
    await step(`茶客发起提案 → ${what}`, async () => {
      await engine(GUEST, 'sync', { house })
      const p = await open(guest.url, 'propose')
      await p.click(`[data-problem="${problemId}"]`)
      await p.click('.seg button:nth-child(2)')                     // 新增一份成果
      await p.fill('.field input', `${keep ? '采纳' : '撤回'}用 ${stamp}`)
      await p.fill('textarea[name=body]', '茶客写的成果。')
      await p.fill('textarea[name=reason]', '测试')
      await p.click('button.send')
      await p.waitForTimeout(2500)
      await engine(GUEST, 'sync', { house })
      await engine(HOST, 'sync', { house })
      const prop = await proposalOf(`${keep ? '采纳' : '撤回'}用 ${stamp}`)
      if (!prop) throw new Error('proposal not on the relay')
      ok(`茶客发起提案：老板那边看到了（${prop.id}）`)
      if (!keep) {
        const g = await open(guest.url, `proposal/${encodeURIComponent(prop.id)}`)
        await g.click('.buttons button.secondary')
        await g.click('.buttons button.sure')
        await g.waitForURL(/#\/wutong\/proposals$/)
        await engine(HOST, 'sync', { house })
        if (await proposalOf(`撤回用 ${stamp}`)) throw new Error('still there after withdraw')
        ok('撤回：老板那边没有了')
        await g.close()
      } else {
        const h = await open(host.url, `proposal/${encodeURIComponent(prop.id)}`)
        await h.click('button.adopt')
        await h.waitForTimeout(2000)
        const works = (await engine(HOST, 'problem', { house, problem: problemId })).works
        if (!works.some((w) => w.title.includes(`采纳用 ${stamp}`))) throw new Error('adopted work not in main')
        ok('采纳：成果写进了茶馆')
        await h.close()
      }
      await p.close()
    })
  }
  await guest.close()
}

await host.close()
await browser.close()
console.log(failed ? `\n${failed} 处失败` : '\n全部通过')
process.exit(failed ? 1 : 0)
