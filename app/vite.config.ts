import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { fileURLToPath } from 'node:url'
import { readFileSync } from 'node:fs'
// @ts-ignore 纯 JS 的构建脚本
import { slimPreview } from './scripts/slim-preview.mjs'
// @ts-ignore 纯 JS 的构建脚本
import { engineBridge } from './scripts/engine-bridge.mjs'

// 版本号和代号（每个版本一种茶，拉丁字母，见 docs/DEVELOP.md）只写在 package.json 里
const pkg = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf8'))

export default defineConfig(({ mode }) => ({
  define: { __RELEASE__: JSON.stringify({ version: pkg.version, codename: pkg.codename }) },
  // 相对路径：打包结果放进任何目录（例如共享文件夹里的 预览/）都能打开；路由本来就是 hash 路由
  base: './',
  plugins: [svelte(), engineBridge(), ...(mode === 'preview' ? [slimPreview(fileURLToPath(new URL('./src', import.meta.url)))] : [])],
  // 预览打成单个普通脚本，直接双击 index.html 也能用（浏览器不允许从本地文件加载 module 脚本）
  build: mode === 'preview' ? { cssCodeSplit: false, rollupOptions: { output: { format: 'iife', inlineDynamicImports: true } } } : {},
  server: { port: 5173, strictPort: true },
}))
