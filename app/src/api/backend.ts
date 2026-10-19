// 界面和引擎之间只有这一个口子：call(命令名, 参数)。
// 命令名、参数、返回值见 docs/CLIENT-API.md 第 4 节（类型在 ./types.ts）。
//   fake  —— 现在：假数据（./fake.ts），截图和预览都用它
//   http  —— 第 2 步：开发期 POST http://127.0.0.1:7377/v1/<命令名>
//   tauri —— 打包后 invoke('<命令名>', 参数)（./tauri.ts）
// 选择方式：环境变量 VITE_BACKEND（fake / local / tauri）；没设时，在 Tauri 应用里用 tauri，在浏览器里用 fake。
// 应用里还有一个"示例"开关（localStorage teahouse.demo）：打开时用假数据，方便没拿到邀请也能看看。
// 界面任何地方都不要绕过这里去读数据。
import type { ApiError, Args, Command, Result, Returns } from './types'
import { fakeBackend } from './fake'
import { httpBackend } from './http'
import { tauriBackend } from './tauri'
import { convertFileSrc } from '@tauri-apps/api/core'
import { report } from './errors.svelte'

export interface Backend {
  kind: 'fake' | 'http' | 'tauri'
  call<C extends Command>(cmd: C, args: Args<C>): Promise<Result<Returns<C>>>
}

export class BackendError extends Error {
  constructor(public error: ApiError) {
    super(error.message)
  }
}

// local / http：开发期接真引擎（dev-stack.sh 设 VITE_BACKEND=local）
/** 是不是在打包好的应用里（Tauri 会在页面上放这个对象） */
export const inApp = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
/** 应用里的"示例"开关打开着 */
export const demoMode = inApp && (() => {
  try { return localStorage.getItem('teahouse.demo') === '1' } catch { return false }
})()
export function setDemo(on: boolean) {
  try {
    if (on) localStorage.setItem('teahouse.demo', '1')
    else localStorage.removeItem('teahouse.demo')
  } catch {}
  location.hash = ''
  location.reload()
}

const which: string = import.meta.env.VITE_BACKEND ?? (inApp && !demoMode ? 'tauri' : 'fake')
const httpBase: string = import.meta.env.VITE_BACKEND_URL ?? 'http://127.0.0.1:7377'
export const backend: Backend =
  which === 'local' || which === 'http' ? httpBackend(httpBase)
  : which === 'tauri' ? tauriBackend
  : fakeBackend

/** 读取用：出错时抛出 BackendError，由调用的地方决定怎么显示 */
export async function api<C extends Command>(cmd: C, args: Args<C>): Promise<Returns<C>> {
  const r = await backend.call(cmd, args)
  if (!r.ok) throw new BackendError(r.error)
  return r.data
}

/** 写入用：出错时在页面顶部报告（本地化的说明），返回 null；成功时返回结果 */
export async function write<C extends Command>(cmd: C, args: Args<C>): Promise<Returns<C> | null> {
  const r = await backend.call(cmd, args)
  if (r.ok) return r.data
  report(r.error)
  return null
}

/**
 * 附件显示用的地址。blob 命令给的是本机路径，浏览器不能直接打开：
 * 开发期用 serve 的只读地址 GET /v1/blob?house=&id=；Tauri 里用 convertFileSrc（CLIENT-API 第 4 节）。
 */
export function blobUrl(house: string, id: string, path: string): string {
  if (/^(https?:|data:|blob:|fake:)/.test(path)) return path
  if (backend.kind === 'http') return `${httpBase}/v1/blob?house=${encodeURIComponent(house)}&id=${encodeURIComponent(id)}`
  if (backend.kind === 'tauri') return convertFileSrc(path)
  return path
}
