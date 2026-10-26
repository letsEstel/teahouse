// 正文里的图片：图片本身照常作为附件进茶馆（addWork，CLIENT-API 1.6：内容进附件仓库，git 里只有引用），
// 正文里写一行 Markdown 指向这份附件：![标题](works/0001/w12-照片.png.ref.md)。
// 显示时用 blob 命令拿到本地缓存的地址。
import { api, write, stageFile, blobUrl } from '../api/backend'

/** 把选中的图片作为附件加进问题，返回要插进正文的那一行；变成提案（茶客）或失败时返回 null */
export async function attachImage(house: string, problem: string, file: File): Promise<string | null> {
  const title = file.name.replace(/\.[^.]+$/, '')
  const r = await write('addWork', { house, problem, kind: 'image', title, fromPath: await stageFile(file) })
  if (!r || r.becameProposal || !r.id) return null
  return `![${title}](${r.id})`
}

/** 正文里图片地址 → 能显示的地址（附件从本地缓存取，没有缓存时引擎先下载） */
export async function imageUrl(house: string, src: string): Promise<string | null> {
  if (/^(https?:|data:|blob:)/.test(src)) return src
  const path = await api('blob', { house, id: src }).catch(() => null)
  return path ? blobUrl(house, src, path) : null
}

/** 一段文字按行切开：单独一行的 ![标题](地址) 是图片，相邻的其余行合成一段文字（去掉首尾的空行） */
export function mdPieces(p: string): { text?: string; img?: { alt: string; src: string } }[] {
  const out: { text?: string; img?: { alt: string; src: string } }[] = []
  for (const line of p.split('\n')) {
    const m = /^\s*!\[([^\]]*)\]\(([^)\s]+)\)\s*$/.exec(line)
    if (m) out.push({ img: { alt: m[1], src: m[2] } })
    else if (out.length && out[out.length - 1].text !== undefined) out[out.length - 1].text += '\n' + line
    else out.push({ text: line })
  }
  return out
    .map((x) => (x.text === undefined ? x : { text: x.text.replace(/^\n+|\n+$/g, '') }))
    .filter((x) => x.img || x.text?.trim())
}

// 提出问题时问题还没有编号，图片挂不上去：先留在本机（pending:N），提出以后再作为附件加进去
const pending = new Map<string, File>()
let pendingCount = 0
/** 先记下图片，返回插进正文的一行（地址是 pending:N，预览时用 pendingUrl） */
export function holdImage(file: File): string {
  const key = `pending:${++pendingCount}`
  pending.set(key, file)
  return `![${file.name.replace(/\.[^.]+$/, '')}](${key})`
}
export function pendingUrl(src: string): string | null {
  const f = pending.get(src)
  return f ? URL.createObjectURL(f) : null
}
/** 问题提出以后：正文里每张 pending 图片作为附件加进问题，换成真正的地址；有一张失败就返回 null */
export async function attachPending(house: string, problem: string, text: string): Promise<string | null> {
  let out = text
  for (const [key, file] of pending) {
    if (!out.includes(`(${key})`)) continue
    const line = await attachImage(house, problem, file)
    if (!line) return null
    out = out.replace(new RegExp(`!\\[[^\\]]*\\]\\(${key}\\)`), line)
    pending.delete(key)
  }
  return out
}
/** 正文里还有没挂上的图片吗 */
export const hasPending = (text: string) => /\(pending:\d+\)/.test(text)
