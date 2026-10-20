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
