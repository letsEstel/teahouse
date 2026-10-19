// 界面用的小工具：在 CLIENT-API 的数据上做的查找和措辞，不碰数据来源。
import type { Event, House, Member } from './types'
import { t } from '../i18n/index.svelte'

export const member = (h: House, id: string): Member =>
  h.members.find((m) => m.id === id) ?? { id, name: id, role: 'guest', joined: '' }
export const self = (h: House) => member(h, h.me)
export const host = (h: House) => h.members.find((m) => m.role === 'host')!
export const pad = (n: number) => String(n).padStart(2, '0')

/** 动作的说法。inProblem：在问题页的"经过"里（"提出了这个问题"），否则带上标题 */
// 提交说明的第一行就是界面上显示的动作描述（CLIENT-API 1.4），所以 title 有内容时原样显示；
// 没有时（比如有人直接用 git 提交）才按动作给一句通用的说法。
export function eventText(e: Event, inProblem = false): string {
  if (e.title) return e.title
  const ref = ''
  switch (e.act) {
    case 'open-problem': return inProblem ? t('ev.open') : t('ev.openProblem', { ref })
    case 'set-status': return t(e.status === 'solved' ? 'ev.markSolved' : 'ev.markActive')
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
