<script lang="ts">
  import Shape from '../lib/Shape.svelte'
  import StatusMark from '../lib/StatusMark.svelte'
  import { problems, proposals, byId, type Status } from '../data/fake'

  let filter = $state<'all' | Status>('all')
  const tabs: { key: 'all' | Status; name: string }[] = [
    { key: 'all', name: '全部' },
    { key: 'active', name: '进行中' },
    { key: 'open', name: '待认领' },
    { key: 'solved', name: '已办成' },
  ]
  let list = $derived(filter === 'all' ? problems : problems.filter((p) => p.status === filter))
  const pending = (id: string) => proposals.filter((p) => p.problem === id).length
  const n = (s: 'all' | Status) => (s === 'all' ? problems.length : problems.filter((p) => p.status === s).length)
</script>

<div class="wrap">
  <header>
    <div>
      <div class="label">02 · 问题</div>
      <h1>问题<sup class="num">{String(problems.length).padStart(2, '0')}</sup></h1>
      <p class="lede">每一次动作，都面向其中一个。</p>
    </div>
    <button class="new"><i>+</i>提出问题</button>
  </header>

  <div class="tabs">
    {#each tabs as t}
      <button class:on={filter === t.key} onclick={() => (filter = t.key)}>
        {t.name}<span class="num">{n(t.key)}</span>
      </button>
    {/each}
  </div>

  <ol>
    {#each list as p (p.id)}
      {@const m = byId(p.by)}
      <li class:solved={p.status === 'solved'}>
        <a href="#/problem/{p.id}">
          <span class="no num">{p.id}</span>
          <span class="body">
            <span class="title">{p.title}</span>
            <span class="summary">{p.summary}</span>
          </span>
          <span class="st"><StatusMark status={p.status} /></span>
          <span class="works"><b class="num">{p.works.length}</b> 成果</span>
          <span class="pr">
            {#if pending(p.id)}<em class="num">{pending(p.id)}</em> 提案待审{/if}
          </span>
          <span class="by"><Shape role={m.role} size={10} />{m.name}</span>
          <span class="when num">{p.updated}</span>
        </a>
      </li>
    {/each}
  </ol>
</div>

<style>
  .wrap { padding: 64px 72px 80px 64px; }

  header { display: flex; justify-content: space-between; align-items: flex-end; }
  h1 {
    font-family: var(--han);
    font-weight: 900;
    font-size: 84px;
    line-height: 1;
    margin-top: 22px;
    letter-spacing: 0.04em;
  }
  sup { font-size: 24px; font-weight: 300; vertical-align: top; margin-left: 10px; position: relative; top: 4px; letter-spacing: 0; }
  .lede { margin-top: 18px; color: var(--ink-2); font-size: 16px; letter-spacing: 0.06em; }

  .new {
    display: flex;
    align-items: center;
    gap: 14px;
    background: var(--ink);
    color: var(--paper);
    font-size: 15px;
    font-weight: 700;
    letter-spacing: 0.12em;
    padding: 0 28px 0 0;
    height: 56px;
    transition: background 200ms var(--ease);
  }
  .new i {
    width: 56px; height: 56px; display: grid; place-items: center;
    background: var(--red); font-style: normal; font-family: var(--latin); font-size: 28px; font-weight: 300;
  }
  .new:hover { background: #2a2a2a; }

  .tabs { display: flex; gap: 36px; margin-top: 56px; border-bottom: 2px solid var(--ink); }
  .tabs button {
    font-size: 15px; font-weight: 500; padding: 0 0 14px; position: relative; color: var(--ink-2);
    display: flex; gap: 8px; align-items: baseline;
  }
  .tabs button span { font-size: 12px; color: var(--ink-3); }
  .tabs button.on { color: var(--ink); font-weight: 700; }
  .tabs button.on::after {
    content: ''; position: absolute; left: 0; right: 0; bottom: -2px; height: 6px; background: var(--ink);
  }

  ol { list-style: none; }
  li { border-bottom: 1px solid var(--ink); }
  li a {
    display: grid;
    grid-template-columns: 132px minmax(0, 1fr) 120px 84px 104px 92px 72px;
    gap: 20px;
    align-items: center;
    padding: 26px 0;
    transition: background 200ms var(--ease), padding 240ms var(--ease);
  }
  li a:hover { background: var(--paper-deep); padding-left: 16px; }

  .no { font-size: 44px; font-weight: 300; line-height: 1; letter-spacing: -0.01em; }
  .body { display: flex; flex-direction: column; gap: 6px; min-width: 0; }
  .title { font-family: var(--han); font-size: 22px; font-weight: 700; line-height: 1.3; }
  .summary { font-size: 14px; color: var(--ink-2); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

  .works { font-size: 13px; color: var(--ink-2); }
  .works b { font-size: 22px; font-weight: 400; color: var(--ink); margin-right: 2px; }
  .pr { font-size: 13px; color: var(--red); font-weight: 500; white-space: nowrap; }
  .pr em { font-style: normal; background: var(--red); color: var(--paper); padding: 0 6px; margin-right: 4px; }
  .by { display: flex; align-items: center; gap: 8px; font-size: 13px; font-weight: 500; white-space: nowrap; }
  .when { font-size: 12px; color: var(--ink-3); text-align: right; white-space: nowrap; }

  .solved .no, .solved .title { color: var(--ink-3); }
</style>
