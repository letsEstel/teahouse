// 自动同步（CLIENT-API 第 4 节：引擎每 60 秒后台同步，有变化通知界面）。
//   应用里（Tauri）：同步循环在外壳里（src-tauri/src/lib.rs）。这里只告诉它前台还是后台
//     （syncLoop { active }，来自 visibilitychange；变成前台时外壳会立刻同步一次，所以启动时也会同步），
//     外壳同步到新内容时发 teahouse://changed，这里收到就 refresh()。
//   开发期（serve 自己每 60 秒同步）：回到前台时同步一次当前茶馆，并且每 60 秒刷新一次界面。
//   假数据：什么也不做。
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { api, backend } from './backend'

export function startAutoSync(refresh: () => void, currentHouse: () => string | undefined): () => void {
  const visible = () => document.visibilityState === 'visible'
  if (backend.kind === 'tauri') {
    const report = () => invoke('syncLoop', { active: visible() }).catch(() => {})
    report()
    document.addEventListener('visibilitychange', report)
    const unlisten = listen('teahouse://changed', () => refresh())
    return () => {
      document.removeEventListener('visibilitychange', report)
      unlisten.then((f) => f())
    }
  }
  if (backend.kind !== 'http') return () => {}
  const syncNow = async () => {
    if (!visible()) return
    const h = currentHouse()
    if (h) await api('sync', { house: h }).catch(() => null)
    refresh()
  }
  document.addEventListener('visibilitychange', syncNow)
  const timer = setInterval(() => { if (visible()) refresh() }, 60_000)
  return () => {
    document.removeEventListener('visibilitychange', syncNow)
    clearInterval(timer)
  }
}
