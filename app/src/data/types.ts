import type { Text } from '../i18n/index.svelte'

export type Role = 'boss' | 'heir' | 'guest'
export type Status = 'open' | 'active' | 'solved'
export type WorkKind = 'doc' | 'bill' | 'plan' | 'image'
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
  nameLines: Record<keyof Text, string[]>
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
}
