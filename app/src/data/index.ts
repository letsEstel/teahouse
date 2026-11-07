// 核桃园：示例数据（docs/NAMES.zh.md）。七个茶摊子，给还没拿到邀请的人看看茶馆长什么样，也用来检验
// 同一套界面是否适合不同的小团体（社区、读书、家庭、创业、艺术、科研、老同学）。茶摊子是假的，真的才叫茶馆。
import type { House, Member, Problem } from './types'
import { wutong } from './wutong'
import { kant } from './kant'
import { family } from './family'
import { startup } from './startup'
import { band } from './band'
import { lab } from './lab'
import { xiaobei } from './xiaobei'

export * from './types'

export const houses: House[] = [wutong, kant, family, startup, band, lab, xiaobei]

export const findHouse = (id: string | undefined) => houses.find((h) => h.id === id) ?? houses[0]
export const member = (h: House, id: string): Member => h.members.find((m) => m.id === id)!
export const self = (h: House) => member(h, h.me)
export const boss = (h: House) => h.members.find((m) => m.role === 'host')!
export const workCount = (p: Problem) => p.works.length || (p.workCount ?? 0)
export const pad = (n: number) => String(n).padStart(2, '0')
