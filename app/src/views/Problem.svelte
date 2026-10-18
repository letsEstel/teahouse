<script lang="ts">
  import Shape from '../lib/Shape.svelte'
  import StatusMark from '../lib/StatusMark.svelte'
  import WorkGlyph from '../lib/WorkGlyph.svelte'
  import ParkingMap from '../lib/ParkingMap.svelte'
  import { problems, byId, kindName, roleName } from '../data/fake'

  let { id }: { id: string } = $props()
  let p = $derived(problems.find((x) => x.id === id) ?? problems[0])
  let author = $derived(byId(p.by))
  let feature = $derived(p.works[0])
  let rest = $derived(p.works.slice(1))
</script>

<div class="wrap">
  <a class="back label" href="#/problems">← 全部问题</a>

  <header>
    <div class="no num">{p.id}</div>
    <div class="head">
      <StatusMark status={p.status} />
      <h1>{p.title}</h1>
      <div class="meta">
        <span><Shape role={author.role} size={10} />{author.name} 提出 · {roleName[author.role]}</span>
        <span class="num">{p.opened} 起</span>
        <span><b class="num">{p.works.length}</b> 份成果</span>
      </div>
    </div>
  </header>

  <div class="cols">
    <section class="brief">
      {#each p.body as b, i}
        <div class="para">
          <div class="label">{String(i + 1).padStart(2, '0')} · {b.heading}</div>
          <p>{b.text}</p>
        </div>
      {/each}

      <div class="label tl-head">经过</div>
      <ol class="timeline">
        {#each p.events as e}
          {@const m = byId(e.by)}
          <li>
            <span class="node"><Shape role={m.role} size={12} /></span>
            <div>
              <div class="ev"><b>{m.name}</b> {e.text}</div>
              <div class="when num">{e.when}</div>
            </div>
          </li>
        {/each}
      </ol>
    </section>

    <section class="works">
      <div class="works-head">
        <h2>成果</h2>
        <button class="add"><i>+</i>添加成果</button>
      </div>

      <div class="grid">
        {#if feature}
          {@const fm = byId(feature.by)}
          <article class="tile feature">
            <div class="preview"><ParkingMap /></div>
            <div class="tile-foot">
              <div>
                <div class="kind label">{kindName[feature.kind]} · 最新</div>
                <div class="t">{feature.title}</div>
                <div class="note">{feature.note}</div>
              </div>
              <div class="by"><Shape role={fm.role} size={10} />{fm.name}<span class="num">{feature.when}</span></div>
            </div>
          </article>
        {/if}

        {#each rest as w}
          {@const wm = byId(w.by)}
          <article class="tile">
            <div class="top">
              <WorkGlyph kind={w.kind} size={32} />
              <span class="kind label">{kindName[w.kind]}</span>
            </div>
            <div class="t">{w.title}</div>
            {#if w.note}<div class="note">{w.note}</div>{/if}
            <div class="by"><Shape role={wm.role} size={10} />{wm.name}<span class="num">{w.when}</span></div>
          </article>
        {/each}
      </div>
    </section>
  </div>
</div>

<style>
  .wrap { padding: 48px 72px 80px 64px; }
  .back { display: inline-block; transition: color 200ms var(--ease); }
  .back:hover { color: var(--ink); }

  header {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 40px;
    align-items: end;
    margin-top: 28px;
    padding-bottom: 32px;
    border-bottom: 2px solid var(--ink);
  }
  .no { font-size: 168px; font-weight: 300; line-height: 0.78; letter-spacing: -0.04em; color: var(--ink); }
  .head { padding-bottom: 2px; }
  h1 { font-family: var(--han); font-size: 48px; font-weight: 900; line-height: 1.15; margin: 14px 0 14px; letter-spacing: 0.02em; }
  .meta { display: flex; gap: 28px; font-size: 13px; color: var(--ink-2); }
  .meta span { display: inline-flex; align-items: center; gap: 8px; }
  .meta b { font-weight: 500; color: var(--ink); }

  .cols { display: grid; grid-template-columns: 340px minmax(0, 1fr); gap: 64px; margin-top: 40px; }

  .para { margin-bottom: 28px; }
  .para p { margin-top: 8px; font-size: 15px; line-height: 1.85; }

  .tl-head { margin-top: 44px; padding-bottom: 10px; border-bottom: 1px solid var(--ink); }
  .timeline { list-style: none; position: relative; padding-top: 18px; }
  .timeline::before { content: ''; position: absolute; left: 5px; top: 24px; bottom: 28px; width: 2px; background: var(--ink); }
  .timeline li { display: grid; grid-template-columns: 12px 1fr; gap: 18px; padding-bottom: 18px; position: relative; }
  .node { padding-top: 5px; background: var(--paper); box-shadow: 0 0 0 3px var(--paper); height: fit-content; position: relative; }
  .ev { font-size: 14px; line-height: 1.5; color: var(--ink-2); }
  .ev b { color: var(--ink); }
  .timeline .when { font-size: 12px; color: var(--ink-3); margin-top: 2px; }

  .works-head { display: flex; justify-content: space-between; align-items: center; padding-bottom: 16px; }
  h2 { font-family: var(--han); font-size: 28px; font-weight: 900; letter-spacing: 0.1em; }
  .add {
    display: flex; align-items: center; gap: 12px; height: 40px; padding-right: 18px;
    border: 2px solid var(--ink); font-size: 14px; font-weight: 700; letter-spacing: 0.08em;
    transition: background 200ms var(--ease), color 200ms var(--ease);
  }
  .add i { width: 36px; height: 36px; display: grid; place-items: center; background: var(--ink); color: var(--paper); font-style: normal; font-family: var(--latin); font-size: 22px; font-weight: 300; }
  .add:hover { background: var(--ink); color: var(--paper); }

  .grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    border-top: 2px solid var(--ink);
    border-left: 1px solid var(--ink);
  }
  .tile {
    border-right: 1px solid var(--ink);
    border-bottom: 1px solid var(--ink);
    padding: 20px 20px 18px;
    min-height: 196px;
    display: flex;
    flex-direction: column;
    transition: background 200ms var(--ease);
    cursor: pointer;
  }
  .tile:hover { background: var(--paper-deep); }
  .top { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 22px; }
  .t { font-family: var(--han); font-size: 16px; font-weight: 700; line-height: 1.4; }
  .note { font-size: 13px; color: var(--ink-2); margin-top: 4px; line-height: 1.5; }
  .by { margin-top: auto; padding-top: 14px; display: flex; align-items: center; gap: 8px; font-size: 12px; font-weight: 500; }
  .by span { margin-left: auto; color: var(--ink-3); font-weight: 400; }

  .feature { grid-column: span 3; padding: 0; min-height: 0; }
  .preview { padding: 28px 28px 18px; border-bottom: 1px solid var(--ink); }
  .tile-foot { display: grid; grid-template-columns: 1fr auto; align-items: end; padding: 16px 28px 18px; gap: 20px; }
  .tile-foot .t { font-size: 20px; margin-top: 4px; }
  .tile-foot .by { padding: 0; }
  .tile-foot .by span { margin-left: 10px; }
  .feature .kind { color: var(--red); }
</style>
