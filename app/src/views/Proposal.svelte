<script lang="ts">
  import Shape from '../lib/Shape.svelte'
  import MachineMark from '../lib/MachineMark.svelte'
  import WorkGlyph from '../lib/WorkGlyph.svelte'
  import Diff from '../lib/Diff.svelte'
  import Markdown from '../lib/Markdown.svelte'
  import { bodyLines, isMarkdown, stripFront } from '../lib/markdown'
  import { fileAct, proposalAct, isNewProblem, hostOnly, stewardCannotDecline } from '../lib/proposals'
  import { member, self, host, type House, type Proposal } from '../api/view'
  import { write } from '../api/backend'
  import Notes from '../lib/Notes.svelte'
  import Write from '../lib/Write.svelte'
  import type { Note } from '../api/types'
  import { fit } from '../lib/fit'
  import { t, when, i18n } from '../i18n/index.svelte'

  // id 为空时是"提案列表"：宽屏显示列表 + 第一份提案，窄屏只显示列表（两级页面）。
  let { house, id, refresh }: { house: House; id?: string; refresh?: () => void } = $props()
  // 地址里点名的提案找不到（已经处理过，或者地址错了）时，不能悄悄换成另一份：老板会以为在看这一份
  let p = $derived(id === undefined ? house.proposals[0] : house.proposals.find((x) => x.id === id))
  let author = $derived(member(house, p?.by ?? ''))
  // 茶客提出的新问题也是提案：它的编号是在茶客自己的副本里取的，正本上可能还没有这个问题，
  // 也可能已经被别人用掉（采纳时引擎会改号，Renumbered:）。所以这种提案不链接到同号的问题
  const newProblem = isNewProblem
  let isNew = $derived(!!p && newProblem(p))
  let problem = $derived(p && !isNew ? house.problems.find((x) => x.id === p.problem) : undefined)
  let me = $derived(self(house))
  let owner = $derived(host(house))
  let kind = $derived(p ? proposalAct(p) : null)
  const actLabel = (q: Proposal) => { const k = proposalAct(q); return t(`prop.act.${k.act}`) + (k.more ? t('prop.actMore', { n: k.more }) : '') }
  // 伙计当管理员（2026-11-07）：采纳、不采纳；改座位表或茶馆设置的只归老板
  let decider = $derived(me.role === 'host' || (me.role === 'steward' && !!p && !hostOnly(p)))
  let canDecline = $derived(me.role === 'host' || (!!p && !stewardCannotDecline(p, p.problem === 'board' || house.problems.some((x) => x.id === p.problem))))
  let outcome = $state<'pending' | 'merged' | 'rejected'>('pending')
  // 撤回自己的提案：先点"撤回"，再点"确定撤回"
  let withdrawing = $state(false)
  async function withdraw() {
    if (!p) return
    if (!(await write('withdraw', { house: house.id, proposal: p.id }))) return
    withdrawing = false
    refresh?.()
    location.hash = `#/${house.id}/proposals`
  }
  // 采纳 / 不采纳走引擎。决定不能撤回（CLIENT-API 第 1.5 节）
  async function decide(adopt: boolean) {
    if (!p) return
    const done = adopt ? await write('adopt', { house: house.id, proposal: p.id }) : await write('decline', { house: house.id, proposal: p.id })
    if (done) outcome = adopt ? 'merged' : 'rejected'
  }

  // 提案下的留言（0022）：谁都能写，写下不改；东家、伙计能删。决定之后照样能留言
  async function note(text: string) {
    if (!p || !(await write('note', { house: house.id, proposal: p.id, text }))) return false
    refresh?.()
  }
  async function removeNote(n: Note) {
    if (await write('removeNote', { house: house.id, id: n.id })) refresh?.()
  }

  type File = Proposal['files'][number]
  // Markdown 文件能看成品。新文件全是新增，差异没有意义，默认看成品；改动过的默认看改了什么
  const canRead = (f: File) => isMarkdown(f.path) && f.change !== 'delete' && !!f.lines?.length
  let views = $state<Record<string, 'read' | 'diff'>>({})
  const showing = (f: File) => views[f.path] ?? (canRead(f) && f.change === 'add' ? 'read' : 'diff')
  const after = (f: File) => (f.lines ?? []).filter((l) => l.kind !== 'del').map((l) => l.text).join('\n')
  const shown = (f: File) => (isMarkdown(f.path) ? bodyLines(f.lines ?? []) : f.lines ?? [])
  const tally = (f: File) => ({
    add: shown(f).filter((l) => l.kind === 'add').length,
    del: shown(f).filter((l) => l.kind === 'del').length,
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
            <span class="q-no num">{#if !newProblem(q)}{q.problem} · {/if}<span class="q-act">{actLabel(q)}</span>{#if q.by === me.id}<span class="mine">{t('prop.yours')}</span>{/if}</span>
            <span class="q-t hyphenate">{q.title}</span>
            <span class="q-by"><Shape role={m.role} size={9} />{m.name}<MachineMark {m} /> · {when(q.when)}</span>
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
          <span class="q-t">{h.title}</span>
          <span class="q-by">{member(house, h.by).name}<MachineMark m={member(house, h.by)} /> · {t(h.result === 'adopted' ? 'prop.merged' : 'prop.rejected')} · {when(h.when)}</span>
        </li>
      {/each}
    </ul>
  </aside>

  {#if p}
    <section class="review" class:merged={outcome === 'merged'} class:rejected={outcome === 'rejected'}>
      <a class="back label" href="#/{house.id}/proposals">← {t('prop.back')}</a>
      <div class="crumb label">
        <span class="act">{kind ? actLabel(p) : ''}</span>
        {#if isNew}{t('prop.newProblem')}
        {:else if problem}{t('prop.forProblem')} <a href="#/{house.id}/problem/{problem.id}"><span class="num">{problem.id}</span> {problem.title}</a>
        {:else}{t('prop.forProblem')} <span class="num">{p.problem}</span>{/if}
      </div>

      <h2>{p.title}</h2>

      <div class="who">
        <span class="person"><Shape role={author.role} size={14} /><b>{author.name}</b><MachineMark m={author} />{t(`role.${author.role}`)}</span>
        <span>{when(p.when)}</span>
        <span>{t('prop.changes', { n: p.files.length })}</span>
      </div>

      {#if p.reason}
        <blockquote>{p.reason}</blockquote>
      {/if}

      {#each p.files as f}
        {@const tl = tally(f)}
        <div class="file">
          <div class="file-head">
            <WorkGlyph kind={f.kind} size={24} />
            <span class="kind label">{t(`prop.file.${fileAct(f)}`)} · {t(`kind.${f.kind}`)}</span>
            <span class="ft">{f.title}</span>
            <span class="tally">{outcome === 'merged' ? t('prop.mergedText') : t('prop.tally', tl)}</span>
          </div>
          {#if outcome !== 'merged' && canRead(f)}
            <div class="seg view" role="group">
              <button class:on={showing(f) === 'read'} aria-pressed={showing(f) === 'read'} onclick={() => (views[f.path] = 'read')}>{t('prop.viewRead')}</button>
              <button class:on={showing(f) === 'diff'} aria-pressed={showing(f) === 'diff'} onclick={() => (views[f.path] = 'diff')}>{t('prop.viewDiff')}</button>
            </div>
          {/if}
          {#if outcome === 'merged' || showing(f) === 'read'}
            <!-- 采纳后、或者"看成品"：直接显示合并后的正文，没有标记和行号 -->
            <div class="doc" class:muted={outcome === 'rejected'}>
              {#if canRead(f)}
                <Markdown text={stripFront(after(f))} house={house.id} />
              {:else}
                {#each (f.lines ?? []).filter((l) => l.kind !== 'del') as l}
                  <p class:gap={!l.text}>{l.text}</p>
                {/each}
              {/if}
            </div>
          {:else}
            <Diff lines={shown(f)} muted={outcome === 'rejected'} />
          {/if}
        </div>
      {/each}

      <div class="actions" data-write>
        {#if decider}
          {#if outcome === 'pending'}
            <p class="consequence">{t('prop.adoptHint')}</p>
            <div class="buttons">
              <div class="opt">
                <button class="secondary reject" disabled={!canDecline} onclick={() => decide(false)}>{t('prop.reject')}</button>
                <small>{canDecline ? t('prop.rejectNote', { name: author.name }) : t('prop.declineHostOnly', { name: owner.name })}</small>
              </div>
              <button class="primary adopt" onclick={() => decide(true)}><i><b></b></i><span>{t('prop.adopt')}</span></button>
            </div>
          {:else}
            <div class="verdict">
              <i></i>
              <div>
                <b>{outcome === 'merged' ? t('prop.verdictMerged') : t('prop.verdictRejected')}</b>
                <span>{outcome === 'merged'
                  ? t('prop.verdictMergedSub', { when: t('time.justNow') })
                  : t('prop.verdictRejectedSub', { name: author.name, when: t('time.justNow') })}</span>
              </div>
            </div>
          {/if}
        {:else}
          <p class="consequence wait">
            {me.role === 'steward' ? t('prop.hostOnly', { name: owner.name }) : p.by === me.id ? t('prop.ownWaiting') : t('prop.waiting')}
          </p>
          <div class="buttons">
            {#if me.role !== 'steward'}
              {#if p.by === me.id}
                {#if withdrawing}
                  <button class="secondary sure" data-write onclick={withdraw}>{t('prop.withdrawSure')}</button>
                  <button class="secondary" onclick={() => (withdrawing = false)}>{t('prop.withdrawKeep')}</button>
                {:else}
                  <button class="secondary" data-write onclick={() => (withdrawing = true)}>{t('prop.withdraw')}</button>
                {/if}
              {/if}
              <a class="primary" href="#/{house.id}/propose"><i>+</i><span>{t('act.newProposal')}</span></a>
            {/if}
          </div>
        {/if}
      </div>

      <section class="talk">
        <div class="talk-head">
          <h2>{t('talk.title')}</h2>
          <p class="consequence">{t('prop.talkLede')}</p>
        </div>
        {#if p.notes?.length}
          <Notes {house} notes={p.notes} ondelete={me.role !== 'guest' ? removeNote : undefined} />
        {:else}
          <p class="quiet none">{t('talk.none')}</p>
        {/if}
        <div class="talk-write" data-write><Write placeholder={t('prop.talkPlaceholder')} hint={me.role === 'guest' ? t('write.hintGuest') : ''} onsubmit={note} /></div>
      </section>
    </section>
  {:else if id !== undefined}
    <section class="review gone">
      <a class="back label" href="#/{house.id}/proposals">← {t('prop.back')}</a>
      <p class="consequence">{t('prop.gone')}</p>
    </section>
  {/if}
</div>

<style>
  .sure { background: var(--ink); color: var(--paper); }
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
  .doc.muted { color: var(--ink-3); }
  .act { display: inline-block; margin-right: 10px; padding: 1px 7px; background: var(--ink); color: var(--paper); font-weight: 700; }
  .q-act { font-family: var(--sans); letter-spacing: 0; }
  .reject:disabled { color: var(--ink-3); border-color: var(--ink-3); cursor: default; }
  .view { display: flex; gap: 0; margin-top: 10px; }
  .view button { padding: 4px 12px; background: transparent; border: 1px solid var(--ink); color: var(--ink); font: inherit; font-size: 13px; cursor: pointer; }
  .view button + button { border-left: 0; }
  .view button.on { background: var(--ink); color: var(--paper); }
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
  .talk { margin-top: 56px; max-width: 48em; }
  .talk-head { display: flex; flex-wrap: wrap; align-items: baseline; gap: 6px 24px; padding-bottom: 14px; border-bottom: 2px solid var(--ink); margin-bottom: 12px; }
  .talk-head h2 { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(22px, 2.2vw, 30px); letter-spacing: var(--track); }
  .talk .none { padding: 12px 0; color: var(--ink-3); font-size: 14px; }
  .talk-write { margin-top: 20px; }
</style>
