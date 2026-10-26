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
import { existsSync, readdirSync, mkdirSync, mkdtempSync, writeFileSync } from 'node:fs'
import { homedir, tmpdir } from 'node:os'
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

// 选文件：页面在 http 模式下把文件交给 window.__teahouseStage，这里存进临时目录，路径交给引擎（backend.ts stageFile）
const stageDir = mkdtempSync(join(tmpdir(), 'teahouse-e2e-stage-'))
async function open(base, hash) {
  const page = await browser.newPage({ viewport: { width: 1366, height: 900 }, reducedMotion: 'reduce' })
  await page.exposeFunction('__teahouseStage', (name, b64) => {
    const path = join(stageDir, `${Date.now()}-${name.replace(/[\\/]/g, '_')}`)
    writeFileSync(path, Buffer.from(b64, 'base64'))
    return path
  })
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

await step('问题列表的日期（不再全是"刚刚"）', async () => {
  const h = await engine(HOST, 'house', { house })
  const old = h.problems.filter((x) => Date.now() - Date.parse(x.updated) > 86400_000)
  if (!old.length) throw new Error('no problem older than a day in the fixtures')
  const p = await open(host.url, 'problems')
  const shown = await p.$$eval('.when', (els) => els.map((e) => e.textContent.trim()))
  const just = shown.filter((x) => x === '刚刚').length
  if (just > h.problems.length - old.length) throw new Error(`${just} of ${shown.length} say 刚刚: ${shown.join(' / ')}`)
  ok(`问题列表的日期：${shown.slice(0, 4).join(' / ')} …（${old.length} 个超过一天的不显示"刚刚"）`)
  await p.close()
})

await step('改状态、指定负责人（setStatus）', async () => {
  const h = await engine(HOST, 'house', { house })
  const who = h.members.find((m) => m.id !== h.me)
  const p = await open(host.url, `problem/${problemId}`)
  await p.click('.manage .states button:nth-child(2)')
  await p.waitForSelector('.manage .states button.on:nth-child(2)')
  let got = await engine(HOST, 'problem', { house, problem: problemId })
  if (got.status !== 'active') throw new Error(`status ${got.status}`)
  await p.selectOption('#owner', who.id)
  await p.waitForSelector(`.meta :text("负责人：${who.name}")`)
  got = await engine(HOST, 'problem', { house, problem: problemId })
  if (got.owner !== who.id || got.status !== 'active') throw new Error(`owner ${got.owner}, status ${got.status}`)
  await shot(p, '2-status-owner')
  await p.click('.manage .states button:nth-child(3)')
  await p.waitForSelector('.manage .states button.on:nth-child(3)')
  got = await engine(HOST, 'problem', { house, problem: problemId })
  if (got.status !== 'solved') throw new Error(`status ${got.status}`)
  await p.waitForTimeout(3500)   // 等按钮上的结果消失
  await p.click('.manage .states button:nth-child(1)')
  await p.waitForSelector('.manage .states button.on:nth-child(1)')
  await p.selectOption('#owner', '')
  await p.waitForTimeout(1500)
  got = await engine(HOST, 'problem', { house, problem: problemId })
  if (got.status !== 'open' || got.owner) throw new Error(`status ${got.status}, owner ${got.owner}`)
  const list = (await engine(HOST, 'house', { house })).problems.find((x) => x.id === problemId)
  if (list.status !== 'open') throw new Error('house still shows the old status')
  ok(`改状态、负责人：进行中 + ${who.name} → 已办成 → 待认领 + 没有人，引擎里一致`)
  await p.close()
})

await step('修改问题（updateProblem）；单个回车显示成换行', async () => {
  const p = await open(host.url, `problem/${problemId}`)
  await p.click('header button.edit')
  await p.fill('#pedit-title', `楼道里的自行车和电动车 ${stamp}`)
  await p.click('.pedit .cm-content')
  await p.keyboard.press('Control+A')
  await p.keyboard.type('第一行。\n第二行，只隔一个回车。\n\n新的一段。\n## 现状\n一楼到三楼一共 8 辆。')
  await p.click('.pedit button[type=submit]')
  await p.waitForSelector('.wrap > .saved')
  const got = await engine(HOST, 'problem', { house, problem: problemId })
  if (got.title !== `楼道里的自行车和电动车 ${stamp}`) throw new Error(`title ${got.title}`)
  const intro = got.body.find((b) => !b.heading)?.text ?? ''
  if (intro !== '第一行。\n第二行，只隔一个回车。\n\n新的一段。') throw new Error(`intro ${JSON.stringify(intro)}`)
  if (!got.body.some((b) => b.heading === '现状' && b.text.includes('8 辆'))) throw new Error(`sections ${JSON.stringify(got.body)}`)
  if (got.status !== 'open') throw new Error(`status changed to ${got.status}`)
  // 显示：两行在屏幕上是上下两行（高度至少是一行的两倍多）
  await p.waitForSelector('.lead:has-text("第二行")')
  const lines = await p.$eval('.lead', (e) => Math.round(e.getBoundingClientRect().height / parseFloat(getComputedStyle(e).lineHeight)))
  if (lines < 4) throw new Error(`lead shows ${lines} lines`)
  await shot(p, '3-edited-problem')
  ok(`修改问题：标题、说明写进引擎，状态不变；摘要显示成 ${lines} 行（含空行）`)
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

const findNode = (n, f) => (f(n) ? n : (n.children ?? []).map((c) => findNode(c, f)).find(Boolean))

await step('首页：只有一台设备时提醒添加第二台（0007）', async () => {
  const devs = await engine(HOST, 'devices', { house })
  const p = await open(host.url, 'house')
  await p.waitForTimeout(800)
  const tip = await p.$('.keytip')
  if (devs.length === 1 && !tip) throw new Error('one device but no tip')
  if (devs.length > 1 && tip) throw new Error(`${devs.length} devices but the tip shows`)
  if (tip) {
    await shot(p, '4-key-tip')
    await p.click('.keytip .quietbtn')
    await p.waitForSelector('.keytip', { state: 'detached' })
    await p.reload(); await p.waitForTimeout(800)
    if (await p.$('.keytip')) throw new Error('tip came back after 知道了')
    await p.evaluate(() => localStorage.removeItem('teahouse.oneDeviceTip'))
  }
  ok(`首页提醒：${devs.length} 台设备，${tip ? '显示提醒；点"知道了"以后不再出现' : '不显示'}`)
  await p.close()
})

await step('档案：座位表只读并说明原因（0012）', async () => {
  const p = await open(host.url, `archive/${encodeURIComponent('.teahouse/members.toml')}`)
  await p.waitForSelector('.tools .readonly')
  if (await p.$('.tools button.primary')) throw new Error('members.toml has an edit button')
  const why = await p.textContent('.tools .consequence')
  if (!why.includes('座位表不能在这里改')) throw new Error(`reason: ${why}`)
  await shot(p, '5-members-readonly')
  ok('座位表：标着"只能看"，没有编辑按钮，写了原因')
  await p.close()
})

await step('档案：直接改问题说明（updateProblem）和公告（updateNotice）', async () => {
  const tree = await engine(HOST, 'archive', { house, view: 'dirs' })
  const pf = findNode(tree, (n) => n.kind === 'problem' && n.problem === problemId)
  const before = await engine(HOST, 'problem', { house, problem: problemId })
  let p = await open(host.url, `archive/${encodeURIComponent(pf.id)}`)
  await p.click('.tools button.primary')
  const draft = await p.textContent('.editing .cm-content')
  if (draft.includes('title:') || draft.includes('status:')) throw new Error('editor shows the frontmatter')
  await p.click('.editing .cm-content')
  await p.keyboard.press('Control+End')
  await p.keyboard.type('\n在档案里补的一行。')
  await p.click('.editing .save .primary')
  await p.waitForSelector('.saved')
  const after = await engine(HOST, 'problem', { house, problem: problemId })
  if (!after.body.some((b) => b.text.includes('在档案里补的一行'))) throw new Error('line not in the problem')
  if (after.title !== before.title || after.status !== before.status) throw new Error('title or status changed')
  ok('改问题说明：引擎里有新的一行，标题和状态没动')
  await p.close()
  const nf = findNode(tree, (n) => n.kind === 'notice')
  p = await open(host.url, `archive/${encodeURIComponent(nf.id)}`)
  await p.click('.tools button.primary')
  await p.click('.editing .cm-content')
  await p.keyboard.press('Control+End')
  await p.keyboard.type(`\n补充 ${stamp}`)
  await p.click('.editing .save .primary')
  await p.waitForSelector('.saved')
  const f = await engine(HOST, 'file', { house, id: nf.id })
  if (!f.text.includes(`补充 ${stamp}`)) throw new Error('notice not changed')
  if ((f.text.match(/^---$/gm) ?? []).length !== 2) throw new Error('frontmatter written twice')
  ok('改公告：文件里有补充，frontmatter 只有一份')
  await p.close()
})

await step('带走整间茶馆（export）', async () => {
  const p = await open(host.url, 'archive')
  await p.click('.export button.secondary')
  await p.waitForSelector('.exported', { timeout: 60_000 })
  const path = (await p.textContent('.exported code'))?.trim()
  if (!path) throw new Error(await p.textContent('.exported'))
  const { readdirSync: ls, existsSync: has } = await import('node:fs')
  const top = ls(path)
  if (!top.some((x) => x.endsWith('.bundle')) || !has(join(path, 'files/.teahouse/members.toml')) || !has(join(path, 'README.txt'))) throw new Error(`exported ${top.join(', ')}`)
  const problems = ls(join(path, 'files/problems')).length
  await shot(p, '6-exported')
  ok(`带走：${path}（${top.join('、')}；问题 ${problems} 个）`)
  await p.close()
})

// 一张 8×8 的墨色方块 PNG，给插图用
const dot = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAgAAAAICAIAAABLbSncAAAAD0lEQVR4nGMQwQEYhpYEAN0TDwGnz5/SAAAAAElFTkSuQmCC', 'base64')

await step('提出问题时插图（先提出、再挂附件、补进正文）', async () => {
  const p = await open(host.url, 'problems')
  await p.click('.action button.primary')
  await p.fill('#problem-title', `有图的问题 ${stamp}`)
  await p.click('.compose .cm-content')
  await p.keyboard.type('车棚现在的样子：')
  await p.setInputFiles('.compose input.picker', { name: 'shed.png', mimeType: 'image/png', buffer: dot })
  await p.waitForFunction(() => document.querySelector('.compose .cm-content')?.textContent?.includes('pending') || !!document.querySelector('.compose .cm-content img'))
  await p.keyboard.press('Control+End')
  await p.keyboard.type('一共 6 辆。')
  await p.click('.compose button[type=submit]')
  await p.waitForURL(/#\/wutong\/problem\/\d{4}$/, { timeout: 30_000 })
  const id = p.url().split('/').pop()
  const got = await engine(HOST, 'problem', { house, problem: id })
  const all = got.body.map((b) => b.text).join('\n')
  const m = /!\[shed\]\((works\/[^)]+\.ref\.md)\)/.exec(all)
  if (!m) throw new Error(`no image line in ${JSON.stringify(all)}`)
  if (all.includes('pending:')) throw new Error('placeholder left in the text')
  if (!got.works.some((w) => w.id === m[1] && w.kind === 'image')) throw new Error('image is not a work of the problem')
  await engine(HOST, 'blob', { house, id: m[1] })
  await p.waitForSelector('.brief figure img', { timeout: 15_000 })
  await shot(p, '7-problem-with-image')
  ok(`提出问题时插图：正文里是 ${m[1]}，是这个问题的图片成果，页面上显示出来了`)
  await p.close()
})

await step('修改问题时插图', async () => {
  const p = await open(host.url, `problem/${problemId}`)
  await p.click('header button.edit')
  await p.click('.pedit .cm-content')
  await p.keyboard.press('Control+End')
  await p.setInputFiles('.pedit input.picker', { name: 'bikes.png', mimeType: 'image/png', buffer: dot })
  await p.waitForFunction(() => /works\//.test(document.querySelector('.pedit .cm-content')?.textContent ?? '') || !!document.querySelector('.pedit .cm-content img'), null, { timeout: 30_000 })
  await p.click('.pedit button[type=submit]')
  await p.waitForSelector('.wrap > .saved')
  const got = await engine(HOST, 'problem', { house, problem: problemId })
  if (!got.body.some((b) => /!\[bikes\]\(works\/[^)]+\)/.test(b.text))) throw new Error(`no image line: ${JSON.stringify(got.body)}`)
  await p.waitForSelector('.brief figure img', { timeout: 15_000 })
  ok('修改问题时插图：正文里有图片，页面上显示出来了')
  await p.close()
})

await step('加入链接（addWork kind=link）：问题页和档案里显示成卡片', async () => {
  const p = await open(host.url, `problem/${problemId}`)
  await p.click('.works-head button:has-text("加入链接")')
  await p.fill('#link-url', 'https://docs.qq.com/sheet/DTest' + stamp)
  await p.fill('#link-note', '大家在这里一起填')
  await p.click('form.compose button[type=submit]')
  await p.waitForSelector('a.linkrow', { timeout: 30_000 })
  const got = await engine(HOST, 'problem', { house, problem: problemId })
  const w = got.works.find((x) => x.kind === 'link')
  if (!w || w.url !== 'https://docs.qq.com/sheet/DTest' + stamp) throw new Error(`no link work: ${JSON.stringify(got.works.map((x) => [x.kind, x.url]))}`)
  if (w.title !== '腾讯文档') throw new Error(`title ${w.title} (expected the platform name when left empty)`)
  if (w.note !== '大家在这里一起填') throw new Error(`note ${w.note}`)
  const row = await p.textContent('a.linkrow')
  if (!row.includes('腾讯文档') || !row.includes('打开')) throw new Error(`row: ${row}`)
  if ((await p.getAttribute('a.linkrow', 'href')) !== w.url) throw new Error('row does not point at the url')
  await shot(p, '8-link-work')
  const a = await open(host.url, `archive/${encodeURIComponent(w.id)}`)
  await a.waitForSelector('.linkcard .lc-url', { timeout: 15_000 })
  if ((await a.textContent('.linkcard .lc-url')) !== w.url) throw new Error('archive card url')
  if (await a.$('.tools button:has-text("换一个文件")')) throw new Error('a link offers to replace a file')
  ok(`加入链接：引擎里是 kind=link、url、note，标题默认是平台名；问题页、档案里都是卡片`)
  await a.close(); await p.close()
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
  await step('茶客：看不到改状态；修改问题变成提案（updateProblem）', async () => {
    await engine(GUEST, 'sync', { house })
    const p = await open(guest.url, `problem/${problemId}`)
    if (await p.$('.manage')) throw new Error('guest sees the status controls')
    await p.click('header button.edit:has-text("提议修改")')
    await p.fill('#pedit-title', `茶客改的标题 ${stamp}`)
    await p.click('.pedit button[type=submit]')
    await p.waitForSelector('.wrap > .saved:has-text("提案")')
    await engine(GUEST, 'sync', { house })
    await engine(HOST, 'sync', { house })
    const got = await engine(HOST, 'problem', { house, problem: problemId })
    if (got.title.startsWith('茶客改的标题')) throw new Error('written directly, not proposed')
    const prop = (await engine(HOST, 'house', { house })).proposals.find((x) => x.files.some((f) => f.path.startsWith('problems/')) && x.problem === problemId)
    if (!prop) throw new Error('no proposal on the host side')
    ok(`茶客修改问题：没有直接写进去，老板那边有提案（${prop.id}）`)
    await p.close()
  })
  await guest.close()
}

await host.close()
await browser.close()
console.log(failed ? `\n${failed} 处失败` : '\n全部通过')
process.exit(failed ? 1 : 0)
