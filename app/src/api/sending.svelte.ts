// 写入正在送出时的状态（用户反馈：服务器慢，点下去没有反应，就点了第二次，结果提交了两次）。
//
//   * 点下写入按钮，这个按钮立即变成"正在送出…"；送出期间所有写入类的按钮都点不动（鼠标和键盘都算）。
//   * 完成后在页面底部说一句结果：写进茶馆了，还是先存在这台设备上、联网后再送出。出错照旧走 ErrorBar。
//   * 另外 backend.ts 的 write() 把"同一个命令、同样的参数、上一次还没回来"的调用合并成一次，
//     就算哪个按钮漏了锁，也不会写两遍。
import { t } from '../i18n/index.svelte'
import type { WriteResult } from './types'

export const sending = $state<{ count: number; result: null | 'pushed' | 'local' | 'done' }>({ count: 0, result: null })

/** 写入类的按钮：主按钮、次按钮、提交按钮，以及 data-write 区域里的任何按钮 */
const WRITE_BUTTONS = 'button.primary, button.secondary, button.submit, button[type=submit], [data-write] button, button[data-write]'

let lastButton: HTMLButtonElement | null = null
let lastAt = 0

if (typeof document !== 'undefined') {
  // 捕获阶段：比组件自己的点击处理先到，送出期间直接拦下
  document.addEventListener('click', (e) => {
    const b = (e.target as Element | null)?.closest?.('button') as HTMLButtonElement | null
    if (!b) return
    if (sending.count > 0 && b.matches(WRITE_BUTTONS)) {
      e.preventDefault()
      e.stopImmediatePropagation()
      return
    }
    lastButton = b
    lastAt = performance.now()
  }, true)
  document.addEventListener('submit', (e) => {
    if (sending.count > 0) {
      e.preventDefault()
      e.stopImmediatePropagation()
      return
    }
    const b = (e as SubmitEvent).submitter
    if (b instanceof HTMLButtonElement) { lastButton = b; lastAt = performance.now() }
  }, true)
}

let resultTimer: ReturnType<typeof setTimeout> | undefined

/** 一次写入开始；返回结束时调用的函数 */
export function begin(): (outcome: unknown | null) => void {
  sending.count++
  sending.result = null
  document.body.classList.add('sending')
  // 刚点过的按钮（几秒之内，中间可能先暂存了附件）就是发起这次写入的按钮
  const button = lastButton && performance.now() - lastAt < 5000 && lastButton.isConnected ? lastButton : null
  if (button) {
    button.setAttribute('aria-busy', 'true')
    button.dataset.sending = t('send.sending')
  }
  return (outcome) => {
    sending.count = Math.max(0, sending.count - 1)
    if (sending.count === 0) document.body.classList.remove('sending')
    if (button) {
      button.removeAttribute('aria-busy')
      delete button.dataset.sending
    }
    if (outcome === null) return   // 出错：ErrorBar 会说
    const w = outcome as Partial<WriteResult>
    sending.result = w && typeof w === 'object' && 'commit' in w ? (w.commit === 'pushed' ? 'pushed' : 'local') : 'done'
    clearTimeout(resultTimer)
    resultTimer = setTimeout(() => (sending.result = null), 3500)
  }
}
