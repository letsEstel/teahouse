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
  import Join from './views/Join.svelte'
  import ErrorBar from './lib/ErrorBar.svelte'
  import { startAutoSync } from './api/autosync'
  import Sending from './lib/Sending.svelte'
  import StatusBar from './lib/StatusBar.svelte'
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
    // #/join 是入座页，不属于任何一间茶馆
    if (h === 'join') return { houseId: '', page: 'join', arg: undefined }
    return { houseId: h, page: page || 'house', arg: arg ? decodeURIComponent(arg) : arg }
  })

  // 数据只经过 api()（src/api/backend.ts）。写入之后调用 refresh() 重新读取。
  let houses = $state<HouseSummary[]>([])
  let house = $state<HouseData | null>(null)
  let version = $state(0)
  let failed = $state<ApiError | null>(null)
  let loaded = $state(false)
  const refresh = () => version++
  // 自动同步：前台每 60 秒、启动时、回到前台时；有新内容就重新读取（./api/autosync.ts）
  $effect(() => startAutoSync(refresh, () => house?.id))
  $effect(() => {
    const id = route.houseId
    void i18n.lang  // 假数据的内容跟着界面语言变；真引擎里这一行无害
    void version
    ;(async () => {
      try {
        const list = await api('houses', {})
        houses = list
        // 这台设备上还没有任何茶馆：直接到入座页
        if (!list.length) { house = null; failed = null; loaded = true; return }
        const target = list.find((x) => x.id === id) ?? list[0]
        const h = await api('house', { house: target.id })
        houses = list
        house = h
        failed = null
        loaded = true
      } catch (e) {
        failed = e instanceof BackendError ? e.error : { code: 'internal', message: String(e) }
      }
    })()
  })
  // 已离座：引擎给的 me 在座位表里找不到（CLIENT-API 第 2 节）→ 整间茶馆只读
  let seated = $derived(!!house && house.members.some((m) => m.id === house!.me))
  let section = $derived(
    route.page === 'problems' || route.page === 'problem' ? 'problems'
    : route.page === 'proposals' || route.page === 'proposal' || route.page === 'propose' ? 'proposals'
    : route.page === 'board' ? 'board' : route.page === 'archive' ? 'archive' : 'house'
  ) as 'house' | 'board' | 'problems' | 'proposals' | 'archive'
</script>

{#if route.page === 'join' || (!house && !failed && loaded && !houses.length)}
  <main class="solo">
    <ErrorBar />
    <Join {houses} onjoined={(seat) => { location.hash = `#/${seat.id}/house`; refresh() }} />
  </main>
{:else if house}
<Rail {house} {houses} {section} {seated} />
<main class:readonly={!seated}>
  <ErrorBar />
  <StatusBar {house} {seated} {refresh} />
  {#key hash + i18n.lang + house.id}
    <div class="page">
      {#if route.page === 'problems'}
        <Problems {house} {refresh} view={route.arg === 'kanban' ? 'kanban' : 'list'} compose={route.arg === 'new'} />
      {:else if route.page === 'board'}
        <Board {house} {refresh} />
      {:else if route.page === 'archive'}
        <Archive {house} id={route.arg || undefined} />
      {:else if route.page === 'propose'}
        <Propose {house} arg={route.arg} />
      {:else if route.page === 'problem'}
        <ProblemView {house} id={route.arg ?? '0001'} />
      {:else if route.page === 'proposals' || route.page === 'proposal'}
        <Proposal {house} {refresh} id={route.page === 'proposal' ? route.arg : undefined} />
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

<Sending />

<style>
  main { margin-left: var(--rail); min-height: 100vh; }
  main.solo { margin-left: 0; }
  .failed { padding: var(--pad-top) var(--pad-x); display: flex; flex-direction: column; align-items: flex-start; gap: 16px; max-width: 40em; }
  .failed h1 { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(32px, 4vw, 56px); letter-spacing: var(--track); }
  .failed .detail { font-size: 13px; color: var(--ink-3); font-family: var(--latin); overflow-wrap: anywhere; }
  .page { animation: enter 360ms var(--ease) both; }
  @keyframes enter { from { opacity: 0; } to { opacity: 1; } }
</style>
