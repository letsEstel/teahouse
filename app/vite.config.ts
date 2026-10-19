import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { fileURLToPath } from 'node:url'
// @ts-ignore 纯 JS 的构建脚本
import { slimPreview } from './scripts/slim-preview.mjs'
// @ts-ignore 纯 JS 的构建脚本
import { engineBridge } from './scripts/engine-bridge.mjs'

export default defineConfig(({ mode }) => ({
  // 相对路径：打包结果放进任何目录（例如共享文件夹里的 预览/）都能打开；路由本来就是 hash 路由
  base: './',
  plugins: [svelte(), engineBridge(), ...(mode === 'preview' ? [slimPreview(fileURLToPath(new URL('./src', import.meta.url)))] : [])],
  // 预览打成单个普通脚本，直接双击 index.html 也能用（浏览器不允许从本地文件加载 module 脚本）
  build: mode === 'preview' ? { cssCodeSplit: false, rollupOptions: { output: { format: 'iife', inlineDynamicImports: true } } } : {},
  server: { port: 5173, strictPort: true },
}))
