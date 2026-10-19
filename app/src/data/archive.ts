// 档案：把一间茶馆的数据还原成"文件"，像文件树一样浏览。
// M0 从假数据推出来；M1 起直接读本地仓库的文件和历史。
// path 是仓库里的真实路径，只给程序用，界面上一律不显示。
import type { DiffLine, House, Note, Notice, Problem, Work } from './types'

export type FileKind = 'problem' | 'doc' | 'table' | 'plan' | 'image' | 'notice' | 'note' | 'members'

/** "来历"里的一条：谁、什么时候、为了哪个问题、做了什么。problem 是问题编号、'board' 或 'meta' */
export interface HistEntry {
  by: string
  when: string
  problem: string
  act: 'add' | 'update' | 'merge' | 'open' | 'markActive' | 'markSolved' | 'seat'
  who?: string
  diff?: DiffLine[]
}

export interface AFile {
  id: string
  path: string
  kind: FileKind
  problem: string
  size: number
  history: HistEntry[]
  problemRef?: Problem
  work?: Work
  notice?: Notice
  note?: Note
}

const bytes = (...xs: (string | undefined)[]) => new TextEncoder().encode(xs.filter(Boolean).join('\n')).length
// 图片没有真实内容，按编号给一个稳定的假大小
const fakeSize = (id: string) => 140_000 + ([...id].reduce((n, c) => n * 31 + c.charCodeAt(0), 7) % 700_000)
const slug = (s: string) => s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'x'
const stamp = (iso: string) => iso.replace(/[-:]/g, '').slice(0, 13) + '00Z'
const ext = (k: Work['kind']) => (k === 'image' ? 'png' : k === 'table' ? 'csv' : 'md')

function workHistory(p: Problem, w: Work): HistEntry[] {
  if (w.history) return w.history.map((h) => ({ ...h, problem: p.id }))
  const first: HistEntry = {
    by: w.by, when: w.when, problem: p.id, act: 'add',
    diff: w.lines?.map((text) => ({ kind: 'add' as const, text })),
  }
  // 问题经过里提到这份成果的更新、采纳
  const later = p.events
    .filter((e) => (e.act === 'update' || e.act === 'merge') && e.ref?.zh === w.title.zh && e.when !== w.when)
    .map((e): HistEntry => ({ by: e.by, when: e.when, problem: p.id, act: e.act as 'update' | 'merge' }))
  return [first, ...later]
}

export function archive(h: House): AFile[] {
  const files: AFile[] = []
  const boss = h.members.find((m) => m.role === 'host')!
  for (const p of h.problems) {
    const status = p.events.filter((e) => e.act === 'open' || e.act === 'markActive' || e.act === 'markSolved')
    files.push({
      id: `p-${p.id}`,
      path: `problems/${p.id}-${slug(p.title.en)}.md`,
      kind: 'problem',
      problem: p.id,
      problemRef: p,
      size: bytes(p.title.zh, p.summary.zh, ...p.body.flatMap((b) => [b.heading.zh, b.text.zh])),
      history: status.length
        ? status.map((e) => ({ by: e.by, when: e.when, problem: p.id, act: e.act as HistEntry['act'] }))
        : [{ by: p.by, when: p.opened, problem: p.id, act: 'open' }],
    })
    for (const w of p.works) {
      files.push({
        id: `w-${w.id}`,
        path: `works/${p.id}/${w.id}.${ext(w.kind)}`,
        kind: w.kind,
        problem: p.id,
        work: w,
        size: w.kind === 'image' ? fakeSize(w.id) : bytes(w.title.zh, w.note?.zh, ...(w.lines?.map((l) => l.zh) ?? [])),
        history: workHistory(p, w),
      })
    }
    ;(p.notes ?? []).forEach((n, i) =>
      files.push({
        id: `t-${p.id}-${i}`,
        path: `talk/${p.id}/${stamp(n.when)}-${n.by}.md`,
        kind: 'note',
        problem: p.id,
        note: n,
        size: bytes(n.text.zh),
        history: [{ by: n.by, when: n.when, problem: p.id, act: 'add' }],
      }),
    )
  }
  for (const n of h.board) {
    files.push({
      id: `b-${n.id}`,
      path: `board/${n.id}.md`,
      kind: 'notice',
      problem: 'board',
      notice: n,
      size: bytes(n.title.zh, n.text.zh),
      history: [{ by: n.by, when: n.when, problem: 'board', act: 'add' }],
    })
    n.replies.forEach((r, i) =>
      files.push({
        id: `r-${n.id}-${i}`,
        path: `talk/board/${n.id}/${stamp(r.when)}-${r.by}.md`,
        kind: 'note',
        problem: 'board',
        notice: n,
        note: r,
        size: bytes(r.text.zh),
        history: [{ by: r.by, when: r.when, problem: 'board', act: 'add' }],
      }),
    )
  }
  files.push({
    id: 'members',
    path: '.teahouse/members.toml',
    kind: 'members',
    problem: 'meta',
    size: bytes(...h.members.map((m) => `${m.id} ${m.role} ${m.joined}`)) + 64 * h.members.length,
    history: [...h.members]
      .sort((a, b) => a.joined.localeCompare(b.joined))
      .map((m) => ({ by: boss.id, when: `${m.joined}T09:00`, problem: 'meta', act: 'seat', who: m.id })),
  })
  return files
}
