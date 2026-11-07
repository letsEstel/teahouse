// 一份提案是干什么的：从它改动的文件看出来（CLIENT-API 的 FileChange）。
// 界面上用它写清楚“提出新问题 / 加成果 / 改问题说明……”，也用它判断谁能决定。
import type { FileChange, Proposal } from '../api/types'

export type ProposalAct =
  | 'newProblem' | 'editProblem' | 'addWork' | 'editWork' | 'removeWork'
  | 'postNotice' | 'editNotice' | 'removeNotice' | 'note' | 'seats' | 'other'

export function fileAct(f: FileChange): ProposalAct {
  const { path, change } = f
  if (path.startsWith('.teahouse/')) return 'seats'
  if (path.startsWith('problems/')) return change === 'add' ? 'newProblem' : change === 'modify' ? 'editProblem' : 'other'
  if (path.startsWith('works/')) return change === 'add' ? 'addWork' : change === 'modify' ? 'editWork' : 'removeWork'
  if (path.startsWith('board/')) return change === 'add' ? 'postNotice' : change === 'modify' ? 'editNotice' : 'removeNotice'
  if (path.startsWith('talk/')) return 'note'
  return 'other'
}

/** 几处改动里最要紧的那一种，用来给整份提案起个名目；more = 其余几处 */
const ORDER: ProposalAct[] = ['seats', 'newProblem', 'editProblem', 'postNotice', 'editNotice', 'removeNotice', 'addWork', 'editWork', 'removeWork', 'note', 'other']
export function proposalAct(p: Proposal): { act: ProposalAct; more: number } {
  const acts = p.files.map(fileAct)
  const act = ORDER.find((a) => acts.includes(a)) ?? 'other'
  return { act, more: Math.max(0, p.files.length - 1) }
}

/** 茶客提出的新问题：编号是在茶客自己的副本里取的，采纳前不在正本上，采纳时可能改号 */
export const isNewProblem = (p: Proposal) => p.files.some((f) => f.change === 'add' && f.path.startsWith('problems/'))

/** 改座位表或茶馆设置（Problem: meta）的提案只有老板能决定（法则第 8、9 条） */
export const hostOnly = (p: Proposal) => p.problem === 'meta' || p.files.some((f) => f.path.startsWith('.teahouse/'))

/** 伙计暂时不能“不采纳”的：这条记录要写 Problem: meta（新问题、茶馆设置） */
export const stewardCannotDecline = (p: Proposal, problemExists: boolean) => hostOnly(p) || (p.problem !== 'board' && !problemExists)
