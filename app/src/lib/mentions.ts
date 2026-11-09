// @点名（收件箱 0026）：引擎在留言里找 “@名字” 或 “@成员id”，被点到的人收件箱里多一条。
// 界面这边做两件事：留言里把 @名字 加亮（点到自己的更醒目）；输入框里打 @ 弹出成员名单。
import { getContext, setContext } from 'svelte'

export interface MentionToken { token: string; me: boolean }
const KEY = Symbol('mentions')

/** 留言列表告诉里面的文字：这间包厢有哪些名字可以被点到、哪个是我 */
export function provideMentions(get: () => MentionToken[]) { setContext(KEY, get) }
export function useMentions(): (() => MentionToken[]) | undefined { return getContext(KEY) }

export function tokensOf(members: { id: string; name: string }[], me: string): MentionToken[] {
  const out: MentionToken[] = []
  for (const m of members) for (const s of new Set([m.name, m.id])) if (s) out.push({ token: `@${s}`, me: m.id === me })
  // 长的先配：@陈立国 不会被当成 @陈立
  return out.sort((a, b) => b.token.length - a.token.length)
}

/** 名字后面能不能就此断开：英文名字后面不能紧跟字母数字或“.字母”（那是邮箱、域名）；中文名字后面接着写字照常算 */
function boundaryAfter(token: string, after: string): boolean {
  if (!/[A-Za-z0-9]$/.test(token)) return true
  return !/^([A-Za-z0-9_]|\.[A-Za-z0-9])/.test(after)
}

/** 一段文字切成：普通文字和 @点名 交替 */
export function splitMentions(text: string, tokens: MentionToken[]): { text: string; at?: MentionToken }[] {
  if (!tokens.length || !text.includes('@')) return [{ text }]
  const out: { text: string; at?: MentionToken }[] = []
  let rest = text
  while (rest) {
    const i = rest.indexOf('@')
    if (i < 0) { out.push({ text: rest }); break }
    // 邮箱、网址里的 @ 不算点名：@ 前面紧挨着字母数字（abc@li.com），或者英文名字后面还连着字母数字、.域名
    const before = i > 0 ? rest[i - 1] : out.length ? out[out.length - 1].text.slice(-1) : ''
    const hit = /[A-Za-z0-9._%+-]/.test(before) ? undefined : tokens.find((t) => rest.startsWith(t.token, i) && boundaryAfter(t.token, rest.slice(i + t.token.length)))
    if (!hit) { out.push({ text: rest.slice(0, i + 1) }); rest = rest.slice(i + 1); continue }
    if (i > 0) out.push({ text: rest.slice(0, i) })
    out.push({ text: hit.token, at: hit })
    rest = rest.slice(i + hit.token.length)
  }
  // 合并相邻的普通文字
  return out.reduce<{ text: string; at?: MentionToken }[]>((a, p) => {
    const last = a[a.length - 1]
    if (last && !last.at && !p.at) last.text += p.text
    else a.push({ ...p })
    return a
  }, [])
}
