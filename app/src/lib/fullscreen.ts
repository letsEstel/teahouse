// 电脑版（0029）：F11 进入或退出全屏，全屏时 Esc 也能退出，和浏览器的习惯一样。
// 应用里用 Tauri 的窗口全屏（权限 core:window:allow-set-fullscreen）；浏览器里用页面全屏。手机上不需要。
import { getCurrentWindow } from '@tauri-apps/api/window'
import { inApp } from '../api/backend'

const phone = /Android|iPhone|iPad/i.test(navigator.userAgent)

async function isFull(): Promise<boolean> {
  return inApp ? getCurrentWindow().isFullscreen() : !!document.fullscreenElement
}

async function setFull(on: boolean) {
  if (inApp) await getCurrentWindow().setFullscreen(on)
  else if (on) await document.documentElement.requestFullscreen()
  else await document.exitFullscreen()
}

if (!phone) {
  addEventListener('keydown', (e) => {
    if (e.key !== 'F11' && e.key !== 'Escape') return
    if (e.key === 'F11') e.preventDefault()
    // Esc 只在全屏时退出全屏，不拦着页面里别的 Esc（关抽屉、关对话框）
    isFull()
      .then((full) => {
        if (e.key === 'F11') return setFull(!full)
        if (full) return setFull(false)
      })
      .catch(() => {})
  })
}
