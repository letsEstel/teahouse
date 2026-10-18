<script lang="ts">
  import Shape from '../lib/Shape.svelte'
  import StatusMark from '../lib/StatusMark.svelte'
  import { member, self, workCount, pad, type House, type Status } from '../data'
  import { t, L, when } from '../i18n/index.svelte'

  let { house }: { house: House } = $props()
  let me = $derived(self(house))

  let filter = $state<'all' | Status>('all')
  const tabs: ('all' | Status)[] = ['all', 'active', 'open', 'solved']
  let list = $derived(filter === 'all' ? house.problems : house.problems.filter((p) => p.status === filter))
  const pending = (id: string) => house.proposals.filter((p) => p.problem === id).length
  const n = (s: 'all' | Status) => (s === 'all' ? house.problems.length : house.problems.filter((p) => p.status === s).length)

  // 茶客不能直接写问题，只能提议
  let action = $derived(
    me.role === 'guest'
      ? { label: t('act.suggestProblem'), hint: t('act.suggestProblemHint') }
      : { label: t('act.newProblem'), hint: t('act.newProblemHint') }
  )
</script>

<div class="wrap">
  <header>
    <div class="titles">
      <div class="label">02 · {t('nav.problems')}</div>
      <h1>{t('nav.problems')}<sup class="num">{pad(house.problems.length)}</sup></h1>
      <p class="lede">{t('problems.lede')}</p>
    </div>
    <div class="action">
      <button class="primary"><i>+</i><span>{action.label}</span></button>
      <p class="consequence">{action.hint}</p>
    </div>
  </header>

  <div class="tabs" role="tablist">
    {#each tabs as k}
      <button role="tab" aria-selected={filter === k} class:on={filter === k} onclick={() => (filter = k)}>
        {t(`status.${k}`)}<span class="num">{n(k)}</span>
      </button>
    {/each}
  </div>

  <div class="list">
    <ol>
      {#each list as p (p.id)}
        {@const m = member(house, p.by)}
        {@const pr = pending(p.id)}
        <li class:solved={p.status === 'solved'}>
          <a href="#/{house.id}/problem/{p.id}">
            <span class="no num">{p.id}</span>
            <span class="body">
              <span class="title">{L(p.title)}</span>
              <span class="summary">{L(p.summary)}</span>
            </span>
            <span class="meta">
              <span class="st"><StatusMark status={p.status} /></span>
              <span class="works"><b class="num">{workCount(p)}</b> {t('problems.works', { n: workCount(p) })}</span>
              <span class="pr">{#if pr}<em class="badge">{pr}</em> {t('problems.pending')}{/if}</span>
              <span class="by"><Shape role={m.role} size={10} />{L(m.name)}</span>
              <span class="when num">{when(p.updated)}</span>
            </span>
          </a>
        </li>
      {/each}
    </ol>
  </div>
</div>

<style>
  .wrap { padding: var(--pad-top) var(--pad-x) 80px; }

  header { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: flex-end; gap: 28px 48px; }
  .titles { min-width: 0; }
  h1 {
    font-family: var(--display);
    font-weight: var(--heavy);
    font-size: clamp(52px, 5.8vw, 84px);
    line-height: 1;
    margin-top: 22px;
    letter-spacing: var(--track);
  }
  sup { font-size: 0.3em; font-weight: 300; vertical-align: top; margin-left: 10px; position: relative; top: 0.15em; letter-spacing: 0; }
  .lede { margin-top: 18px; color: var(--ink-2); font-size: 16px; letter-spacing: 0.04em; }
  .action { display: flex; flex-direction: column; align-items: flex-start; gap: 10px; max-width: 26em; }

  .tabs { display: flex; flex-wrap: wrap; gap: 8px 36px; margin-top: 48px; border-bottom: 2px solid var(--ink); }
  .tabs button {
    font-size: 15px; font-weight: 500; padding: 0 0 14px; position: relative; color: var(--ink-2);
    display: flex; gap: 8px; align-items: baseline;
  }
  .tabs button span { font-size: 12px; color: var(--ink-3); }
  .tabs button.on { color: var(--ink); font-weight: 700; }
  .tabs button.on::after { content: ''; position: absolute; left: 0; right: 0; bottom: -2px; height: 6px; background: var(--ink); }

  .list { container-type: inline-size; }
  /* 宽：一行一个问题，各列用 subgrid 对齐 */
  ol {
    list-style: none;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) repeat(5, auto);
    column-gap: clamp(16px, 2cqi, 28px);
  }
  li { grid-column: 1 / -1; display: grid; grid-template-columns: subgrid; border-bottom: 1px solid var(--ink); }
  li a { grid-column: 1 / -1; display: grid; grid-template-columns: subgrid; align-items: center; padding: 24px 0; transition: background 200ms var(--ease); }
  li a:hover { background: var(--paper-deep); }
  .meta { display: contents; }

  .no { font-size: clamp(32px, 3.2cqi, 44px); font-weight: 300; line-height: 1; letter-spacing: -0.01em; min-width: 2.6em; }
  .body { display: flex; flex-direction: column; gap: 6px; min-width: 0; }
  .title { font-family: var(--display); font-size: clamp(18px, 1.6vw, 22px); font-weight: 700; line-height: 1.3; letter-spacing: var(--track); }
  .summary { font-size: 14px; color: var(--ink-2); line-height: 1.5; }

  .works { font-size: 13px; color: var(--ink-2); white-space: nowrap; }
  .works b { font-size: 22px; font-weight: 400; color: var(--ink); margin-right: 2px; }
  .pr { font-size: 13px; font-weight: 500; white-space: nowrap; }
  .by { display: flex; align-items: center; gap: 8px; font-size: 13px; font-weight: 500; white-space: nowrap; }
  .when { font-size: 12px; color: var(--ink-3); text-align: right; white-space: nowrap; }

  .solved .no, .solved .title { color: var(--ink-3); }

  /* 窄：元信息折到标题下面 */
  @container (max-width: 900px) {
    ol { display: block; }
    li { display: block; }
    li a { display: grid; grid-template-columns: auto minmax(0, 1fr); column-gap: 20px; align-items: start; padding: 20px 0; }
    .no { min-width: 0; font-size: 30px; }
    .meta { display: flex; flex-wrap: wrap; align-items: center; gap: 8px 22px; grid-column: 2; margin-top: 12px; }
    .pr:empty { display: none; }
    .when { text-align: left; }
  }
</style>
