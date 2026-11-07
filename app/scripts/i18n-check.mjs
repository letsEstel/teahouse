// 西夏碑（docs/NAMES.zh.md）：翻译键检查。
// 三种语言的翻译键要一样多（0038 时发现德文漏了 86 个，界面上直接冒出英文，没有任何报错）。
// npm run check 会跑它；缺了或多了都算失败。
import { readFileSync } from 'node:fs'

const langs = ['zh', 'en', 'de']
const keys = Object.fromEntries(langs.map((l) => {
  const src = readFileSync(new URL(`../src/i18n/${l}.ts`, import.meta.url), 'utf8')
  return [l, new Set([...src.matchAll(/^\s*'([\w.-]+)':/gm)].map((m) => m[1]))]
}))
const all = new Set(langs.flatMap((l) => [...keys[l]]))
let bad = 0
for (const l of langs) {
  const missing = [...all].filter((k) => !keys[l].has(k))
  if (missing.length) { bad++; console.error(`${l} 缺 ${missing.length} 个翻译键：${missing.join(', ')}`) }
}
if (bad) process.exit(1)
console.log(`翻译键齐全：${langs.join(' / ')} 各 ${all.size} 个`)
