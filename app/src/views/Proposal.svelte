<script lang="ts">
  import Shape from '../lib/Shape.svelte'
  import WorkGlyph from '../lib/WorkGlyph.svelte'
  import Diff from '../lib/Diff.svelte'
  import { member, self, boss, type House, type Proposal } from '../data'
  import { fit } from '../lib/fit'
  import { t, L, when, i18n } from '../i18n/index.svelte'

  // id 为空时是"提案列表"：宽屏显示列表 + 第一份提案，窄屏只显示列表（两级页面）。
  let { house, id }: { house: House; id?: string } = $props()
  let p = $derived(house.proposals.find((x) => x.id === id) ?? house.proposals[0])
  let author = $derived(p && member(house, p.by))
  let problem = $derived(p && house.problems.find((x) => x.id === p.problem)!)
  let me = $derived(self(house))
  let owner = $derived(boss(house))
  let state = $state<'pending' | 'merged' | 'rejected'>('pending')

  const tally = (f: Proposal['files'][number]) => ({
    add: f.lines.filter((l) => l.kind === 'add').length,
    del: f.lines.filter((l) => l.kind === 'del').length,
  })
</script>

<div class="wrap" class:detail={id !== undefined}>
  <aside class="queue">
    <div class="label">04 · {t('nav.proposals')}</div>
    <h1 use:fit={i18n.lang}><span>{t('prop.title')}</span></h1>

    <div class="label sub">{t('prop.pending')} <em class="badge">{house.proposals.length}</em></div>
    <ul>
      {#each house.proposals as q}
        {@const m = member(house, q.by)}
        <li class:on={q.id === p?.id}>
          <a href="#/{house.id}/proposal/{q.id}">
            <span class="q-no num">{q.problem}{#if q.by === me.id}<span class="mine">{t('prop.yours')}</span>{/if}</span>
            <span class="q-t hyphenate">{L(q.title)}</span>
            <span class="q-by"><Shape role={m.role} size={9} />{L(m.name)} · {when(q.when)}</span>
          </a>
        </li>
      {:else}
        <li class="none">{t('prop.none')}</li>
      {/each}
    </ul>

    <div class="label sub">{t('prop.handled')}</div>
    <ul class="done">
      {#each house.decided as h}
        <li>
          <span class="q-t">{L(h.title)}</span>
          <span class="q-by">{L(member(house, h.by).name)} · {t(`prop.${h.result}`)} · {when(h.when)}</span>
        </li>
      {/each}
    </ul>
  </aside>

  {#if p}
    <section class="review" class:merged={state === 'merged'} class:rejected={state === 'rejected'}>
      <a class="back label" href="#/{house.id}/proposals">← {t('prop.back')}</a>
      <div class="crumb label">
        {t('prop.forProblem')} <a href="#/{house.id}/problem/{problem.id}"><span class="num">{problem.id}</span> {L(problem.title)}</a>
      </div>

      <h2>{L(p.title)}</h2>

      <div class="who">
        <span class="person"><Shape role={author.role} size={14} /><b>{L(author.name)}</b>{t(`role.${author.role}`)}</span>
        <span>{when(p.when)}</span>
        <span>{t('prop.changes', { n: p.files.length })}</span>
      </div>

      {#if p.reason}
        <blockquote>{L(p.reason)}</blockquote>
      {/if}

      {#each p.files as f}
        {@const tl = tally(f)}
        <div class="file">
          <div class="file-head">
            <WorkGlyph kind={f.kind} size={24} />
            <span class="kind label">{t(`kind.${f.kind}`)}</span>
            <span class="ft">{L(f.title)}</span>
            <span class="tally">{state === 'merged' ? t('prop.mergedText') : t('prop.tally', tl)}</span>
          </div>
          {#if state === 'merged'}
            <!-- 采纳后：直接显示合并后的正文，没有标记和行号 -->
            <div class="doc">
              {#each f.lines.filter((l) => l.kind !== 'del') as l}
                <p class:gap={!L(l.text)}>{L(l.text)}</p>
              {/each}
            </div>
          {:else}
            <Diff lines={f.lines} muted={state === 'rejected'} />
          {/if}
        </div>
      {/each}

      <div class="actions">
        {#if me.role === 'boss'}
          {#if state === 'pending'}
            <p class="consequence">{t('prop.adoptHint')}</p>
            <div class="buttons">
              <div class="opt">
                <button class="secondary reject" onclick={() => (state = 'rejected')}>{t('prop.reject')}</button>
                <small>{t('prop.rejectNote', { name: L(author.name) })}</small>
              </div>
              <button class="primary adopt" onclick={() => (state = 'merged')}><i><b></b></i><span>{t('prop.adopt')}</span></button>
            </div>
          {:else}
            <div class="verdict">
              <i></i>
              <div>
                <b>{state === 'merged' ? t('prop.verdictMerged') : t('prop.verdictRejected')}</b>
                <span>{state === 'merged'
                  ? t('prop.verdictMergedSub', { when: t('time.justNow') })
                  : t('prop.verdictRejectedSub', { name: L(author.name), when: t('time.justNow') })}</span>
              </div>
              <button class="undo" onclick={() => (state = 'pending')}>{t('prop.undo')}</button>
            </div>
          {/if}
        {:else}
          <p class="consequence wait">
            {p.by === me.id ? t('prop.ownWaiting', { name: L(owner.name) }) : t('prop.waiting', { name: L(owner.name) })}
          </p>
          <div class="buttons">
            {#if me.role === 'heir'}
              <div class="opt"><small>{t('prop.editDirectHint')}</small></div>
              <a class="primary" href="#/{house.id}/problem/{problem.id}"><i>→</i><span>{t('prop.editDirect')}</span></a>
            {:else}
              {#if p.by === me.id}<button class="secondary">{t('prop.withdraw')}</button>{/if}
              <a class="primary" href="#/{house.id}/propose"><i>+</i><span>{t('act.newProposal')}</span></a>
            {/if}
          </div>
        {/if}
      </div>
    </section>
  {/if}
</div>

<style>
  .wrap { display: grid; grid-template-columns: clamp(240px, 21vw, 300px) minmax(0, 1fr); min-height: 100vh; }

  .queue { border-right: 1px solid var(--ink); padding: var(--pad-top) 0 40px; min-width: 0; --in: clamp(20px, 2.2vw, 32px); }
  .queue > .label { padding: 0 var(--in); }
  h1 {
    font-family: var(--display); font-weight: var(--heavy); font-size: clamp(40px, 3.9vw, 56px);
    line-height: 1; letter-spacing: var(--track); margin: 22px var(--in) 44px; white-space: nowrap; hyphens: none;
  }
  h1 span { display: block; width: max-content; }
  .sub { padding: 0 var(--in) 10px !important; border-bottom: 1px solid var(--ink); margin-top: 8px; display: flex; align-items: center; gap: 8px; }
  .sub .badge { font-size: 11px; letter-spacing: 0; }
  ul { list-style: none; margin-bottom: 40px; }
  li { border-bottom: 1px solid var(--rule); }
  li a, .done li, li.none { display: flex; flex-direction: column; gap: 4px; padding: 16px var(--in); position: relative; }
  li.none { color: var(--ink-2); font-size: 14px; }
  li a:hover { background: var(--paper-deep); }
  li.on a { background: var(--ink); color: var(--paper); }
  .q-no { font-size: 12px; color: var(--ink-3); display: flex; gap: 10px; align-items: baseline; }
  .mine { font-family: var(--sans); font-size: 11px; font-weight: 500; border: 1px solid currentColor; padding: 0 5px; }
  .on .q-no { color: var(--paper); }
  .q-t { font-family: var(--display); font-size: 15px; font-weight: 700; line-height: 1.45; letter-spacing: var(--track); }
  .q-by { font-size: 12px; color: var(--ink-2); display: flex; flex-wrap: wrap; align-items: center; gap: 6px; }
  .on .q-by { color: var(--paper); }
  .done .q-t { font-weight: 500; color: var(--ink-2); }
  .done .q-by { color: var(--ink-3); }

  .review { padding: var(--pad-top) var(--pad-x) 0 clamp(28px, 3.9vw, 56px); display: flex; flex-direction: column; min-width: 0; }
  .back { display: none; margin-bottom: 18px; }
  .crumb a { color: var(--ink); margin-left: 8px; letter-spacing: 0.05em; border-bottom: 1px solid var(--ink); }
  h2 {
    font-family: var(--display); font-weight: var(--heavy); font-size: clamp(30px, 3vw, 44px);
    line-height: 1.2; margin: 18px 0; max-width: 20em; letter-spacing: var(--track);
  }
  .who { display: flex; flex-wrap: wrap; align-items: center; gap: 8px 0; font-size: 13px; color: var(--ink-2); }
  .who > span { padding: 0 12px; border-left: 1px solid var(--ink-3); white-space: nowrap; }
  .who > span:first-child { padding-left: 0; border-left: 0; }
  .person { display: inline-flex; align-items: center; gap: 10px; }
  .person b { color: var(--ink); font-size: 14px; }

  blockquote {
    margin: 28px 0 8px;
    padding: 2px 0 2px 24px;
    border-left: 4px solid var(--ink);
    font-size: 15px;
    line-height: 1.85;
    max-width: 44em;
  }

  .file { margin-top: 32px; border-top: 2px solid var(--ink); }
  .file-head { display: flex; flex-wrap: wrap; align-items: center; gap: 8px 14px; padding: 14px 0; border-bottom: 1px solid var(--ink); }
  .ft { font-family: var(--display); font-size: 17px; font-weight: 700; letter-spacing: var(--track); }
  .tally { margin-left: auto; font-size: 13px; color: var(--ink-2); }

  .doc { padding: 14px 0 6px; animation: settle 520ms var(--ease) both; }
  .doc p { font-size: 15px; line-height: 1.75; padding: 3px 0; white-space: pre-wrap; }
  .doc p.gap { height: 0.9em; padding: 0; }
  @keyframes settle { from { opacity: 0; transform: translateY(4px); } to { opacity: 1; transform: none; } }

  .actions {
    margin-top: 48px;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 20px 32px;
    padding: 22px 0 40px;
    border-top: 2px solid var(--ink);
  }
  .actions > .consequence { flex: 1 1 18em; }
  .wait { font-size: 14px; color: var(--ink); }
  .buttons { display: flex; flex-wrap: wrap; align-items: flex-start; gap: 12px; }
  .opt { display: flex; flex-direction: column; gap: 6px; max-width: 18em; }
  .opt small { font-size: 12px; color: var(--ink-2); line-height: 1.45; }
  .adopt b { display: block; width: 14px; height: 14px; background: var(--paper); transition: transform 300ms var(--ease); }
  .adopt:hover b { background: var(--ink); transform: rotate(45deg); }

  .verdict { display: flex; flex-wrap: wrap; align-items: center; gap: 16px 20px; width: 100%; animation: land 520ms var(--ease) both; }
  .verdict i { width: 56px; height: 56px; background: var(--ink); flex: none; }
  .rejected .verdict i { background: none; border: 2px solid var(--ink); }
  .verdict b { display: block; font-family: var(--display); font-size: 22px; font-weight: var(--heavy); letter-spacing: var(--track-wide); }
  .verdict span { font-size: 13px; color: var(--ink-2); }
  .undo { margin-left: auto; font-size: 13px; font-weight: 500; border-bottom: 1px solid var(--ink); }
  @keyframes land { from { opacity: 0; transform: translateX(-12px); } to { opacity: 1; transform: none; } }

  /* 窄窗口：列表 → 详情 两级页面 */
  @media (max-width: 1023.98px) {
    .wrap { grid-template-columns: minmax(0, 1fr); min-height: 0; }
    .queue { border-right: 0; }
    .wrap.detail .queue { display: none; }
    .wrap:not(.detail) .review { display: none; }
    .back { display: inline-block; }
    .review { padding-left: var(--pad-x); }
  }
</style>
