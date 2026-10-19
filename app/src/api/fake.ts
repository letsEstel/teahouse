// FakeBackend：把 src/data/ 里的假数据（三种语言）包装成 CLIENT-API 的形状。
// 用户内容按当前界面语言给出（真引擎里内容只有一种语言，界面不翻译它）。
// 写入命令只改内存里的副本，刷新页面就没了；读取永远是"本地"，所以同步状态恒为 synced。
import type * as A from './types'
import { houses as seed, type House as FHouse, type Problem as FProblem, type Work as FWork, type Event as FEvent, type Note as FNote, type Notice as FNotice } from '../data'
import { archive as fakeArchive, type AFile } from '../data/archive'
import { L, t, NOW, i18n, type Text } from '../i18n/index.svelte'

const store: FHouse[] = structuredClone(seed)
const synced: A.SyncState = { state: 'synced', unpushed: 0, lastSync: NOW.toISOString() }
const iso = (s: string) => (s.length === 10 ? `${s}T09:00:00Z` : new Date(s).toISOString())
const text = (s: string): Text => ({ zh: s, en: s, de: s })
const ok = <T>(data: T): A.Result<T> => ({ ok: true, data })
const fail = (code: A.ApiError['code'], message: string): A.Result<never> => ({ ok: false, error: { code, message } })
const find = (id: string) => store.find((h) => h.id === id)

// ---- 文件路径（成果、提案改动里的 id 就是路径）----
const slug = (s: string) => s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'x'
const ext = (k: FWork['kind']) => (k === 'image' ? 'png.ref.md' : k === 'table' ? 'csv' : 'md')
export const workPath = (p: string, w: FWork) => `works/${p}/${w.id}-${slug(w.title.en)}.${ext(w.kind)}`

// 表格：假数据里用空格或全角空格对齐，转成 CSV
const cells = (row: string) => row.trim().split(/\s{2,}|　+/)
const csv = (rows: string[]) => rows.map((r) => cells(r).map((c) => (/[",]/.test(c) ? `"${c.replace(/"/g, '""')}"` : c)).join(',')).join('\n')

function workText(w: FWork): string | undefined {
  if (w.kind === 'image') return undefined
  if (w.lines) return w.kind === 'table' ? csv(w.lines.map(L)) : w.lines.map(L).join('\n')
  if (w.preview && 'excerpt' in w.preview) return L(w.preview.excerpt)
  return undefined
}

function toWork(p: FProblem, w: FWork, file?: AFile): A.Work {
  return {
    id: workPath(p.id, w),
    kind: w.kind,
    title: L(w.title),
    by: w.by,
    when: iso(w.when),
    size: file?.size ?? 0,
    text: workText(w),
    note: w.note ? L(w.note) : undefined,
    blob: w.kind === 'image' ? { hash: `sha256:fake-${w.id}`, type: 'image/png', cached: true } : undefined,
  }
}

// ---- 动作 ----
// 合同 1.4：title 是提交说明的第一行，也就是界面上直接显示的那句话。
// 假数据按当前界面语言生成这句话（真引擎里是作者写下或引擎生成的原话）。
function toEvent(e: FEvent, problem?: string): A.Event {
  const ref = e.ref ? L(e.ref) : ''
  const base = { by: e.by, when: iso(e.when), problem }
  switch (e.act) {
    case 'addWork': return { ...base, act: 'add-work', title: t('ev.addWork', { ref }) }
    case 'update': return { ...base, act: 'update-work', title: t('ev.update', { ref }) }
    case 'merge': return { ...base, act: 'adopt', title: t('ev.merge', { ref }) }
    case 'propose': return { ...base, act: 'propose', title: t('ev.propose', { ref }) }
    case 'open': return { ...base, act: 'open-problem', title: t('ev.open') }
    case 'openProblem': return { ...base, act: 'open-problem', title: t('ev.openProblem', { ref }) }
    case 'markActive': return { ...base, act: 'set-status', status: 'active', title: t('ev.markActive') }
    case 'markSolved': return { ...base, act: 'set-status', status: 'solved', title: t('ev.markSolved') }
  }
}

// 留言：数组顺序就是进入茶馆的先后。进入时间不早于前一条（离线写的那条晚到了）
function toNotes(ns: FNote[], dir = ''): A.Note[] {
  let last = ''
  return ns.map((n) => {
    const written = iso(n.when)
    const when = written < last ? new Date(new Date(last).getTime() + 60_000).toISOString() : written
    last = when
    return { by: n.by, text: L(n.text), when, writtenAt: written, id: dir && `${dir}/${written.replace(/[-:]/g, '').slice(0, 15)}Z-${n.by}.md` }
  })
}
const toNotice = (n: FNotice): A.Notice => ({ id: n.id, by: n.by, title: L(n.title), text: L(n.text), when: iso(n.when), replies: toNotes(n.replies, `talk/board/${n.id}`) })

function summary(h: FHouse, p: FProblem): A.ProblemSummary {
  return {
    id: p.id, title: L(p.title), summary: L(p.summary), status: p.status, by: p.by,
    opened: iso(p.opened), updated: iso(p.updated),
    workCount: p.works.length || (p.workCount ?? 0),
    pendingProposals: h.proposals.filter((x) => x.problem === p.id).length,
  }
}

function toHouse(h: FHouse): A.House {
  return {
    id: h.id, name: L(h.name), signboard: L(h.sign), founded: iso(h.founded), capacity: 15,
    me: h.me,
    members: h.members.map((m) => ({ id: m.id, name: L(m.name), role: m.role, joined: iso(m.joined) })),
    problems: h.problems.map((p) => summary(h, p)),
    recent: h.recent.map((e) => toEvent(e, e.problem)),
    proposals: h.proposals.map((x) => ({
      id: x.id, title: L(x.title), by: x.by, problem: x.problem, when: iso(x.when),
      reason: x.reason ? L(x.reason) : undefined,
      files: x.files.map((f) => ({
        path: `works/${x.problem}/${slug(f.title.en)}.${f.kind === 'table' ? 'csv' : 'md'}`,
        title: L(f.title), kind: f.kind, change: 'modify' as const,
        lines: f.lines.map((l) => ({ kind: l.kind, text: L(l.text) })),
      })),
    })),
    decided: h.decided.map((d, i) => ({ id: `d${i}`, title: L(d.title), by: d.by, result: d.result === 'merged' ? 'adopted' : 'declined', when: iso(d.when) })),
    board: h.board.map(toNotice),
    sync: synced,
  }
}

function toProblem(h: FHouse, p: FProblem): A.Problem {
  const files = fakeArchive(h)
  return {
    ...summary(h, p),
    body: p.body.map((b) => ({ heading: L(b.heading), text: L(b.text) })),
    works: p.works.map((w) => toWork(p, w, files.find((f) => f.work === w))),
    events: [...p.events].reverse().map((e) => toEvent(e, p.id)),
    notes: toNotes(p.notes ?? [], `talk/${p.id}`),
  }
}

// ---- 档案：节点、原始文件文本、来历 ----
const fileId = (h: FHouse, f: AFile) => (f.work ? workPath(f.problem, f.work) : f.path)

// label 只放用户写的内容（合同第 3 节）；"问题说明""成员表"这类固定用语由界面翻译
function nodeOf(h: FHouse, f: AFile): A.ArchiveNode {
  const name = (id: string) => L(h.members.find((m) => m.id === id)!.name)
  const stamp = (s: string) => new Date(s).toISOString().slice(0, 16).replace('T', ' ')
  const label =
    f.kind === 'problem' ? `${f.problem} · ${L(f.problemRef!.title)}`
    : f.kind === 'notice' ? L(f.notice!.title)
    : f.kind === 'note' ? `${name(f.note!.by)} · ${stamp(f.note!.when)}`
    : f.kind === 'members' ? undefined
    : L(f.work!.title)
  return { id: fileId(h, f), label, kind: f.kind, problem: f.problem, size: f.size }
}

function tree(h: FHouse, view: 'dirs' | 'problems'): A.ArchiveNode {
  const files = fakeArchive(h)
  const pl = (id: string) => `${id} · ${L(h.problems.find((p) => p.id === id)!.title)}`
  const leaves = (pred: (f: AFile) => boolean, _inProblem = false) => files.filter(pred).map((f) => nodeOf(h, f))
  const dir = (id: string, label: string, children: A.ArchiveNode[], code?: A.ArchiveNode['dir']): A.ArchiveNode => (code ? { id, dir: code, kind: 'dir', children } : { id, label, kind: 'dir', children })
  const meta = dir('meta', t('archive.meta'), leaves((f) => f.kind === 'members'), 'meta')
  if (view === 'dirs') {
    return dir('root', h.id, [
      dir('problems', t('archive.problems'), leaves((f) => f.kind === 'problem'), 'problems'),
      dir('works', t('archive.works'), h.problems.filter((p) => p.works.length).map((p) => dir(`works/${p.id}`, pl(p.id), leaves((f) => !!f.work && f.problem === p.id))), 'works'),
      dir('board', t('archive.board'), leaves((f) => f.kind === 'notice'), 'board'),
      dir('talk', t('archive.talk'), [
        ...h.problems.filter((p) => p.notes?.length).map((p) => dir(`talk/${p.id}`, pl(p.id), leaves((f) => f.kind === 'note' && f.problem === p.id))),
        ...h.board.filter((n) => n.replies.length).map((n) => dir(`talk/board/${n.id}`, t('archive.noticePrefix', { title: L(n.title) }), leaves((f) => f.kind === 'note' && f.notice?.id === n.id))),
      ], 'talk'),
      meta,
    ])
  }
  return dir('root', h.id, [
    ...h.problems.map((p) => dir(`p/${p.id}`, pl(p.id), leaves((f) => f.problem === p.id, true))),
    dir('pboard', t('archive.board'), h.board.flatMap((n) => [
      ...leaves((f) => f.kind === 'notice' && f.notice!.id === n.id),
      ...leaves((f) => f.kind === 'note' && f.notice?.id === n.id, true),
    ]), 'board'),
    meta,
  ])
}

/** 文件原文，格式照 CLIENT-API 第 1 节 */
function rawText(h: FHouse, f: AFile): string | undefined {
  if (f.kind === 'problem') {
    const p = f.problemRef!
    return [
      '---', `title: ${L(p.title)}`, `status: ${p.status}`, '---',
      L(p.summary),
      ...p.body.flatMap((b) => ['', `## ${L(b.heading)}`, L(b.text)]),
    ].join('\n')
  }
  if (f.kind === 'notice') return ['---', `title: ${L(f.notice!.title)}`, '---', L(f.notice!.text)].join('\n')
  if (f.kind === 'note') return L(f.note!.text)
  if (f.kind === 'members') {
    return h.members.map((m) => [`[[member]]`, `id = "${m.id}"`, `name = "${L(m.name)}"`, `role = "${m.role}"`, `joined = ${m.joined}`].join('\n')).join('\n\n')
  }
  const w = f.work!
  if (w.kind === 'table') return w.lines ? csv(w.lines.map(L)) : undefined
  // 图片不进 git，只有一份引用文件（CLIENT-API 1.6）
  if (w.kind === 'image') return ['---', `title: ${L(w.title)}`, `blob: sha256:fake-${w.id}`, `size: ${f.size}`, 'type: image/png', '---', ...(w.note ? [L(w.note)] : [])].join('\n')
  const body = workText(w)
  const fm = ['---', `title: ${L(w.title)}`, ...(w.kind === 'plan' ? ['kind: plan'] : []), ...(w.note ? [`note: ${L(w.note)}`] : []), '---']
  return [...fm, body ?? ''].join('\n')
}

function history(h: FHouse, f: AFile): A.HistoryEntry[] {
  const name = (id: string) => L(h.members.find((m) => m.id === id)!.name)
  // title 是提交说明的原话（合同 1.4），假数据按当前语言生成
  return f.history.map((e) => {
    const [act, title]: [A.Act, string] =
      e.act === 'add' ? [f.kind === 'note' ? 'note' : f.kind === 'notice' ? 'notice' : f.kind === 'problem' ? 'open-problem' : 'add-work', f.kind === 'problem' ? t('ev.open') : t('archive.act.add')]
      : e.act === 'update' ? ['update-work', t('archive.act.update')]
      : e.act === 'merge' ? ['adopt', t('archive.act.merge')]
      : e.act === 'open' ? ['open-problem', t('ev.open')]
      : e.act === 'seat' ? ['seat', t('archive.act.seat', { name: name(e.who!) })]
      : ['set-status', t(e.act === 'markSolved' ? 'ev.markSolved' : 'ev.markActive')]
    const status: A.Status | undefined = e.act === 'markSolved' ? 'solved' : e.act === 'markActive' ? 'active' : undefined
    return { by: e.by, when: iso(e.when), act, problem: e.problem, title, status, diff: e.diff?.map((d) => ({ kind: d.kind, text: L(d.text) })) }
  })
}

// ---- 命令 ----
type Handlers = { [C in A.Command]: (args: A.Args<C>) => A.Result<A.Returns<C>> }

const written = (becameProposal = false): A.Result<A.WriteResult> => ok({ commit: 'local', sync: synced, becameProposal })

const handlers: Handlers = {
  houses: () => ok(store.map((h) => ({
    id: h.id, name: L(h.name), me: h.me, myRole: h.members.find((m) => m.id === h.me)!.role,
    seats: h.members.length, pendingProposals: h.proposals.length, sync: synced,
  }))),
  house: ({ house }) => {
    const h = find(house)
    return h ? ok(toHouse(h)) : fail('not_found', `no house ${house}`)
  },
  problem: ({ house, problem }) => {
    const h = find(house)
    const p = h?.problems.find((x) => x.id === problem)
    return h && p ? ok(toProblem(h, p)) : fail('not_found', `no problem ${problem}`)
  },
  archive: ({ house, view }) => {
    const h = find(house)
    return h ? ok(tree(h, view)) : fail('not_found', `no house ${house}`)
  },
  file: ({ house, id }) => {
    const h = find(house)
    const f = h && fakeArchive(h).find((x) => fileId(h, x) === id)
    if (!h || !f) return fail('not_found', `no file ${id}`)
    return ok({ node: nodeOf(h, f), text: rawText(h, f), history: history(h, f) })
  },
  // 假数据里只有车位图是"图片"，界面认得这个特殊地址
  blob: ({ id }) => (id.includes('/w1-') ? ok('fake:parking') : fail('not_found', 'no blob in M0')),
  export: () => ok(null),
  sync: () => ok(synced),
  join: () => fail('offline', 'M0 has no relay'),

  removeNote: () => written(),
  note: ({ house, problem, notice, text: s }) => {
    const h = find(house)
    if (!h) return fail('not_found', house)
    const n: FNote = { by: h.me, when: NOW.toISOString(), text: text(s) }
    if (notice) h.board.find((x) => x.id === notice)?.replies.push(n)
    else {
      const p = h.problems.find((x) => x.id === problem)
      if (!p) return fail('not_found', `${problem}`)
      ;(p.notes ??= []).push(n)
    }
    return written()
  },
  postNotice: ({ house, title, text: s }) => {
    const h = find(house)
    if (!h) return fail('not_found', house)
    const me = h.members.find((m) => m.id === h.me)!
    if (me.role === 'guest') return written(true)
    h.board.unshift({ id: `b${h.board.length + 1}`, by: h.me, when: NOW.toISOString(), title: text(title), text: text(s), replies: [] })
    return written()
  },
  propose: ({ house }) => (find(house) ? ok({ commit: 'local', sync: synced, id: 'p-new' }) : fail('not_found', house)),
  // 采纳、不采纳、撤回：M0 只在界面上表示，不改假数据（演示时可以"撤回决定"）
  withdraw: () => written(),
  adopt: () => written(),
  decline: () => written(),
  openProblem: () => written(),
  setStatus: () => written(),
  addWork: () => written(),
  updateWork: () => written(),
  seat: () => written(),
  invite: () => ok('TEA-M0-DEMO'),
  redeem: () => fail('offline', 'M0 has no relay'),
  purgeBlob: () => written(),
}

export const fakeBackend = {
  kind: 'fake' as const,
  async call<C extends A.Command>(cmd: C, args: A.Args<C>): Promise<A.Result<A.Returns<C>>> {
    return (handlers[cmd] as (a: A.Args<C>) => A.Result<A.Returns<C>>)(args)
  },
  /** 只给截图和导出脚本用：直接拿到某间茶馆的 API 形状 */
  houseIds: () => store.map((h) => h.id),
}
