// 界面用的小工具：在 CLIENT-API 的数据上做的查找和措辞，不碰数据来源。
import type { Event, House, Member } from './types'
import { t } from '../i18n/index.svelte'

export const member = (h: House, id: string): Member =>
  h.members.find((m) => m.id === id) ?? { id, name: id, role: 'guest', joined: '' }
export const self = (h: House) => member(h, h.me)
export const host = (h: House) => h.members.find((m) => m.role === 'host')!
export const pad = (n: number) => String(n).padStart(2, '0')

/**
 * 你对茶馆里一个文件能做什么（RELAY.md 的法则，界面只用来标注，真正把关的是 Relay）：
 * write = 直接改；propose = 改动会变成提案；read = 只能看。
 * 留言写下不改，谁都是只读；.teahouse/ 和仓库根上的文件只有老板能改。
 */
export type Access = 'write' | 'propose' | 'read'
export function access(h: House, path: string): Access {
  const role = self(h).role
  if (path.startsWith('talk/')) return 'read'
  if (/^(problems|works|board)\//.test(path)) return role === 'guest' ? 'propose' : 'write'
  return role === 'host' ? 'write' : 'read'
}
/** 文件在茶馆里的真实名字。附件只有一份引用文件（CLIENT-API 1.6），显示附件本身的名字 */
export const fileName = (path: string) => (path.split('/').pop() ?? path).replace(/\.ref\.md$/, '')

/** 动作的说法。inProblem：在问题页的"经过"里（"提出了这个问题"），否则带上标题 */
// 提交说明的第一行就是界面上显示的动作描述（CLIENT-API 1.4），所以 title 有内容时原样显示；
// 没有时（比如有人直接用 git 提交）才按动作给一句通用的说法。
export function eventText(e: Event, inProblem = false): string {
  // 改状态、改问题的提交标题就是问题名，在问题页里写成做了什么
  if (e.title && !(inProblem && (e.act === 'set-status' || e.act === 'update-problem'))) return e.title
  const ref = ''
  switch (e.act) {
    case 'open-problem': return inProblem ? t('ev.open') : t('ev.openProblem', { ref })
    case 'set-status': return t(e.status === 'solved' ? 'ev.markSolved' : e.status === 'open' ? 'ev.markOpen' : 'ev.markActive')
    case 'update-problem': return t('ev.updateProblem')
    case 'add-work': return t('ev.addWork', { ref })
    case 'update-work':
    case 'update': return t('ev.update', { ref })
    case 'propose': return t('ev.propose', { ref })
    case 'adopt': return t('ev.merge', { ref })
    case 'decline': return t('ev.decline', { ref })
    case 'note': return t('ev.note')
    case 'remove-note': return t('ev.removeNote')
    case 'notice': return t('ev.notice', { ref })
    case 'seat': return t('ev.seat')
  }
}

/**
 * 茶馆名在首页是超大标题：长名字断成两行，断在词边界上，让两行尽量一样长。
 * 中文用 Intl.Segmenter 分词；西文只在空格和连字符后面断。
 */
export function nameLines(name: string, lang: string): string[] {
  const cjk = /[㐀-鿿]/.test(name)
  const len = [...name].length
  if (cjk ? len < 5 : len < 12) return [name]
  const breaks: number[] = []
  if (cjk) {
    let i = 0
    for (const s of new Intl.Segmenter(lang, { granularity: 'word' }).segment(name)) {
      i += s.segment.length
      if (i > 0 && i < name.length) breaks.push(i)
    }
  } else {
    for (let i = 1; i < name.length; i++) if (name[i - 1] === ' ' || name[i - 1] === '-') breaks.push(i)
  }
  if (!breaks.length) return [name]
  // 让较长的一行尽量短；一样长时把多出来的字放在第一行
  const best = breaks.reduce((a, b) => {
    const cost = (i: number) => Math.max(i, name.length - i)
    return cost(b) < cost(a) || (cost(b) === cost(a) && b > a) ? b : a
  })
  return [name.slice(0, best).trimEnd(), name.slice(best).trimStart()]
}

export type * from './types'
