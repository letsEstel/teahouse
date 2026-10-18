<script lang="ts">
  import Shape from '../lib/Shape.svelte'
  import WorkGlyph from '../lib/WorkGlyph.svelte'
  import { proposals, problems, byId, kindName, roleName } from '../data/fake'

  let { id }: { id: string } = $props()
  let p = $derived(proposals.find((x) => x.id === id) ?? proposals[0])
  let author = $derived(byId(p.by))
  let problem = $derived(problems.find((x) => x.id === p.problem)!)
  let state = $state<'pending' | 'merged' | 'rejected'>('pending')

  const history = [
    { title: '地锁采购报价', by: 'li', result: '已合并', when: '10.12' },
    { title: '楼道灯维修费用分摊', by: 'wu', result: '已合并', when: '07.20' },
    { title: '停车位按楼栋抽签', by: 'zheng', result: '未采纳', when: '06.03' },
  ]
  const tally = (f: (typeof p.files)[number]) => ({
    add: f.lines.filter((l) => l.kind === 'add').length,
    del: f.lines.filter((l) => l.kind === 'del').length,
  })
</script>

<div class="wrap">
  <aside class="queue">
    <div class="label">03 · 提案</div>
    <h1>提案</h1>

    <div class="label sub">待审 <span class="num">{proposals.length}</span></div>
    <ul>
      {#each proposals as q}
        {@const m = byId(q.by)}
        <li class:on={q.id === p.id}>
          <a href="#/proposal/{q.id}">
            <span class="q-no num">{q.problem}</span>
            <span class="q-t">{q.title}</span>
            <span class="q-by"><Shape role={m.role} size={9} />{m.name} · <span class="num">{q.when}</span></span>
          </a>
        </li>
      {/each}
    </ul>

    <div class="label sub">已处理</div>
    <ul class="done">
      {#each history as h}
        <li>
          <span class="q-t">{h.title}</span>
          <span class="q-by">{byId(h.by).name} · {h.result} · <span class="num">{h.when}</span></span>
        </li>
      {/each}
    </ul>
  </aside>

  <section class="review" class:merged={state === 'merged'} class:rejected={state === 'rejected'}>
    <div class="crumb label">
      面向问题 <a href="#/problem/{problem.id}"><span class="num">{problem.id}</span> {problem.title}</a>
    </div>

    <h2>{p.title}</h2>

    <div class="who">
      <Shape role={author.role} size={14} />
      <b>{author.name}</b><span>{roleName[author.role]}</span>
      <span class="num">{p.when}</span>
      <span>改动 <b class="num">{p.files.length}</b> 份成果</span>
    </div>

    {#if p.reason}
      <blockquote>{p.reason}</blockquote>
    {/if}

    {#each p.files as f}
      {@const t = tally(f)}
      <div class="file">
        <div class="file-head">
          <WorkGlyph kind={f.kind} size={24} />
          <span class="kind label">{kindName[f.kind]}</span>
          <span class="ft">{f.title}</span>
          <span class="tally num"><em class="a">+{t.add}</em><em class="d">−{t.del}</em></span>
        </div>
        <div class="lines">
          {#each f.lines as l, i}
            <div class="line {l.kind}">
              <span class="ln num">{String(i + 1).padStart(2, '0')}</span>
              <span class="mk">{l.kind === 'add' ? '+' : l.kind === 'del' ? '−' : ''}</span>
              <span class="tx">{l.text || ' '}</span>
            </div>
          {/each}
        </div>
      </div>
    {/each}

    <div class="actions">
      {#if state === 'pending'}
        <p class="hint">合并后，这些改动成为茶馆的正式成果，所有座位上的人都会看到。</p>
        <button class="reject" onclick={() => (state = 'rejected')}>不采纳</button>
        <button class="merge" onclick={() => (state = 'merged')}><i></i>合并</button>
      {:else}
        <div class="verdict">
          <i></i>
          <div>
            <b>{state === 'merged' ? '已合并入茶馆' : '未采纳'}</b>
            <span>{state === 'merged' ? '成果已更新 · 刚刚' : `已告知${author.name} · 刚刚`}</span>
          </div>
          <button class="undo label" onclick={() => (state = 'pending')}>撤回</button>
        </div>
      {/if}
    </div>
  </section>
</div>

<style>
  .wrap { display: grid; grid-template-columns: 300px minmax(0, 1fr); min-height: 100vh; }

  .queue { border-right: 1px solid var(--ink); padding: 64px 0 40px; }
  .queue > .label { padding: 0 32px; }
  h1 { font-family: var(--han); font-weight: 900; font-size: 56px; line-height: 1; letter-spacing: 0.06em; margin: 22px 32px 48px; }
  .sub { padding: 0 32px 10px !important; border-bottom: 1px solid var(--ink); margin-top: 8px; }
  .sub .num { color: var(--red); margin-left: 6px; }
  ul { list-style: none; margin-bottom: 40px; }
  li { border-bottom: 1px solid color-mix(in srgb, var(--ink) 18%, transparent); }
  li a, .done li { display: flex; flex-direction: column; gap: 4px; padding: 16px 32px; position: relative; }
  li a:hover { background: var(--paper-deep); }
  li.on a { background: var(--ink); color: var(--paper); }
  li.on a::before { content: ''; position: absolute; left: 0; top: 0; bottom: 0; width: 8px; background: var(--red); }
  .q-no { font-size: 12px; color: var(--ink-3); }
  .on .q-no { color: var(--paper); opacity: 0.6; }
  .q-t { font-family: var(--han); font-size: 15px; font-weight: 700; line-height: 1.45; }
  .q-by { font-size: 12px; color: var(--ink-2); display: flex; align-items: center; gap: 6px; }
  .on .q-by { color: var(--paper); opacity: 0.8; }
  .done .q-t { font-weight: 500; color: var(--ink-2); }
  .done .q-by { color: var(--ink-3); }

  .review { padding: 64px 72px 0 56px; display: flex; flex-direction: column; }
  .crumb a { color: var(--ink); margin-left: 8px; letter-spacing: 0.05em; border-bottom: 1px solid var(--ink); }
  h2 { font-family: var(--han); font-weight: 900; font-size: 44px; line-height: 1.2; margin: 18px 0 18px; max-width: 18em; letter-spacing: 0.01em; }
  .who { display: flex; align-items: center; gap: 12px; font-size: 13px; color: var(--ink-2); }
  .who b { color: var(--ink); font-size: 14px; }
  .who > span + span { padding-left: 12px; border-left: 1px solid var(--ink-3); }

  blockquote {
    margin: 28px 0 8px;
    padding: 2px 0 2px 24px;
    border-left: 4px solid var(--blue);
    font-size: 15px;
    line-height: 1.85;
    max-width: 44em;
  }

  .file { margin-top: 32px; border-top: 2px solid var(--ink); }
  .file-head { display: flex; align-items: center; gap: 14px; padding: 14px 0; border-bottom: 1px solid var(--ink); }
  .ft { font-family: var(--han); font-size: 17px; font-weight: 700; }
  .tally { margin-left: auto; display: flex; gap: 14px; font-size: 14px; }
  .tally em { font-style: normal; font-weight: 500; }
  .tally .a { color: var(--blue); }
  .tally .d { color: var(--red); }

  .lines { padding: 6px 0 4px; }
  .line {
    display: grid;
    grid-template-columns: 40px 28px 1fr;
    align-items: baseline;
    font-size: 15px;
    line-height: 1.75;
    padding: 3px 0;
    position: relative;
    transition: opacity 400ms var(--ease), max-height 500ms var(--ease), padding 500ms var(--ease);
    max-height: 4em;
  }
  .ln { font-size: 11px; color: var(--ink-3); }
  .mk { font-family: var(--latin); font-weight: 500; text-align: center; }
  .line.add::before, .line.del::before { content: ''; position: absolute; left: 30px; top: 4px; bottom: 4px; width: 4px; }
  .line.add::before { background: var(--blue); }
  .line.add .mk { color: var(--blue); }
  .line.add .tx { font-weight: 500; }
  .line.del::before { background: var(--red); }
  .line.del .mk { color: var(--red); }
  .line.del .tx { color: var(--ink-2); text-decoration: line-through; text-decoration-color: var(--red); text-decoration-thickness: 1.5px; }

  /* 合并：删去的行收起，新增的行落定为墨色 */
  .merged .line.del { opacity: 0; max-height: 0; padding: 0; overflow: hidden; }
  .merged .line.add::before { background: var(--ink); }
  .merged .line.add .mk { color: var(--ink); }
  .rejected .line.add { opacity: 0.35; }

  .actions {
    position: sticky;
    bottom: 0;
    margin-top: 48px;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0;
    padding: 20px 0 32px;
    background: var(--paper);
    border-top: 2px solid var(--ink);
  }
  .hint { margin-right: auto; font-size: 13px; color: var(--ink-2); max-width: 30em; }
  .reject, .merge { height: 56px; font-size: 16px; font-weight: 700; letter-spacing: 0.2em; padding: 0 36px; }
  .reject { border: 2px solid var(--ink); border-right: 0; transition: background 200ms var(--ease); }
  .reject:hover { background: var(--paper-deep); }
  .merge { background: var(--ink); color: var(--paper); display: flex; align-items: center; gap: 16px; padding-left: 28px; padding-right: 44px; border: 2px solid var(--ink); }
  .merge i { width: 14px; height: 14px; border-radius: 50%; background: var(--blue); transition: transform 300ms var(--ease); }
  .merge:hover i { transform: scale(1.4); }

  .verdict { display: flex; align-items: center; gap: 20px; width: 100%; animation: land 520ms var(--ease) both; }
  .verdict i { width: 56px; height: 56px; background: var(--blue); flex: none; }
  .rejected .verdict i { background: none; border: 2px solid var(--ink); }
  .verdict b { display: block; font-family: var(--han); font-size: 22px; font-weight: 900; letter-spacing: 0.08em; }
  .verdict span { font-size: 13px; color: var(--ink-2); }
  .undo { margin-left: auto; border-bottom: 1px solid var(--ink-2); }
  @keyframes land { from { opacity: 0; transform: translateX(-12px); } to { opacity: 1; transform: none; } }
</style>
