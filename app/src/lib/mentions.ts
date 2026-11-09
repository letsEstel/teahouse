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

/** 一段文字切成：普通文字和 @点名 交替 */
export function splitMentions(text: string, tokens: MentionToken[]): { text: string; at?: MentionToken }[] {
  if (!tokens.length || !text.includes('@')) return [{ text }]
  const out: { text: string; at?: MentionToken }[] = []
  let rest = text
  while (rest) {
    const i = rest.indexOf('@')
    if (i < 0) { out.push({ text: rest }); break }
    const hit = tokens.find((t) => rest.startsWith(t.token, i))
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
