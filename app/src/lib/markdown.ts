// Markdown → 一棵普通的数据。解析用 CodeMirror 编辑器已经带着的 @lezer/markdown，和编辑时看到的一致。
// 内容来自茶馆里的其他人，所以不生成 HTML 字符串：Markdown.svelte 逐个节点渲染，
// 别人写的 HTML 只会原样显示成文字；链接只放行 http 和 https。
import { parser, GFM } from '@lezer/markdown'
import type { SyntaxNode } from '@lezer/common'
import type { DiffLine } from '../api/types'

export type Inline =
  | { t: 'text'; text: string }
  | { t: 'code'; text: string }
  | { t: 'strong' | 'em' | 'del'; c: Inline[] }
  | { t: 'link'; url: string; c: Inline[] }
  | { t: 'br' }
  | { t: 'img'; src: string; alt: string }
export type Item = { task?: boolean; blocks: Block[] }
export type Block =
  | { t: 'p'; c: Inline[] }
  | { t: 'h'; level: number; c: Inline[] }
  | { t: 'img'; src: string; alt: string }
  | { t: 'ul'; items: Item[] }
  | { t: 'ol'; start: number; items: Item[] }
  | { t: 'quote'; blocks: Block[] }
  | { t: 'code'; text: string }
  | { t: 'hr' }
  | { t: 'table'; head: Inline[][]; rows: Inline[][][] }

const md = parser.configure(GFM)
/** 这些节点只是标记符号，不是内容 */
const MARKS = new Set(['EmphasisMark', 'CodeMark', 'CodeInfo', 'LinkMark', 'QuoteMark', 'ListMark', 'HeaderMark', 'StrikethroughMark', 'TableDelimiter', 'TaskMarker', 'URL', 'LinkTitle', 'LinkLabel'])
const ENTITIES: Record<string, string> = { '&amp;': '&', '&lt;': '<', '&gt;': '>', '&quot;': '"', '&nbsp;': ' ' }
const safe = (url: string) => /^https?:\/\//i.test(url)
const kids = (n: SyntaxNode) => { const out: SyntaxNode[] = []; for (let c = n.firstChild; c; c = c.nextSibling) out.push(c); return out }

function push(out: Inline[], text: string) {
  if (!text) return
  const last = out[out.length - 1]
  if (last?.t === 'text') last.text += text
  else out.push({ t: 'text', text })
}

/** 一段行内内容：节点之间的空隙就是普通文字 */
function inlines(src: string, n: SyntaxNode, from = n.from, to = n.to): Inline[] {
  const out: Inline[] = []
  let at = from
  for (const k of kids(n)) {
    if (k.to <= from || k.from >= to) continue
    push(out, src.slice(at, k.from))
    at = k.to
    const body = () => {
      const marks = kids(k).filter((c) => c.name.endsWith('Mark'))
      return inlines(src, k, marks[0]?.to ?? k.from, marks[marks.length - 1]?.from ?? k.to)
    }
    switch (k.name) {
      case 'StrongEmphasis': out.push({ t: 'strong', c: body() }); break
      case 'Emphasis': out.push({ t: 'em', c: body() }); break
      case 'Strikethrough': out.push({ t: 'del', c: body() }); break
      case 'InlineCode': {
        const marks = kids(k).filter((c) => c.name === 'CodeMark')
        out.push({ t: 'code', text: src.slice(marks[0]?.to ?? k.from, marks[marks.length - 1]?.from ?? k.to).trim() })
        break
      }
      case 'Link': case 'Image': case 'Autolink': {
        const url = kids(k).find((c) => c.name === 'URL')
        const marks = kids(k).filter((c) => c.name === 'LinkMark')
        const inner = k.name === 'Autolink' ? [] : inlines(src, k, marks[0]?.to, marks[1]?.from)
        const address = url ? src.slice(url.from, url.to) : ''
        if (k.name === 'Autolink') push(out, address)
        else if (k.name === 'Image' && url && !safe(address)) out.push({ t: 'img', src: address, alt: src.slice(marks[0]?.to, marks[1]?.from) })
        else if (url && safe(address)) out.push({ t: 'link', url: address, c: inner })
        else for (const x of inner) x.t === 'text' ? push(out, x.text) : out.push(x)
        break
      }
      case 'Escape': push(out, src.slice(k.from + 1, k.to)); break
      case 'Entity': push(out, ENTITIES[src.slice(k.from, k.to)] ?? src.slice(k.from, k.to)); break
      case 'HardBreak': out.push({ t: 'br' }); break
      default:
        if (MARKS.has(k.name) && k.name !== 'URL') break
        push(out, src.slice(k.from, k.to))
    }
  }
  push(out, src.slice(at, to))
  return out
}

/** 段落里的图片单独成块 */
function paragraph(c: Inline[]): Block[] {
  const out: Block[] = []
  let run: Inline[] = []
  const flush = () => {
    if (run.some((x) => x.t !== 'text' || x.text.trim())) out.push({ t: 'p', c: run })
    run = []
  }
  for (const x of c) {
    if (x.t === 'img') { flush(); out.push(x) } else run.push(x)
  }
  flush()
  return out
}

function trimmed(c: Inline[]): Inline[] {
  const first = c[0], last = c[c.length - 1]
  if (first?.t === 'text') first.text = first.text.trimStart()
  if (last?.t === 'text') last.text = last.text.trimEnd()
  return c
}

function code(src: string, n: SyntaxNode): string {
  const parts = kids(n).filter((c) => c.name === 'CodeText')
  return parts.map((c) => src.slice(c.from, c.to)).join('\n')
}

function blocks(src: string, n: SyntaxNode): Block[] {
  return kids(n).flatMap((k) => block(src, k))
}

function items(src: string, n: SyntaxNode): Item[] {
  return kids(n).filter((c) => c.name === 'ListItem').map((li) => {
    const task = kids(li).find((c) => c.name === 'Task')
    if (!task) return { blocks: blocks(src, li) }
    const mark = kids(task).find((c) => c.name === 'TaskMarker')
    const rest = kids(li).filter((c) => c !== task && c.name !== 'ListMark').flatMap((c) => block(src, c))
    return { task: /x/i.test(src.slice(mark?.from ?? 0, mark?.to ?? 0)), blocks: [...paragraph(trimmed(inlines(src, task, mark?.to))), ...rest] }
  })
}

function block(src: string, n: SyntaxNode): Block[] {
  const heading = /^(?:ATX|Setext)Heading(\d)$/.exec(n.name)
  if (heading) return [{ t: 'h', level: Number(heading[1]), c: trimmed(inlines(src, n)) }]
  switch (n.name) {
    case 'Paragraph': return paragraph(inlines(src, n))
    case 'BulletList': return [{ t: 'ul', items: items(src, n) }]
    case 'OrderedList': {
      const mark = n.firstChild?.firstChild
      return [{ t: 'ol', start: parseInt(mark ? src.slice(mark.from, mark.to) : '1') || 1, items: items(src, n) }]
    }
    case 'Blockquote': return [{ t: 'quote', blocks: blocks(src, n) }]
    case 'FencedCode': case 'CodeBlock': return [{ t: 'code', text: code(src, n) }]
    case 'HorizontalRule': return [{ t: 'hr' }]
    case 'Table': {
      const cells = (r: SyntaxNode) => kids(r).filter((c) => c.name === 'TableCell').map((c) => inlines(src, c))
      const rows = kids(n)
      return [{ t: 'table', head: rows.filter((r) => r.name === 'TableHeader').flatMap(cells.length ? (r) => [cells(r)] : () => [])[0] ?? [], rows: rows.filter((r) => r.name === 'TableRow').map(cells) }]
    }
    case 'ListMark': case 'QuoteMark': return []
    // HTML 和其余一切：原样当文字，不解释
    default: return [{ t: 'p', c: [{ t: 'text', text: src.slice(n.from, n.to) }] }]
  }
}

export function parse(src: string): Block[] {
  return blocks(src, md.parse(src).topNode)
}

/** 文件开头的 frontmatter（--- 之间的 title、note…）去掉，只留正文 */
export const stripFront = (s = '') => s.replace(/^---\n[\s\S]*?\n---\n?/, '').replace(/^\n+/, '')

/** 这个文件的正文是 Markdown 吗？附件引用（.ref.md）和链接（.link.md）只是一张卡片的数据 */
export const isMarkdown = (path: string) => /\.md$/i.test(path) && !/\.(ref|link)\.md$/i.test(path)

/**
 * 差异里的 frontmatter 只是文件的元数据，审阅时不用看：新文件和没动过的都去掉。
 * 这几行本身被改了就留着，老板要看得到改了什么。
 */
export function bodyLines(lines: DiffLine[]): DiffLine[] {
  const live = lines.map((l, i) => ({ l, i })).filter((x) => x.l.kind !== 'del')
  if (live[0]?.l.text !== '---') return lines
  const close = live.slice(1).find((x) => x.l.text === '---')
  if (!close) return lines
  const front = lines.slice(0, close.i + 1)
  if (front.some((l) => l.kind === 'del') || new Set(front.map((l) => l.kind)).size > 1) return lines
  let rest = lines.slice(close.i + 1)
  while (rest.length && !rest[0].text.trim() && rest[0].kind !== 'del') rest = rest.slice(1)
  return rest
}
