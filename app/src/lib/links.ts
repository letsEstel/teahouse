// 外部链接：认出常用的云文档平台；把文字里的网址切出来，显示成可以点的链接
import { t } from '../i18n/index.svelte'

const platforms: [RegExp, string][] = [
  [/(^|\.)docs\.qq\.com$/, 'link.tencent'],
  [/(^|\.)docs\.google\.com$/, 'link.google'],
  [/(^|\.)(feishu\.cn|larksuite\.com)$/, 'link.feishu'],
  [/(^|\.)shimo\.im$/, 'link.shimo'],
  [/(^|\.)notion\.(so|site)$/, 'link.notion'],
  [/(^|\.)yuque\.com$/, 'link.yuque'],
]

/** "腾讯文档" / "Google 文档" / 认不出时就是域名 */
export function platform(url: string): string {
  try {
    const host = new URL(url).hostname
    const hit = platforms.find(([rx]) => rx.test(host))
    return hit ? t(hit[1]) : host.replace(/^www\./, '')
  } catch {
    return url
  }
}

/** 文字切成段：普通文字和网址交替 */
export function linkify(text: string): { text: string; url?: string }[] {
  const out: { text: string; url?: string }[] = []
  const rx = /https?:\/\/[^\s<>"'，。；、）)]+/g
  let last = 0
  for (const m of text.matchAll(rx)) {
    if (m.index! > last) out.push({ text: text.slice(last, m.index) })
    out.push({ text: m[0], url: m[0] })
    last = m.index! + m[0].length
  }
  if (last < text.length) out.push({ text: text.slice(last) })
  return out
}
