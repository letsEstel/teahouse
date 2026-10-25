<script lang="ts">
  import { api, BackendError } from '../api/backend'
  import { report } from '../api/errors.svelte'
  import type { House } from '../api/types'
  import { t, when } from '../i18n/index.svelte'

  // 这间茶馆的两种状态，排在内容最上面、不盖住任何东西：
  //   已离座：本机钥匙不在座位表上（引擎给的 me 为空）→ 只读，副本保留
  //   未同步：有修改还没送到 Relay、Relay 有新内容、离线或同步出错
  //   已同步：一行小字"已同步 · 几分钟前"，旁边照样有"现在同步"——任何时候都能手动拉一次
  let { house, seated, refresh }: { house: House; seated: boolean; refresh: () => void } = $props()
  let s = $derived(house.sync)
  let busy = $state(false)

  async function sync() {
    busy = true
    // 同步不是"送出"，不走 write() 的送出锁；出错照样报告
    await api('sync', { house: house.id }).catch((e) => report(e instanceof BackendError ? e.error : { code: 'internal', message: String(e) }))
    busy = false
    refresh()
  }
</script>

{#if !seated}
  <div class="bar unseated" role="status">
    <i class="glyph"></i>
    <div class="text">
      <p class="what">{t('seat.left')}</p>
      <p class="more">{t('seat.leftMore')}</p>
    </div>
    <a class="act" href="#/{house.id}/archive">{t('seat.takeAway')}</a>
  </div>
{/if}

{#if s.state === 'synced'}
  <div class="quiet-sync" role="status">
    <span>{s.lastSync ? t('sync.syncedAt', { when: when(s.lastSync) }) : t('sync.synced')}</span>
    <button class="act" disabled={busy} onclick={sync}>{busy ? t('sync.busy') : t('sync.now')}</button>
  </div>
{:else}
  <div class="bar sync {s.state}" role="status">
    <i class="glyph"></i>
    <div class="text">
      <p class="what">
        {#if s.state === 'ahead'}{t('sync.ahead', { n: s.unpushed })}
        {:else if s.state === 'behind'}{t('sync.behind')}
        {:else if s.state === 'offline'}{t('sync.offline', { n: s.unpushed })}
        {:else}{t('sync.error')}{/if}
      </p>
      {#if s.state === 'error' && s.message}<p class="more">{s.message}</p>{/if}
      {#if s.lastSync}<p class="more">{t('sync.last', { when: when(s.lastSync) })}</p>{/if}
    </div>
    <button class="act" disabled={busy} onclick={sync}>{busy ? t('sync.busy') : s.state === 'offline' || s.state === 'error' ? t('err.retry') : t('sync.now')}</button>
  </div>
{/if}

<style>
  .quiet-sync { margin: 10px var(--pad-x) 0; display: flex; justify-content: flex-end; align-items: baseline; gap: 12px; font-size: 12px; color: var(--ink-3); }
  .bar { margin: 16px var(--pad-x) 0; display: grid; grid-template-columns: auto minmax(0, 1fr) auto; gap: 16px; align-items: start; border: 2px solid var(--ink); padding: 14px 18px; }
  .what { font-weight: 700; line-height: 1.5; }
  .more { margin-top: 2px; font-size: 13px; color: var(--ink-2); line-height: 1.5; overflow-wrap: anywhere; }
  .act { font-size: 13px; font-weight: 700; border-bottom: 1.5px solid var(--ink); padding: 2px 0; white-space: nowrap; }
  .act:disabled { color: var(--ink-3); border-color: var(--ink-3); }
  /* 几何记号，只用墨色：离座 = 空心方块里一道斜线；未送出 = 半满圆；有新内容 = 半满圆（另一半）；离线 = 空心圆；出错 = 实心方块 */
  .glyph { display: block; width: 16px; height: 16px; margin-top: 3px; border: 2px solid var(--ink); position: relative; overflow: hidden; }
  .unseated .glyph::after { content: ''; position: absolute; left: -4px; right: -4px; top: 50%; height: 2px; background: var(--ink); transform: rotate(-45deg); }
  .sync .glyph { border-radius: 50%; }
  .ahead .glyph::before, .behind .glyph::before { content: ''; position: absolute; top: 0; bottom: 0; width: 50%; background: var(--ink); }
  .ahead .glyph::before { left: 0; }
  .behind .glyph::before { right: 0; }
  .error .glyph { border-radius: 0; background: var(--ink); }
  .unseated { border-style: double; border-width: 4px; }
</style>
