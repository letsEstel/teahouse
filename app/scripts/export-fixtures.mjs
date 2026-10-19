// 导出种子数据：把 FakeBackend 里的六间示例茶馆按 CLIENT-API 第 3 节的形状写成
// ../fixtures/houses.zh.json（中文版），给 2 号的 dev-stack.sh 种成真的 git 仓库。
//
//   npm run export-fixtures
//
// 每间茶馆包含：house（House）、problems（每个问题的完整 Problem）、
// files（档案里每个文件的路径、类型、原文和来历，原文格式照第 1 节）。
import { createServer } from 'vite'
import { mkdirSync, writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

// i18n 模块在浏览器里读这些全局量；在 Node 里给一个中文环境
globalThis.location = { search: '?lang=zh' }
globalThis.document = { documentElement: { lang: '', dataset: {} }, title: '' }

const vite = await createServer({ server: { middlewareMode: true }, appType: 'custom', logLevel: 'warn' })
try {
  const { fakeBackend } = await vite.ssrLoadModule('/src/api/fake.ts')
  const call = async (cmd, args) => {
    const r = await fakeBackend.call(cmd, args)
    if (!r.ok) throw new Error(`${cmd}: ${r.error.message}`)
    return r.data
  }
  const leaves = (n) => (n.children ? n.children.flatMap(leaves) : [n])

  const houses = []
  for (const s of await call('houses', {})) {
    const house = await call('house', { house: s.id })
    const problems = []
    for (const p of house.problems) problems.push(await call('problem', { house: s.id, problem: p.id }))
    const files = []
    for (const node of leaves(await call('archive', { house: s.id, view: 'dirs' }))) {
      const f = await call('file', { house: s.id, id: node.id })
      files.push({ path: node.id, kind: node.kind, problem: node.problem, text: f.text ?? null, history: f.history })
    }
    houses.push({ house, problems, files })
  }

  const out = fileURLToPath(new URL('../../fixtures/', import.meta.url))
  mkdirSync(out, { recursive: true })
  const doc = {
    format: 'teahouse-fixtures/1',
    about: '六间示例茶馆（中文），形状照 docs/CLIENT-API.md 第 3 节；files[].text 是仓库里的文件原文（第 1 节的格式），图片没有原文。由 app/ 的 npm run export-fixtures 生成，不要手改。',
    houses,
  }
  writeFileSync(out + 'houses.zh.json', JSON.stringify(doc, null, 2) + '\n')
  console.log(`已导出 ${houses.length} 间茶馆 → fixtures/houses.zh.json`)
} finally {
  await vite.close()
}
