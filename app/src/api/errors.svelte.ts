// 引擎返回的错误：界面按 code（加上 params）显示本地化的句子；message 只是后备（CLIENT-API 第 5 节）。
import type { ApiError } from './types'
import { t } from '../i18n/index.svelte'

/** 当前要给人看的错误（显示在页面顶部的提示条里，不盖住内容） */
export const errors = $state<{ list: ApiError[] }>({ list: [] })

export function report(e: ApiError) {
  errors.list = [...errors.list, e]
}
export function dismiss(i: number) {
  errors.list = errors.list.filter((_, j) => j !== i)
}

const known = ['not_found', 'not_a_member', 'forbidden', 'invalid', 'conflict', 'offline', 'relay_refused', 'too_large', 'internal']

/** 一句话说明发生了什么 */
export function errorText(e: ApiError): string {
  if (e.params?.engine === 'unreachable') return t('err.engine')
  return known.includes(e.code) ? t(`err.${e.code}`, e.params ?? {}) : e.message
}
