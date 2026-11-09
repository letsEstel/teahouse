<script lang="ts">
  // 大云晓钟：收件箱（docs/NAMES.zh.md）。
  import Shape from '../lib/Shape.svelte'
  import MachineMark from '../lib/MachineMark.svelte'
  import Loading from '../lib/Loading.svelte'
  import { member, type House } from '../api/view'
  import { api, write, BackendError } from '../api/backend'
  import { t, when } from '../i18n/index.svelte'
  import type { ApiError, Notification } from '../api/types'

  // 收件箱（大云晓钟，0026）：跟我有关的新动静。像晓钟，有事敲一下叫你回来，不一直响。
  // 只列还没看过的；点一条，标成看过，同时去那一页。每台设备各记各的。
  let { house, refresh }: { house: House; refresh: () => void } = $props()
  let items = $state<Notification[] | null>(null)
  let failed = $state<ApiError | null>(null)
  async function load() {
    try {
      items = await api('inbox', { house: house.id })
      failed = null
    } catch (e) {
      failed = e instanceof BackendError ? e.error : { code: 'internal', message: String(e) }
    }
  }
  $effect(() => { load() })

  /** 点开去哪一页 */
  function target(n: Notification): string {
    const h = `#/${house.id}`
    if (n.kind === 'history') return `${h}/house`
    if (n.proposal && (n.kind === 'proposal' || n.post)) return `${h}/proposal/${encodeURIComponent(n.proposal)}`
    if (n.notice) return `${h}/board`
    if (n.problem) return `${h}/problem/${n.problem}`
    return n.kind === 'decision' ? `${h}/proposals` : `${h}/house`
  }
  // 标成看过不用等：同步时引擎在忙也不耽误跳过去
  function open(n: Notification) {
    write('markSeen', { house: house.id, ids: [n.id] }).then(() => refresh())
  }
  let clearing = $state(false)
  async function clear() {
    clearing = true
    if (await write('markSeen', { house: house.id })) { items = []; refresh() }
    clearing = false
  }
  const say = (n: Notification) => t(`inbox.${n.kind}`, { name: member(house, n.by).name })
</script>

<div class="wrap">
  <header>
    <div class="label">{t('inbox.kicker')}</div>
    <h1>{t('inbox.title')}</h1>
    <p class="lede">{t('inbox.lede')}</p>
  </header>

  {#if failed}
    <Loading error={failed} retry={load} />
  {:else if !items}
    <Loading />
  {:else if !items.length}
    <p class="none">{t('inbox.none')}</p>
  {:else}
    <div class="bar">
      <span class="label">{t('inbox.count', { n: items.length })}</span>
      <button class="secondary small" disabled={clearing} aria-busy={clearing || undefined} onclick={clear}>{t('inbox.clear')}</button>
    </div>
    <ol class="list">
      {#each items as n (n.id)}
        {@const m = member(house, n.by)}
        <li>
          <a href={target(n)} onclick={() => open(n)}>
            <i class="dot"></i>
            <span class="who">
              {#if n.kind !== 'history'}<Shape role={m.role} size={10} />{/if}
              <b>{say(n)}</b>{#if n.kind !== 'history'}<MachineMark {m} />{/if}
            </span>
            {#if n.kind !== 'history' && n.title}<span class="what">{n.title}</span>{/if}
            <span class="where">{n.problem ? `${n.problem} · ` : ''}{when(n.when)}</span>
          </a>
        </li>
      {/each}
    </ol>
  {/if}
</div>

<style>
  .wrap { padding: var(--pad-top) var(--pad-x) 80px; max-width: 900px; }
  header { padding-bottom: 24px; border-bottom: 2px solid var(--ink); }
  h1 { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(40px, 5vw, 72px); line-height: 1; margin-top: 14px; letter-spacing: var(--track); }
  .lede { margin-top: 14px; color: var(--ink-2); max-width: 34em; line-height: 1.7; }
  .none { padding: 32px 0; color: var(--ink-3); }
  .bar { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 12px; padding: 16px 0; }
  .small { min-height: 40px; font-size: 13px; padding: 6px 16px; }
  .list { list-style: none; border-top: 1px solid var(--ink); }
  .list li { border-bottom: 1px solid var(--rule); }
  .list a { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; grid-template-areas: 'dot who where' '. what what'; gap: 4px 12px; padding: 14px 4px; color: inherit; text-decoration: none; }
  .list a:hover { background: var(--paper-deep); }
  .dot { grid-area: dot; width: 8px; height: 8px; margin-top: 7px; background: var(--ink); }
  .who { grid-area: who; display: flex; flex-wrap: wrap; align-items: center; gap: 4px 8px; font-size: 15px; }
  .who b { font-weight: 700; }
  .what { grid-area: what; font-size: 14px; color: var(--ink-2); overflow-wrap: anywhere; }
  .where { grid-area: where; font-family: var(--latin); font-size: 12px; color: var(--ink-3); white-space: nowrap; }
</style>
