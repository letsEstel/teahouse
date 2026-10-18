<script lang="ts">
  import Shape from '../lib/Shape.svelte'
  import StatusMark from '../lib/StatusMark.svelte'
  import WorkGlyph from '../lib/WorkGlyph.svelte'
  import ParkingMap from '../lib/ParkingMap.svelte'
  import Notes from '../lib/Notes.svelte'
  import Write from '../lib/Write.svelte'
  import { member, self, workCount, pad, type House, type Note } from '../data'
  import { t, L, when, date, NOW } from '../i18n/index.svelte'

  let { house, id }: { house: House; id: string } = $props()
  let p = $derived(house.problems.find((x) => x.id === id) ?? house.problems[0])
  let author = $derived(member(house, p.by))
  let me = $derived(self(house))
  let feature = $derived(p.works[0])
  let rest = $derived(p.works.slice(1))

  let action = $derived(
    me.role === 'guest'
      ? { label: t('act.proposeWork'), hint: t('act.proposeWorkHint'), href: `#/${house.id}/propose/${p.id}` }
      : { label: t('act.addWork'), hint: t('act.addWorkHint'), href: undefined }
  )

  // 问题下的讨论。M0：写下的只留在本页（M1 起每条留言是一次提交）
  let notes = $state<Note[]>([])
  $effect.pre(() => { notes = [...(p.notes ?? [])] })
  function write(text: string) {
    notes.push({ by: me.id, when: NOW.toISOString(), text: { zh: text, en: text } })
  }
</script>

<div class="wrap">
  <a class="back label" href="#/{house.id}/problems">← {t('problem.back')}</a>

  <header>
    <div class="no num">{p.id}</div>
    <div class="head">
      <StatusMark status={p.status} />
      <h1>{L(p.title)}</h1>
      <div class="meta">
        <span><Shape role={author.role} size={10} />{t('problem.raisedBy', { name: L(author.name) })} · {t(`role.${author.role}`)}</span>
        <span>{t('problem.since', { date: date(p.opened, 'short') })}</span>
        <span>{t('problem.works', { n: workCount(p) })}</span>
      </div>
    </div>
  </header>

  <div class="cols">
    <section class="brief">
      {#each p.body as b, i}
        <div class="para">
          <div class="label">{pad(i + 1)} · {L(b.heading)}</div>
          <p>{L(b.text)}</p>
        </div>
      {:else}
        <p class="quiet">{L(p.summary)}</p>
      {/each}

      {#if p.events.length}
        <div class="label tl-head">{t('problem.history')}</div>
        <ol class="timeline">
          {#each p.events as e}
            {@const m = member(house, e.by)}
            <li>
              <span class="node"><Shape role={m.role} size={12} /></span>
              <div>
                <div class="ev"><b>{L(m.name)}</b> {t(`ev.${e.act}`, { ref: e.ref ? L(e.ref) : '' })}</div>
                <div class="when">{when(e.when)}</div>
              </div>
            </li>
          {/each}
        </ol>
      {/if}
    </section>

    <section class="works">
      <div class="works-head">
        <h2>{t('problem.worksHead')}</h2>
        <div class="action">
          <p class="consequence">{action.hint}</p>
          {#if action.href}
            <a class="primary" href={action.href}><i>+</i><span>{action.label}</span></a>
          {:else}
            <button class="primary"><i>+</i><span>{action.label}</span></button>
          {/if}
        </div>
      </div>

      {#if workCount(p) === 0}
        <div class="empty">
          <svg width="48" height="48" viewBox="0 0 48 48" aria-hidden="true"><rect x="1.5" y="1.5" width="45" height="45" fill="none" stroke="var(--ink)" stroke-width="2" stroke-dasharray="6 5" /></svg>
          <p>{t('problem.empty')}</p>
        </div>
      {/if}

      <div class="grid">
        {#if feature}
          {@const fm = member(house, feature.by)}
          <article class="tile feature">
            {#if feature.preview}
              <div class="preview">
                {#if 'chart' in feature.preview}
                  <ParkingMap />
                {:else}
                  <blockquote>{L(feature.preview.excerpt)}</blockquote>
                {/if}
              </div>
            {/if}
            <div class="tile-foot">
              <div>
                <div class="kind label">{t(`kind.${feature.kind}`)} · {t('problem.latest')}</div>
                <div class="t">{L(feature.title)}</div>
                {#if feature.note}<div class="note">{L(feature.note)}</div>{/if}
              </div>
              <div class="by"><Shape role={fm.role} size={10} />{L(fm.name)}<span>{when(feature.when)}</span></div>
            </div>
          </article>
        {/if}

        {#each rest as w}
          {@const wm = member(house, w.by)}
          <article class="tile">
            <div class="top">
              <WorkGlyph kind={w.kind} size={32} />
              <span class="kind label">{t(`kind.${w.kind}`)}</span>
            </div>
            <div class="t hyphenate">{L(w.title)}</div>
            {#if w.note}<div class="note hyphenate">{L(w.note)}</div>{/if}
            <div class="by"><Shape role={wm.role} size={10} /><b>{L(wm.name)}</b><span>{when(w.when)}</span></div>
          </article>
        {/each}
      </div>

      <section class="talk">
        <div class="talk-head">
          <h2>{t('talk.title')}</h2>
          <p class="consequence">{t('talk.lede')}</p>
        </div>
        {#if notes.length}
          <Notes {house} {notes} />
        {:else}
          <p class="quiet none">{t('talk.none')}</p>
        {/if}
        <div class="talk-write"><Write placeholder={t('talk.placeholder')} onsubmit={write} /></div>
      </section>
    </section>
  </div>
</div>

<style>
  .wrap { padding: calc(var(--pad-top) * 0.75) var(--pad-x) 80px; }
  .back { display: inline-block; padding: 6px 0; }
  .back:hover { color: var(--ink); }

  header {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: clamp(24px, 3vw, 40px);
    align-items: end;
    margin-top: 22px;
    padding-bottom: 32px;
    border-bottom: 2px solid var(--ink);
  }
  .no { font-size: clamp(88px, 11vw, 168px); font-weight: 300; line-height: 0.78; letter-spacing: -0.04em; white-space: nowrap; }
  .head { min-width: 0; padding-bottom: 2px; }
  h1 {
    font-family: var(--display);
    font-size: clamp(30px, 3.3vw, 48px);
    font-weight: var(--heavy);
    line-height: 1.15;
    margin: 14px 0;
    letter-spacing: var(--track);
    max-width: 24em;
  }
  .meta { display: flex; flex-wrap: wrap; gap: 6px 28px; font-size: 13px; color: var(--ink-2); }
  .meta span { display: inline-flex; align-items: center; gap: 8px; }

  .cols { display: grid; grid-template-columns: minmax(240px, 340px) minmax(0, 1fr); gap: clamp(36px, 4.4vw, 64px); margin-top: 40px; }

  .para { margin-bottom: 28px; }
  .para p { margin-top: 8px; font-size: 15px; line-height: 1.85; }
  .quiet { color: var(--ink-2); line-height: 1.8; }

  .tl-head { margin-top: 44px; padding-bottom: 10px; border-bottom: 1px solid var(--ink); }
  .timeline { list-style: none; position: relative; padding-top: 18px; }
  .timeline::before { content: ''; position: absolute; left: 5px; top: 24px; bottom: 28px; width: 2px; background: var(--ink); }
  .timeline li { display: grid; grid-template-columns: 12px minmax(0, 1fr); gap: 18px; padding-bottom: 18px; position: relative; }
  .node { padding-top: 5px; background: var(--paper); outline: 3px solid var(--paper); height: fit-content; position: relative; }
  .ev { font-size: 14px; line-height: 1.5; color: var(--ink-2); }
  .ev b { color: var(--ink); }
  .timeline .when { font-family: var(--latin); font-size: 12px; color: var(--ink-3); margin-top: 2px; }

  .works { min-width: 0; }
  .works-head { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 16px 24px; padding-bottom: 18px; }
  h2 { font-family: var(--display); font-size: 28px; font-weight: var(--heavy); letter-spacing: var(--track-wide); }
  .action { display: flex; flex-wrap: wrap; align-items: center; justify-content: flex-end; gap: 10px 20px; }
  .action .consequence { text-align: right; max-width: 20em; }

  .empty { display: flex; align-items: center; gap: 20px; padding: 28px 0; border-top: 2px solid var(--ink); color: var(--ink-2); }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 220px), 1fr));
    border-left: 1px solid var(--ink);
  }
  .grid:not(:empty) { border-top: 2px solid var(--ink); }
  .tile {
    border-right: 1px solid var(--ink);
    border-bottom: 1px solid var(--ink);
    padding: 20px 20px 18px;
    min-height: 196px;
    min-width: 0;
    display: flex;
    flex-direction: column;
    transition: background 200ms var(--ease);
    cursor: pointer;
  }
  .tile:hover { background: var(--paper-deep); }
  .top { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; margin-bottom: 22px; }
  .t { font-family: var(--display); font-size: 16px; font-weight: 700; line-height: 1.4; letter-spacing: var(--track); }
  .note { font-size: 13px; color: var(--ink-2); margin-top: 4px; line-height: 1.5; }
  .by { margin-top: auto; padding-top: 14px; display: flex; flex-wrap: wrap; align-items: center; gap: 4px 8px; font-size: 12px; font-weight: 500; }
  .by span { margin-left: auto; color: var(--ink-3); font-weight: 400; font-family: var(--latin); white-space: nowrap; }

  .feature { grid-column: 1 / -1; padding: 0; min-height: 0; }
  .preview { padding: 28px 28px 22px; border-bottom: 1px solid var(--ink); }
  .preview > :global(figure) { max-width: 720px; }
  blockquote {
    white-space: pre-line;
    font-family: var(--display);
    font-size: clamp(16px, 1.35vw, 19px);
    font-weight: 500;
    line-height: 1.75;
    border-left: 4px solid var(--ink);
    padding-left: 22px;
    max-width: 44em;
  }
  .tile-foot { display: grid; grid-template-columns: minmax(0, 1fr) auto; align-items: end; padding: 16px 28px 18px; gap: 12px 20px; }
  .tile-foot .t { font-size: 20px; margin-top: 4px; }
  .tile-foot .by { padding: 0; }
  .tile-foot .by span { margin-left: 10px; }
  .feature .kind { color: var(--ink); }

  .talk { margin-top: 56px; max-width: 48em; }
  .talk-head { display: flex; flex-wrap: wrap; align-items: baseline; gap: 6px 24px; padding-bottom: 14px; border-bottom: 2px solid var(--ink); margin-bottom: 12px; }
  .none { padding: 8px 0 16px; }
  .talk-write { margin-top: 20px; }

  @media (max-width: 1279.98px) {
    .cols { grid-template-columns: minmax(0, 1fr); }
    .brief { max-width: 44em; }
  }
  @media (max-width: 700px) {
    header { grid-template-columns: minmax(0, 1fr); }
    .tile-foot { grid-template-columns: minmax(0, 1fr); }
    .action { justify-content: flex-start; }
    .action .consequence { text-align: left; }
  }
</style>
