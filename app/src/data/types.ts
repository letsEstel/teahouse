import type { Text } from '../i18n/index.svelte'

export type Role = 'boss' | 'heir' | 'guest'
export type Status = 'open' | 'active' | 'solved'
export type WorkKind = 'doc' | 'table' | 'plan' | 'image'  // table：预算、报价、数据表都用它
export type Act = 'addWork' | 'propose' | 'merge' | 'update' | 'markActive' | 'markSolved' | 'open' | 'openProblem'

export interface Member { id: string; name: Text; role: Role; joined: string }

export interface Work {
  id: string
  kind: WorkKind
  title: Text
  by: string
  when: string
  note?: Text
  /** 首张成果的预览：定制图（只有梧桐里的车位图）或一段摘录 */
  preview?: { chart: 'parking' } | { excerpt: Text }
  /** 文本成果的正文（逐行），发起提案时可以在此基础上修改 */
  lines?: Text[]
  /** 档案里"来历"的样例：这份成果每一次进入茶馆的改动（M1 起直接来自茶馆的历史） */
  history?: { by: string; when: string; act: 'add' | 'update' | 'merge'; diff: DiffLine[] }[]
}

/** 写下来的一段话：公告的回应、问题下的讨论。每条都是一次提交。 */
export interface Note { by: string; text: Text; when: string }

/** 公告板上的一条公告（board/ 目录下一个文件），只有老板和伙计能张贴 */
export interface Notice {
  id: string; by: string; title: Text; text: Text; when: string
  /** 回应，同样按进入茶馆的先后排列 */
  replies: Note[]
}

export interface Event { by: string; act: Act; ref?: Text; when: string; problem?: string }

export interface Problem {
  id: string
  title: Text
  summary: Text
  status: Status
  by: string
  opened: string
  updated: string
  body: { heading: Text; text: Text }[]
  works: Work[]
  /** 没有展开的问题只给成果数 */
  workCount?: number
  events: Event[]
  /** 问题下的讨论。数组顺序就是进入茶馆的先后（说明书第三节），界面照此显示，不按 when 排序 */
  notes?: Note[]
}

export interface DiffLine { kind: 'same' | 'add' | 'del'; text: Text }

export interface Proposal {
  id: string
  title: Text
  by: string
  problem: string
  when: string
  reason?: Text
  files: { title: Text; kind: WorkKind; lines: DiffLine[] }[]
}

export interface Decided { title: Text; by: string; result: 'merged' | 'rejected'; when: string }

export interface House {
  id: string
  name: Text
  /** 大标题的断行位置，按语言给出 */
  nameLines: { zh: string[]; en: string[]; de?: string[] }
  /** 招牌：老板写的一句话 */
  sign: Text
  founded: string
  capacity: number
  /** M0：当前用户在这间茶馆里是谁 */
  me: string
  members: Member[]
  problems: Problem[]
  recent: Event[]
  proposals: Proposal[]
  decided: Decided[]
  board: Notice[]
}
