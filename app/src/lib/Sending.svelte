<script lang="ts">
  // 写入的进行和结果，显示在页面底部（见 ../api/sending.svelte.ts）。不盖住内容，读屏软件会念出来。
  import { sending } from '../api/sending.svelte'
  import { t } from '../i18n/index.svelte'

  // 很快就回来的写入不必闪一下"正在送出"：超过 0.4 秒才显示
  let slow = $state(false)
  $effect(() => {
    if (sending.count === 0) { slow = false; return }
    const timer = setTimeout(() => (slow = true), 400)
    return () => clearTimeout(timer)
  })
  let text = $derived(
    sending.count > 0 ? (slow ? t('send.slow') : '')
    : sending.result === 'pushed' ? t('send.pushed')
    : sending.result === 'local' ? t('send.local')
    : sending.result === 'done' ? t('send.done')
    : ''
  )
</script>

<div class="sending" role="status" aria-live="polite">
  {#if text}<p class:busy={sending.count > 0}><i aria-hidden="true"></i>{text}</p>{/if}
</div>

<style>
  .sending { position: fixed; left: 50%; bottom: max(16px, env(safe-area-inset-bottom)); transform: translateX(-50%); z-index: 50; pointer-events: none; width: max-content; max-width: calc(100vw - 32px); }
  p { display: flex; align-items: center; gap: 10px; background: var(--ink); color: var(--paper); padding: 10px 16px; font-size: 14px; line-height: 1.4; }
  i { flex: none; width: 8px; height: 8px; background: var(--paper); }
  /* 正在送出：小方块一亮一暗；减少动效时不动 */
  .busy i { animation: blink 1s steps(2, start) infinite; }
  @keyframes blink { to { visibility: hidden; } }
  @media (prefers-reduced-motion: reduce) { .busy i { animation: none; } }
</style>
