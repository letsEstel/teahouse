// 引擎返回的数据模型：docs/CLIENT-API.md 第 3 节，逐字照搬。
// 用户写的内容都是普通字符串（不分语言）；界面自身的文字走 i18n。
// 标了"提议"的字段是 1 号在 PR #1 里提出、尚未写进合同的，全部可选，缺了界面照样能用。

export type Role = 'host' | 'steward' | 'guest'
export type Status = 'open' | 'active' | 'solved'
export type WorkKind = 'doc' | 'plan' | 'table' | 'image' | 'file'
export type Act =
  | 'open-problem' | 'set-status' | 'add-work' | 'update-work' | 'note' | 'notice'
  | 'propose' | 'adopt' | 'decline' | 'seat' | 'update'

export interface SyncState {
  state: 'synced' | 'ahead' | 'behind' | 'offline' | 'error'
  unpushed: number
  lastSync?: string
  message?: string
}

export interface HouseSummary {
  id: string; name: string; me: string; myRole: Role
  seats: number
  pendingProposals: number
  sync: SyncState
}

export interface Member { id: string; name: string; role: Role; joined: string }
export interface Event { by: string; act: Act; title: string; when: string; problem?: string; status?: Status }
/** when = 进入茶馆的时间，writtenAt = 文件名里的时间戳 */
export interface Note { by: string; text: string; when: string; writtenAt: string }
export interface Notice { id: string; by: string; title: string; text: string; when: string; replies: Note[] }

export interface Work {
  id: string
  kind: WorkKind; title: string; by: string; when: string
  size: number
  text?: string
  blob?: { hash: string; type: string; cached: boolean; purged?: boolean }
  /** 提议：一句话说明，来自 frontmatter 的 note:，显示在成果卡片标题下面 */
  note?: string
}

export interface ProblemSummary {
  id: string; title: string; summary: string; status: Status
  by: string; owner?: string; opened: string; updated: string
  workCount: number; pendingProposals: number
}
export interface Problem extends ProblemSummary {
  body: { heading: string; text: string }[]
  works: Work[]
  events: Event[]
  notes: Note[]
}

export interface DiffLine { kind: 'same' | 'add' | 'del'; text: string }
export interface FileChange { path: string; title: string; kind: WorkKind; change: 'add' | 'modify' | 'delete'; lines?: DiffLine[] }
export interface Proposal { id: string; title: string; by: string; problem: string; when: string; reason?: string; files: FileChange[] }
export interface Decided { id: string; title: string; by: string; result: 'adopted' | 'declined'; when: string; reason?: string }

export interface House {
  id: string; name: string; signboard?: string; founded: string; capacity: 15
  me: string; members: Member[]
  problems: ProblemSummary[]; recent: Event[]
  proposals: Proposal[]; decided: Decided[]; board: Notice[]
  sync: SyncState
}

export interface ArchiveNode {
  id: string; label: string
  kind: 'dir' | 'problem' | WorkKind | 'notice' | 'note' | 'members'
  problem?: string; size?: number
  children?: ArchiveNode[]
  /** 提议：目录的代码，由界面翻译；有它时界面不用 label */
  dir?: 'problems' | 'works' | 'board' | 'talk' | 'meta'
}
export interface HistoryEntry { by: string; when: string; act: Act; problem: string; title: string; diff?: DiffLine[] }

export interface ApiError {
  code: 'not_found' | 'not_a_member' | 'forbidden' | 'invalid' | 'conflict'
      | 'offline' | 'relay_refused' | 'too_large' | 'internal'
  message: string
  reasons?: string[]
}

export interface WriteResult { commit: 'local' | 'pushed'; sync: SyncState; becameProposal?: boolean; id?: string }

type HouseArg = { house: string }
type Change = { id?: string; kind: WorkKind; title: string; text?: string; fromPath?: string; delete?: boolean }

/** 第 4 节的全部命令：名字 → [参数, 返回值] */
export interface Commands {
  houses: [Record<string, never>, HouseSummary[]]
  house: [HouseArg, House]
  problem: [HouseArg & { problem: string }, Problem]
  archive: [HouseArg & { view: 'dirs' | 'problems' }, ArchiveNode]
  file: [HouseArg & { id: string }, { node: ArchiveNode; text?: string; history: HistoryEntry[] }]
  blob: [HouseArg & { id: string }, string]
  export: [HouseArg & { to: string }, null]

  sync: [{ house?: string }, SyncState]
  join: [{ relay: string; house: string }, HouseSummary]

  openProblem: [HouseArg & { title: string; body: { heading: string; text: string }[] }, WriteResult]
  setStatus: [HouseArg & { problem: string; status: Status; owner?: string }, WriteResult]
  addWork: [HouseArg & { problem: string; kind: WorkKind; title: string; text?: string; fromPath?: string }, WriteResult]
  updateWork: [HouseArg & { id: string; text?: string; fromPath?: string }, WriteResult]
  note: [HouseArg & { problem?: string; notice?: string; text: string }, WriteResult]
  postNotice: [HouseArg & { title: string; text: string }, WriteResult]
  propose: [HouseArg & { problem: string; title: string; reason?: string; changes: Change[] }, WriteResult]
  withdraw: [HouseArg & { proposal: string }, WriteResult]
  adopt: [HouseArg & { proposal: string }, WriteResult]
  decline: [HouseArg & { proposal: string; reason?: string }, WriteResult]
  seat: [HouseArg & { remove?: string; role?: { id: string; role: Role } }, WriteResult]
  invite: [HouseArg & { role?: Role; name?: string; days?: number }, string]
  redeem: [{ relay: string; code: string; name: string }, HouseSummary]
  purgeBlob: [HouseArg & { id: string }, WriteResult]
}
export type Command = keyof Commands
export type Args<C extends Command> = Commands[C][0]
export type Returns<C extends Command> = Commands[C][1]
export type Result<T> = { ok: true; data: T } | { ok: false; error: ApiError }
