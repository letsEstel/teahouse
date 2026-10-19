// 只用于预览打包（vite build --mode preview）的 Vite 插件：
// 1. 字体瘦身：fontsource 把思源黑体切成几百块（按 unicode-range），这里只保留界面和假数据里
//    真正出现过的字所在的块，并且只留 .woff2。预览从 1000 多个文件、30 MB 降到一百多个文件、约 4 MB，
//    方便同步到手机和发布成网页。正式应用不用这个插件，用户自己写的字照样有字体。
// 2. 去掉 type="module"：改成普通的 defer 脚本（打包格式为 iife），这样直接双击 index.html 也能打开。
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'

// 浏览器 Intl 生成的日期、相对时间里会出现、源码里未必有的字
const INTL = '年月日昨天前今明小时分钟秒刚周一二三四五六七八九十'

function usedChars(dir) {
  const walk = (d) => readdirSync(d).flatMap((f) => { const p = join(d, f); return statSync(p).isDirectory() ? walk(p) : [p] })
  const set = new Set([...INTL].map((c) => c.codePointAt(0)))
  for (const f of walk(dir)) for (const c of readFileSync(f, 'utf8')) set.add(c.codePointAt(0))
  return set
}

const ranges = (ur) =>
  ur.split(',').map((r) => {
    const [a, b] = r.trim().replace(/^U\+/i, '').split('-')
    return [parseInt(a, 16), parseInt(b ?? a, 16)]
  })

export function slimPreview(srcDir) {
  return {
    name: 'teahouse-slim-preview',
    apply: 'build',
    enforce: 'post',
    generateBundle(_, bundle) {
      const chars = usedChars(srcDir)
      const keep = new Set()
      for (const asset of Object.values(bundle)) {
        if (asset.type !== 'asset' || !asset.fileName.endsWith('.css')) continue
        let css = String(asset.source)
        css = css.replace(/@font-face\{[^}]*\}/g, (face) => {
          const ur = face.match(/unicode-range:([^;}]*)/)
          const hit = !ur || [...chars].some((c) => ranges(ur[1]).some(([a, b]) => c >= a && c <= b))
          if (!hit) return ''
          // 只留 woff2
          face = face.replace(/,\s*url\([^)]*\.woff\)\s*format\(["']woff["']\)/g, '')
          for (const m of face.matchAll(/url\(([^)]*\.woff2)\)/g)) keep.add(m[1].split('/').pop())
          return face
        })
        asset.source = css
      }
      for (const [key, asset] of Object.entries(bundle)) {
        if (asset.type === 'asset' && /\.woff2?$/.test(asset.fileName) && !keep.has(asset.fileName.split('/').pop())) delete bundle[key]
      }
    },
    transformIndexHtml: {
      order: 'post',
      handler: (html) => html.replace(/<script type="module" crossorigin/g, '<script defer').replace(/ crossorigin(?=[ >])/g, ''),
    },
  }
}
