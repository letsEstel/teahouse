<script lang="ts">
  import Rail from './lib/Rail.svelte'
  import Tour from './lib/Tour.svelte'
  import House from './views/House.svelte'
  import Problems from './views/Problems.svelte'
  import ProblemView from './views/Problem.svelte'
  import Proposal from './views/Proposal.svelte'
  import Board from './views/Board.svelte'
  import Propose from './views/Propose.svelte'
  import Archive from './views/Archive.svelte'
  import ErrorBar from './lib/ErrorBar.svelte'
  import { api, BackendError } from './api/backend'
  import { errorText } from './api/errors.svelte'
  import { t } from './i18n/index.svelte'
  import type { ApiError, House as HouseData, HouseSummary } from './api/types'
  import { i18n } from './i18n/index.svelte'

  // 路由：#/<茶馆>/<界面>/<参数>，例如 #/wutong/problem/0001、#/kant/propose/0001、#/wutong/problems/kanban
  let hash = $state(location.hash)
  $effect(() => {
    const on = () => { hash = location.hash; window.scrollTo(0, 0) }
    addEventListener('hashchange', on)
    return () => removeEventListener('hashchange', on)
  })

  let route = $derived.by(() => {
    const [, h, page, arg] = hash.split('/')
    return { houseId: h, page: page || 'house', arg: arg ? decodeURIComponent(arg) : arg }
  })

  // 数据只经过 api()（src/api/backend.ts）。写入之后调用 refresh() 重新读取。
  let houses = $state<HouseSummary[]>([])
  let house = $state<HouseData | null>(null)
  let version = $state(0)
  let failed = $state<ApiError | null>(null)
  const refresh = () => version++
  $effect(() => {
    const id = route.houseId
    void i18n.lang  // 假数据的内容跟着界面语言变；真引擎里这一行无害
    void version
    ;(async () => {
      try {
        const list = await api('houses', {})
        const target = list.find((x) => x.id === id) ?? list[0]
        if (!target) throw new BackendError({ code: 'not_found', message: 'no teahouse on this device' })
        const h = await api('house', { house: target.id })
        houses = list
        house = h
        failed = null
      } catch (e) {
        failed = e instanceof BackendError ? e.error : { code: 'internal', message: String(e) }
      }
    })()
  })
  let section = $derived(
    route.page === 'problems' || route.page === 'problem' ? 'problems'
    : route.page === 'proposals' || route.page === 'proposal' || route.page === 'propose' ? 'proposals'
    : route.page === 'board' ? 'board' : route.page === 'archive' ? 'archive' : 'house'
  ) as 'house' | 'board' | 'problems' | 'proposals' | 'archive'
</script>

{#if house}
<Rail {house} {houses} {section} />
<main>
  <ErrorBar />
  {#key hash + i18n.lang + house.id}
    <div class="page">
      {#if route.page === 'problems'}
        <Problems {house} view={route.arg === 'kanban' ? 'kanban' : 'list'} />
      {:else if route.page === 'board'}
        <Board {house} {refresh} />
      {:else if route.page === 'archive'}
        <Archive {house} id={route.arg || undefined} />
      {:else if route.page === 'propose'}
        <Propose {house} arg={route.arg} />
      {:else if route.page === 'problem'}
        <ProblemView {house} id={route.arg ?? '0001'} />
      {:else if route.page === 'proposals' || route.page === 'proposal'}
        <Proposal {house} id={route.page === 'proposal' ? route.arg : undefined} />
      {:else}
        <House {house} />
      {/if}
    </div>
  {/key}
</main>
{#if route.page === 'house'}
  {#key house.id}<Tour {house} />{/key}
{/if}
{:else if failed}
  <!-- 读不到茶馆（比如引擎没启动）：说清楚，给一个重试 -->
  <div class="failed">
    <h1>{t('err.loadFailed')}</h1>
    <p>{errorText(failed)}</p>
    <p class="detail">{failed.message}</p>
    <button class="secondary" onclick={refresh}>{t('err.retry')}</button>
  </div>
{/if}

<style>
  main { margin-left: var(--rail); min-height: 100vh; }
  .failed { padding: var(--pad-top) var(--pad-x); display: flex; flex-direction: column; align-items: flex-start; gap: 16px; max-width: 40em; }
  .failed h1 { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(32px, 4vw, 56px); letter-spacing: var(--track); }
  .failed .detail { font-size: 13px; color: var(--ink-3); font-family: var(--latin); overflow-wrap: anywhere; }
  .page { animation: enter 360ms var(--ease) both; }
  @keyframes enter { from { opacity: 0; } to { opacity: 1; } }
</style>
