<script lang="ts">
  import Rail from './lib/Rail.svelte'
  import Tour from './lib/Tour.svelte'
  import House from './views/House.svelte'
  import Problems from './views/Problems.svelte'
  import ProblemView from './views/Problem.svelte'
  import Proposal from './views/Proposal.svelte'
  import { findHouse } from './data'
  import { i18n } from './i18n/index.svelte'

  // 路由：#/<茶馆>/<界面>/<参数>，例如 #/wutong/problem/0001
  let hash = $state(location.hash)
  $effect(() => {
    const on = () => { hash = location.hash; window.scrollTo(0, 0) }
    addEventListener('hashchange', on)
    return () => removeEventListener('hashchange', on)
  })

  let route = $derived.by(() => {
    const [, h, page, arg] = hash.split('/')
    return { house: findHouse(h), page: page || 'house', arg }
  })
  let house = $derived(route.house)
  let section = $derived(
    route.page === 'problems' || route.page === 'problem' ? 'problems'
    : route.page === 'proposals' || route.page === 'proposal' ? 'proposals' : 'house'
  ) as 'house' | 'problems' | 'proposals'
</script>

<Rail {house} {section} />
<main>
  {#key hash + i18n.lang}
    <div class="page">
      {#if route.page === 'problems'}
        <Problems {house} />
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

<style>
  main { margin-left: var(--rail); min-height: 100vh; }
  .page { animation: enter 360ms var(--ease) both; }
  @keyframes enter { from { opacity: 0; } to { opacity: 1; } }
</style>
