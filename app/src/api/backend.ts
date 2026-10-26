// 界面和引擎之间只有这一个口子：call(命令名, 参数)。
// 命令名、参数、返回值见 docs/CLIENT-API.md 第 4 节（类型在 ./types.ts）。
//   fake  —— 现在：假数据（./fake.ts），截图和预览都用它
//   http  —— 第 2 步：开发期 POST http://127.0.0.1:7377/v1/<命令名>
//   tauri —— 打包后 invoke('<命令名>', 参数)（./tauri.ts）
// 选择方式：环境变量 VITE_BACKEND（fake / local / tauri）；没设时，在 Tauri 应用里用 tauri，在浏览器里用 fake。
// 应用里还有一个"示例"开关（localStorage teahouse.demo）：打开时用假数据，方便没拿到邀请也能看看。
// 界面任何地方都不要绕过这里去读数据。
import type { ApiError, Args, Command, Result, Returns } from './types'
import { fakeBackend, fakeStaged } from './fake'
import { httpBackend } from './http'
import { tauriBackend } from './tauri'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { report } from './errors.svelte'
import { begin } from './sending.svelte'
import { useFixedClock } from '../i18n/index.svelte'

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
useFixedClock(backend.kind === 'fake')

/** 带走整间茶馆：应用里交给外壳决定放哪（下载文件夹）；开发期放 VITE_EXPORT_DIR */
export const exportTo: string = backend.kind === 'http' ? (import.meta.env.VITE_EXPORT_DIR ?? '/tmp/teahouse-export') : ''

/** 读取用：出错时抛出 BackendError，由调用的地方决定怎么显示 */
export async function api<C extends Command>(cmd: C, args: Args<C>): Promise<Returns<C>> {
  const r = await backend.call(cmd, args)
  if (!r.ok) throw new BackendError(r.error)
  return r.data
}

/**
 * 写入用：出错时在页面顶部报告（本地化的说明），返回 null；成功时返回结果。
 * 送出期间按钮显示"正在送出…"、写入类按钮全部锁住（./sending.svelte.ts）；
 * 同一个命令、同样的参数还在送出时，再调用一次只会等同一个结果，不会写第二遍。
 */
const inflight = new Map<string, Promise<unknown>>()
export function write<C extends Command>(cmd: C, args: Args<C>): Promise<Returns<C> | null> {
  const key = `${cmd} ${JSON.stringify(args ?? {})}`
  const same = inflight.get(key)
  if (same) return same as Promise<Returns<C> | null>
  const finish = begin()
  const p = (async () => {
    try {
      const r = await backend.call(cmd, args)
      if (r.ok) { finish(r.data); return r.data }
      report(r.error)
      finish(null)
      return null
    } catch (e) {
      finish(null)
      throw e
    } finally {
      inflight.delete(key)
    }
  })()
  inflight.set(key, p)
  return p
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

/**
 * 附件从手机的文件选择器进来时只有内容，没有路径；引擎的 addWork / updateWork 要的是本机路径（fromPath）。
 * 应用里：把内容交给外壳的 stageFile（src-tauri），外壳存进应用缓存目录，返回路径。
 * 浏览器里（假数据、开发期）：没有本机路径可给，返回一个占位名，假数据不会去读它。
 */
export async function stageFile(file: File): Promise<string> {
  // 开发期接真引擎（http）：浏览器没法把文件放到引擎读得到的地方。测试脚本可以在页面上挂
  // window.__teahouseStage(文件名, base64) 代存，返回本机路径（scripts/e2e-real.mjs）
  const hook = (window as { __teahouseStage?: (name: string, b64: string) => Promise<string> }).__teahouseStage
  if (backend.kind === 'http' && hook) {
    const bytes = new Uint8Array(await file.arrayBuffer())
    let bin = ''
    for (const b of bytes) bin += String.fromCharCode(b)
    return hook(file.name, btoa(bin))
  }
  if (backend.kind !== 'tauri') {
    const key = `staged:${Date.now()}-${file.name}`
    if (backend.kind === 'fake') fakeStaged.set(key, URL.createObjectURL(file))
    return key
  }
  // 安卓的 IPC 只走 JSON（原始字节到了外壳会变成 JSON），所以安卓上用 base64；桌面直接传字节（src-tauri）
  const r = /Android/.test(navigator.userAgent)
    ? await invoke<Result<{ path: string }>>('stageFile', { name: file.name, data: await base64(file) })
    : await invoke<Result<{ path: string }>>('stageFile', new Uint8Array(await file.arrayBuffer()), {
        headers: { 'x-file-name': encodeURIComponent(file.name) },
      })
  if (!r.ok) throw new BackendError(r.error)
  return r.data.path
}

/** 文件内容的 base64（不带 data: 前缀） */
function base64(file: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const r = new FileReader()
    r.onload = () => resolve(String(r.result).replace(/^data:[^,]*,/, ''))
    r.onerror = () => reject(r.error)
    r.readAsDataURL(file)
  })
}
