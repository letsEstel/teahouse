<script lang="ts">
  import Shape from '../lib/Shape.svelte'
  import AiMark from '../lib/AiMark.svelte'
  import StatusMark from '../lib/StatusMark.svelte'
  import Editor from '../lib/Editor.svelte'
  import { write } from '../api/backend'
  import { member, self, pad, type House, type Status } from '../api/view'
  import { t, when } from '../i18n/index.svelte'

  // 列表和看板是同一份数据的两种视图
  let { house, view = 'list', refresh, compose = false }: { house: House; view?: 'list' | 'kanban'; refresh?: () => void; compose?: boolean } = $props()
  const columns: Status[] = ['open', 'active', 'solved']
  let me = $derived(self(house))

  let filter = $state<'all' | Status>('all')
  const tabs: ('all' | Status)[] = ['all', 'active', 'open', 'solved']
  let list = $derived(filter === 'all' ? house.problems : house.problems.filter((p) => p.status === filter))
  const pending = (id: string) => house.proposals.filter((p) => p.problem === id).length
  const n = (s: 'all' | Status) => (s === 'all' ? house.problems.length : house.problems.filter((p) => p.status === s).length)

  // 茶客不能直接写问题，只能提议
  let action = $derived(
    me.role === 'guest'
      ? { label: t('act.suggestProblem'), hint: t('act.suggestProblemHint'), href: `#/${house.id}/propose/new` }
      : { label: t('act.newProblem'), hint: t('act.newProblemHint'), href: undefined }
  )

  // 提出问题（老板、伙计）：标题 + 正文。正文里第一个 "## " 之前是摘要，之后每个 "## 小标题" 是一节。
  // 走引擎的 openProblem { house, title, body }，成功后跳到新问题的页面。
  // #/<茶馆>/problems/new：直接打开（座位页上伙计的"提出问题"按钮）
  // 页面随地址重建，只取一次初始值就够了
  // svelte-ignore state_referenced_locally
  let composing = $state(compose)
  let title = $state('')
  let text = $state('')
  let busy = $state(false)
  function sections(src: string) {
    const body: { heading: string; text: string }[] = [{ heading: '', text: '' }]
    for (const line of src.split('\n')) {
      const h = /^##\s+(.*)$/.exec(line)
      if (h) body.push({ heading: h[1].trim(), text: '' })
      else body[body.length - 1].text += (body[body.length - 1].text ? '\n' : '') + line
    }
    return body.map((b) => ({ heading: b.heading, text: b.text.trim() })).filter((b) => b.heading || b.text)
  }
  async function submit() {
    if (!title.trim() || busy) return
    busy = true
    const r = await write('openProblem', { house: house.id, title: title.trim(), body: sections(text) })
    busy = false
    if (!r) return
    composing = false
    refresh?.()
    // 引擎在 id 里给出新问题的编号；变成提案时（角色刚变动）去提案页
    location.hash = r.becameProposal ? `#/${house.id}/proposals` : `#/${house.id}/problem/${r.id}`
  }
</script>

<div class="wrap">
  <header>
    <div class="titles">
      <div class="label">03 · {t('nav.problems')}</div>
      <h1>{t('nav.problems')}<sup class="num">{pad(house.problems.length)}</sup></h1>
      <p class="lede">{t('problems.lede')}</p>
    </div>
    <div class="action" data-write>
      {#if action.href}
        <a class="primary" href={action.href}><i>+</i><span>{action.label}</span></a>
      {:else}
        <button class="primary" aria-expanded={composing} onclick={() => (composing = !composing)}><i>+</i><span>{action.label}</span></button>
      {/if}
      <p class="consequence">{action.hint}</p>
    </div>
  </header>

  {#if composing}
    <form class="compose" data-write onsubmit={(e) => { e.preventDefault(); submit() }}>
      <label class="label" for="problem-title">{t('newProblem.title')}</label>
      <input id="problem-title" class="title-in" bind:value={title} placeholder={t('newProblem.titlePh')} />
      <span class="label">{t('newProblem.body')}</span>
      <Editor value="" placeholder={t('newProblem.bodyPh')} onchange={(v) => (text = v)} />
      <p class="consequence">{t('newProblem.hint')}</p>
      <div class="save">
        <button type="submit" class="primary" disabled={!title.trim() || busy}><span>{t('newProblem.submit')}</span></button>
        <button type="button" class="secondary" onclick={() => (composing = false)}>{t('newProblem.cancel')}</button>
      </div>
    </form>
  {/if}

  <div class="bar">
    <div class="tabs" role="tablist">
      {#if view === 'list'}
        {#each tabs as k}
          <button role="tab" aria-selected={filter === k} class:on={filter === k} onclick={() => (filter = k)}>
            {t(`status.${k}`)}<span class="num">{n(k)}</span>
          </button>
        {/each}
      {/if}
    </div>
    <div class="views" role="group">
      <a href="#/{house.id}/problems" class:on={view === 'list'} aria-current={view === 'list' ? 'page' : undefined}>
        <i class="vg list"><b></b><b></b><b></b></i>{t('view.list')}
      </a>
      <a href="#/{house.id}/problems/kanban" class:on={view === 'kanban'} aria-current={view === 'kanban' ? 'page' : undefined}>
        <i class="vg cols"><b></b><b></b><b></b></i>{t('view.kanban')}
      </a>
    </div>
  </div>

  {#if view === 'kanban'}
    <div class="kanban">
      {#each columns as c}
        {@const items = house.problems.filter((p) => p.status === c)}
        <section class="col">
          <div class="col-head"><StatusMark status={c} /><span class="num">{items.length}</span></div>
          {#each items as p (p.id)}
            {@const m = member(house, p.by)}
            {@const pr = pending(p.id)}
            <a class="card" class:solved={c === 'solved'} href="#/{house.id}/problem/{p.id}">
              <span class="num cno">{p.id}</span>
              <span class="ct">{p.title}</span>
              <span class="cmeta">
                <span><b class="num">{p.workCount}</b> {t('problems.works', { n: p.workCount })}</span>
                {#if pr}<span><em class="badge">{pr}</em> {t('problems.pending')}</span>{/if}
                <span class="by"><Shape role={m.role} size={9} />{m.name}<AiMark {m} /></span>
              </span>
            </a>
          {:else}
            <p class="empty">{t('kanban.empty')}</p>
          {/each}
        </section>
      {/each}
    </div>
  {:else}
  <div class="list">
    <ol>
      {#each list as p (p.id)}
        {@const m = member(house, p.by)}
        {@const pr = pending(p.id)}
        <li class:solved={p.status === 'solved'}>
          <a href="#/{house.id}/problem/{p.id}">
            <span class="no num">{p.id}</span>
            <span class="body">
              <span class="title">{p.title}</span>
              <span class="summary">{p.summary}</span>
            </span>
            <span class="meta">
              <span class="st"><StatusMark status={p.status} /></span>
              <span class="works"><b class="num">{p.workCount}</b> {t('problems.works', { n: p.workCount })}</span>
              <span class="pr">{#if pr}<em class="badge">{pr}</em> {t('problems.pending')}{/if}</span>
              <span class="by"><Shape role={m.role} size={10} />{m.name}<AiMark {m} /></span>
              <span class="when num">{when(p.updated)}</span>
            </span>
          </a>
        </li>
      {/each}
    </ol>
  </div>
  {/if}
</div>

<style>
  .compose { display: grid; gap: 10px; max-width: 52rem; margin: 0 0 clamp(28px, 4vw, 44px); padding: clamp(16px, 2.5vw, 24px); border: 2px solid var(--ink); }
  .title-in { border: 1.5px solid var(--ink); background: var(--paper); padding: 10px 14px; font: inherit; font-size: 20px; font-weight: 700; color: var(--ink); }
  .compose .save { display: flex; flex-wrap: wrap; gap: 10px; }
  .compose .primary:disabled { background: transparent; color: var(--ink-3); border-color: var(--ink-3); cursor: default; }
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

  .bar { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: flex-end; gap: 12px 24px; margin-top: 48px; border-bottom: 2px solid var(--ink); }
  .tabs { display: flex; flex-wrap: wrap; gap: 8px 36px; }
  .views { display: flex; margin-bottom: 10px; }
  .views a { display: inline-flex; align-items: center; gap: 8px; padding: 6px 14px; border: 1.5px solid var(--ink); font-size: 13px; font-weight: 500; margin-left: -1.5px; }
  .views a.on { background: var(--ink); color: var(--paper); }
  .vg { display: inline-flex; gap: 2px; width: 14px; height: 12px; }
  .vg b { flex: 1; background: currentColor; }
  .vg.list { flex-direction: column; }
  .vg.list b { flex: none; height: 2px; margin: 1px 0; }

  .kanban { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); border-left: 1px solid var(--ink); }
  .col { border-right: 1px solid var(--ink); border-bottom: 1px solid var(--ink); padding: 18px clamp(12px, 1.4vw, 20px) 24px; display: flex; flex-direction: column; gap: 12px; min-width: 0; }
  .col-head { display: flex; align-items: center; justify-content: space-between; padding-bottom: 12px; border-bottom: 1px solid var(--ink); }
  .col-head .num { font-size: 22px; font-weight: 300; }
  .card { display: flex; flex-direction: column; gap: 8px; padding: 14px 16px 14px; border: 1.5px solid var(--ink); background: var(--paper); transition: background 200ms var(--ease); }
  .card:hover { background: var(--paper-deep); }
  .cno { font-size: 13px; color: var(--ink-3); }
  .ct { font-family: var(--display); font-size: 16px; font-weight: 700; line-height: 1.4; letter-spacing: var(--track); }
  .cmeta { display: flex; flex-wrap: wrap; align-items: center; gap: 6px 14px; font-size: 12px; color: var(--ink-2); }
  .cmeta b { font-weight: 500; color: var(--ink); }
  .card.solved .ct { color: var(--ink-3); }
  .empty { font-size: 13px; color: var(--ink-3); padding: 8px 0; }
  @media (max-width: 700px) { .kanban { grid-template-columns: minmax(0, 1fr); } }
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
