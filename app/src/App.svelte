<script lang="ts">
  import Rail from './lib/Rail.svelte'
  import House from './views/House.svelte'
  import Problems from './views/Problems.svelte'
  import ProblemView from './views/Problem.svelte'
  import Proposal from './views/Proposal.svelte'

  let hash = $state(location.hash || '#/house')
  $effect(() => {
    const on = () => { hash = location.hash || '#/house'; window.scrollTo(0, 0) }
    addEventListener('hashchange', on)
    return () => removeEventListener('hashchange', on)
  })

  let route = $derived.by(() => {
    const [, page, arg] = hash.split('/')
    return { page: page || 'house', arg }
  })
  let section = $derived(
    route.page === 'problems' || route.page === 'problem' ? 'problems'
    : route.page === 'proposal' ? 'proposals' : 'house'
  ) as 'house' | 'problems' | 'proposals'
</script>

<Rail {section} />
<main>
  {#key hash}
    <div class="page">
      {#if route.page === 'problems'}
        <Problems />
      {:else if route.page === 'problem'}
        <ProblemView id={route.arg ?? '0001'} />
      {:else if route.page === 'proposal'}
        <Proposal id={route.arg ?? 'p1'} />
      {:else}
        <House />
      {/if}
    </div>
  {/key}
</main>

<style>
  main { margin-left: var(--rail); min-height: 100vh; }
  .page { animation: enter 360ms var(--ease) both; }
  @keyframes enter { from { opacity: 0; } to { opacity: 1; } }
</style>
