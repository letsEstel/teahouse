<script lang="ts">
  import { t } from '../i18n/index.svelte'
  import { errorText } from '../api/errors.svelte'
  import type { ApiError } from '../api/types'

  // 还没读到数据时的样子（0024）：不留一片空白。
  // 300 毫秒内读到就什么也不显示（大多数时候是这样，不闪）；再久才出现三个形状和一句话。
  // 读不到时说清楚原因，给一个重试。
  let { what = 'page', error = null, retry }: { what?: 'house' | 'page'; error?: ApiError | null; retry?: () => void } = $props()
</script>

{#if error}
  <div class="loading failed" role="alert">
    <h1>{t(what === 'house' ? 'err.loadFailed' : 'load.failed')}</h1>
    <p>{errorText(error)}</p>
    <p class="detail">{error.message}</p>
    {#if retry}<button class="secondary" onclick={retry}>{t('err.retry')}</button>{/if}
  </div>
{:else}
  <div class="loading" role="status" aria-live="polite">
    <svg viewBox="0 0 68 20" aria-hidden="true">
      <rect x="1" y="1" width="18" height="18" />
      <polygon points="34,1.6 43,18.6 25,18.6" />
      <circle cx="58" cy="10" r="9" />
    </svg>
    <span>{t(what === 'house' ? 'load.house' : 'load.page')}</span>
  </div>
{/if}

<style>
  .loading { padding: var(--pad-top) var(--pad-x); display: flex; flex-direction: column; align-items: flex-start; gap: 16px; }
  .loading:not(.failed) { animation: appear 200ms var(--ease) 300ms both; }
  svg { width: 68px; height: 20px; }
  svg > * { fill: none; stroke: var(--ink); stroke-width: 1.5; animation: turn 1.5s steps(1) infinite; }
  svg > :nth-child(2) { animation-delay: 0.5s; }
  svg > :nth-child(3) { animation-delay: 1s; }
  span { font-size: 13px; color: var(--ink-2); letter-spacing: 0.1em; }
  .failed h1 { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(32px, 4vw, 56px); letter-spacing: var(--track); }
  .failed .detail { font-size: 13px; color: var(--ink-3); font-family: var(--latin); overflow-wrap: anywhere; }
  @keyframes appear { from { opacity: 0; } to { opacity: 1; } }
  /* 三个形状轮流涂黑，每次只有一个是实的 */
  @keyframes turn { 0% { fill: var(--ink); } 33.4% { fill: none; } }
</style>
