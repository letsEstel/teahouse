// M0 假数据：六间茶馆，用来检验同一套界面是否适合不同的小团体（社区、读书、家庭、创业、艺术、科研）。
import type { House, Member, Problem } from './types'
import { wutong } from './wutong'
import { kant } from './kant'
import { family } from './family'
import { startup } from './startup'
import { band } from './band'
import { lab } from './lab'

export * from './types'

export const houses: House[] = [wutong, kant, family, startup, band, lab]

export const findHouse = (id: string | undefined) => houses.find((h) => h.id === id) ?? houses[0]
export const member = (h: House, id: string): Member => h.members.find((m) => m.id === id)!
export const self = (h: House) => member(h, h.me)
export const boss = (h: House) => h.members.find((m) => m.role === 'host')!
export const workCount = (p: Problem) => p.works.length || (p.workCount ?? 0)
export const pad = (n: number) => String(n).padStart(2, '0')
