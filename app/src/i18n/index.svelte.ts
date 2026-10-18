// 界面文字全部走翻译键：t('key', { 参数 })。
// 词条可以是字符串，也可以是按复数分类的对象 { one, other }（用参数 n 选择）。
import zh from './zh'
import en from './en'
import de from './de'

export type Lang = 'zh' | 'en' | 'de'
export const langs: Lang[] = ['zh', 'en', 'de']
export type Entry = string | { one?: string; other: string }
export type Dict = Record<string, Entry>

const dicts: Record<Lang, Dict> = { zh, en, de }
const locales: Record<Lang, string> = { zh: 'zh-CN', en: 'en-GB', de: 'de-DE' }

/** 成员写下的内容。M0 的假数据为了检验排版，每段都备了三种语言。 */
export type Text = Record<Lang, string>

function initial(): Lang {
  const q = new URLSearchParams(location.search).get('lang')
  if (q && q in dicts) return q as Lang
  try {
    const s = localStorage.getItem('teahouse.lang')
    if (s && s in dicts) return s as Lang
  } catch {}
  const nav = navigator.language.slice(0, 2)
  return nav in dicts ? (nav as Lang) : 'zh'
}

export const i18n = $state({ lang: initial() })
// 先同步设好 <html lang>：字体栈按它切换，组件挂载时的测量（例如 fit）要用到正确的字体
document.documentElement.lang = locales[i18n.lang]

export function setLang(l: Lang) {
  i18n.lang = l
  try { localStorage.setItem('teahouse.lang', l) } catch {}
}

$effect.root(() => {
  $effect(() => {
    document.documentElement.lang = locales[i18n.lang]
    document.title = t('app.name')
  })
})

export function t(key: string, params: Record<string, string | number> = {}): string {
  const e = dicts[i18n.lang][key] ?? dicts.zh[key]
  if (e === undefined) {
    if (import.meta.env.DEV) console.warn('missing i18n key', key)
    return key
  }
  let s: string
  if (typeof e === 'string') s = e
  else {
    const cat = new Intl.PluralRules(locales[i18n.lang]).select(Number(params.n ?? 0))
    s = (cat === 'one' && e.one) || e.other
  }
  return s.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? `{${k}}`))
}

/** 取一段内容在当前语言下的文字 */
export const L = (x: Text) => x[i18n.lang]

// ---- 日期与时间 ----

// M0 用固定的"现在"，保证截图稳定。M1 换成 new Date()。
export const NOW = new Date('2026-10-18T20:10:00')

const day = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime()

/** 相对时间："2 小时前" / "昨天 22:14" / "10月12日" */
export function when(iso: string): string {
  const d = new Date(iso)
  const loc = locales[i18n.lang]
  const rtf = new Intl.RelativeTimeFormat(loc, { numeric: 'auto' })
  const mins = Math.round((NOW.getTime() - d.getTime()) / 60000)
  const days = Math.round((day(NOW) - day(d)) / 86400000)
  if (mins < 1) return t('time.justNow')
  if (days === 0) return mins < 60 ? rtf.format(-mins, 'minute') : rtf.format(-Math.floor(mins / 60), 'hour')
  const time = new Intl.DateTimeFormat(loc, { hour: '2-digit', minute: '2-digit', hour12: false }).format(d)
  if (days === 1) return `${rtf.format(-1, 'day')} ${time}`
  return date(iso, d.getFullYear() === NOW.getFullYear() ? 'short' : 'long')
}

/** 日期：short = "10月12日" / "12 Oct" / "12. Okt."；long 带年份 */
export function date(iso: string, style: 'short' | 'long' = 'long'): string {
  const opts: Intl.DateTimeFormatOptions =
    style === 'short' ? { month: 'short', day: 'numeric' } : { year: 'numeric', month: 'long', day: 'numeric' }
  return new Intl.DateTimeFormat(locales[i18n.lang], opts).format(new Date(iso))
}
