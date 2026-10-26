<script lang="ts">
  import Shape from '../lib/Shape.svelte'
  import AiMark from '../lib/AiMark.svelte'
  import StatusMark from '../lib/StatusMark.svelte'
  import WorkGlyph from '../lib/WorkGlyph.svelte'
  import ParkingMap from '../lib/ParkingMap.svelte'
  import Notes from '../lib/Notes.svelte'
  import Write from '../lib/Write.svelte'
  import AccessMark from '../lib/AccessMark.svelte'
  import Editor from '../lib/Editor.svelte'
  import { attachImage, imageUrl } from '../lib/images'
  import { member, self, pad, eventText, access, fileName, type House, type Note, type Problem } from '../api/view'
  import { api, write as send, blobUrl, stageFile } from '../api/backend'
  import { t, when, date } from '../i18n/index.svelte'

  let { house, id, refresh }: { house: House; id: string; refresh?: () => void } = $props()
  let p = $state<Problem | null>(null)
  let image = $state<string | null>(null)   // 首张成果是图片时，本地缓存里的地址
  async function load() {
    const fresh = await api('problem', { house: house.id, problem: id })
    const first = fresh.works[0]
    image = first?.kind === 'image' ? await api('blob', { house: house.id, id: first.id }).catch(() => null) : null
    p = fresh
  }
  $effect(() => { load() })

  let author = $derived(p && member(house, p.by))
  // 成果默认按文件列出（茶馆里真实的文件）；卡片是同一批文件的另一种看法
  let view = $state<'files' | 'cards'>('files')
  let me = $derived(self(house))
  let feature = $derived(p?.works[0])
  let rest = $derived(p?.works.slice(1) ?? [])
  // 经过：引擎按时间顺序给出，这里最新的在上
  let events = $derived(p ? [...p.events].reverse() : [])

  let action = $derived(me.role === 'guest' ? { hint: t('act.proposeWorkHint') } : { hint: t('act.addWorkHint') })

  // 添加成果：写一份文档（编辑器），或从手机的文件选择器加入图片、附件。茶客的会自动变成提案。
  let composing = $state(false)
  let newTitle = $state('')
  let newText = $state('')
  let added = $state<'written' | 'proposed' | null>(null)
  async function addDoc() {
    if (!newTitle.trim()) return
    const r = await send('addWork', { house: house.id, problem: id, kind: 'doc', title: newTitle.trim(), text: newText })
    if (!r) return
    added = r.becameProposal ? 'proposed' : 'written'
    composing = false; newTitle = ''; newText = ''
    await load()
  }
  let picker = $state<HTMLInputElement>()
  async function attach(e: Event) {
    const input = e.currentTarget as HTMLInputElement
    const f = input.files?.[0]
    input.value = ''
    if (!f) return
    const kind = f.type.startsWith('image/') ? 'image' : 'file'
    const r = await send('addWork', { house: house.id, problem: id, kind, title: f.name.replace(/\.[^.]+$/, ''), fromPath: await stageFile(f) })
    if (!r) return
    added = r.becameProposal ? 'proposed' : 'written'
    await load()
  }

  // 问题下的讨论：留言直接写进茶馆（茶客也一样）
  async function write(text: string) {
    if (!(await send('note', { house: house.id, problem: id, text }))) return false
    await load()
  }
  // 删除留言（老板、伙计）：removeNote，然后重新读取
  async function remove(n: Note) {
    if (await send('removeNote', { house: house.id, id: n.id })) await load()
  }

  // 状态和负责人（老板、伙计）：setStatus，点一下就写进茶馆
  let canManage = $derived(me.role !== 'guest')
  let owner = $derived(p?.owner ? member(house, p.owner) : null)
  async function setStatus(status: Problem['status'], ownerId = p?.owner ?? '') {
    if (!p || (status === p.status && ownerId === (p.owner ?? ''))) return
    if (await send('setStatus', { house: house.id, problem: id, status, owner: ownerId })) { await load(); refresh?.() }
  }

  // 修改问题本身：标题和说明（updateProblem）；状态、负责人不变。茶客的修改自动变成提案
  let editing = $state(false)
  let editTitle = $state('')
  let editText = $state('')
  let edited = $state<'written' | 'proposed' | null>(null)
  const bodyText = (body: Problem['body']) => body.map((b) => (b.heading ? `## ${b.heading}\n${b.text}` : b.text)).join('\n\n')
  function startEdit() {
    if (!p) return
    editTitle = p.title; editText = bodyText(p.body); edited = null; editing = true
  }
  let editChanged = $derived(!!p && !!editTitle.trim() && (editTitle.trim() !== p.title || editText.trim() !== bodyText(p.body).trim()))
  async function saveEdit() {
    if (!p || !editChanged) return
    const title = editTitle.trim() !== p.title ? editTitle.trim() : undefined
    const text = editText.trim() !== bodyText(p.body).trim() ? editText.trim() + '\n' : undefined
    const r = await send('updateProblem', { house: house.id, problem: id, title, text })
    if (!r) return
    edited = r.becameProposal ? 'proposed' : 'written'
    editing = false
    await load(); refresh?.()
  }
</script>

{#if p && author}

<div class="wrap">
  <a class="back label" href="#/{house.id}/problems">← {t('problem.back')}</a>

  <header>
    <div class="no num">{p.id}</div>
    <div class="head">
      <StatusMark status={p.status} />
      <h1>{p.title}</h1>
      <div class="meta">
        <span><Shape role={author.role} size={10} />{t('problem.raisedBy', { name: author.name })}<AiMark m={author} /> · {t(`role.${author.role}`)}</span>
        <span>{t('problem.since', { date: date(p.opened, 'short') })}</span>
        <span>{t('problem.works', { n: p.workCount })}</span>
        {#if owner}<span><Shape role={owner.role} size={10} />{t('problem.owner', { name: owner.name })}<AiMark m={owner} /></span>{/if}
      </div>
      {#if !editing}
        <button class="secondary small edit" data-write onclick={startEdit}>{t(me.role === 'guest' ? 'pedit.startGuest' : 'pedit.start')}</button>
      {/if}
    </div>
  </header>

  {#if edited}<p class="saved" role="status">{t(edited === 'written' ? 'edit.written' : 'edit.proposed')}</p>{/if}

  {#if editing}
    <form class="pedit" data-write onsubmit={(e) => { e.preventDefault(); saveEdit() }}>
      <label class="label" for="pedit-title">{t('newProblem.title')}</label>
      <input id="pedit-title" class="title-in" bind:value={editTitle} />
      <span class="label">{t('newProblem.body')}</span>
      <Editor value={editText} placeholder={t('newProblem.bodyPh')} onchange={(v) => (editText = v)} />
      <p class="consequence">{t(me.role === 'guest' ? 'pedit.hintGuest' : 'pedit.hint')}</p>
      <div class="save">
        <button type="submit" class="primary" disabled={!editChanged}><span>{t(me.role === 'guest' ? 'edit.saveProposal' : 'edit.save')}</span></button>
        <button type="button" class="secondary" onclick={() => (editing = false)}>{t('edit.cancel')}</button>
      </div>
    </form>
  {/if}

  <div class="cols">
    <section class="brief">
      {#if canManage}
        <div class="manage" data-write>
          <div class="label">{t('manage.status')}</div>
          <div class="states" role="group" aria-label={t('manage.status')}>
            {#each ['open', 'active', 'solved'] as const as s}
              <button class:on={p.status === s} aria-pressed={p.status === s} onclick={() => setStatus(s)}><StatusMark status={s} /></button>
            {/each}
          </div>
          <label class="label" for="owner">{t('manage.owner')}</label>
          <select id="owner" value={p.owner ?? ''} onchange={(e) => setStatus(p!.status, e.currentTarget.value)}>
            <option value="">{t('manage.nobody')}</option>
            {#each house.members as m}<option value={m.id}>{m.name}{m.kind === 'ai' ? ' · AI' : ''}</option>{/each}
          </select>
          <p class="consequence">{t('manage.hint')}</p>
        </div>
      {/if}
      <!-- 没有小标题的一段（第一个 "## " 之前）是摘要，不编号；有小标题的各节依次编号 -->
      {#each p.body as b, i}
        {#if b.heading}
          <div class="para">
            <div class="label">{pad(p.body.slice(0, i + 1).filter((x) => x.heading).length)} · {b.heading}</div>
            <p>{b.text}</p>
          </div>
        {:else if b.text}
          <p class="lead">{b.text}</p>
        {/if}
      {:else}
        <p class="quiet">{p.summary}</p>
      {/each}

      {#if events.length}
        <div class="label tl-head">{t('problem.history')}</div>
        <ol class="timeline">
          {#each events as e}
            {@const m = member(house, e.by)}
            <li>
              <span class="node"><Shape role={m.role} size={12} /></span>
              <div>
                <div class="ev"><b>{m.name}</b><AiMark {m} /> {eventText(e, true)}</div>
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
        <div class="action" data-write>
          <p class="consequence">{action.hint}</p>
          <button class="primary" aria-expanded={composing} onclick={() => { composing = !composing; added = null }}><i>+</i><span>{t('edit.newDoc')}</span></button>
          <button class="secondary" onclick={() => picker?.click()}>+ {t('edit.attach')}</button>
          <input class="picker" type="file" bind:this={picker} onchange={attach} />
        </div>
      </div>

      {#if added}<p class="saved" role="status">{t(added === 'written' ? 'edit.written' : 'edit.proposed')}</p>{/if}

      {#if composing}
        <div class="compose" data-write>
          <label class="label" for="new-title">{t('edit.title')}</label>
          <input id="new-title" class="title-in" bind:value={newTitle} placeholder={t('edit.titlePlaceholder')} />
          <Editor value="" placeholder={t('edit.bodyPlaceholder')} onchange={(v) => (newText = v)}
            onimage={(f) => attachImage(house.id, id, f)} resolveImage={(src) => imageUrl(house.id, src)} />
          <div class="save">
            <button class="primary" disabled={!newTitle.trim()} onclick={addDoc}><span>{t(me.role === 'guest' ? 'edit.saveProposal' : 'edit.save')}</span></button>
            <button class="secondary" onclick={() => (composing = false)}>{t('edit.cancel')}</button>
          </div>
        </div>
      {/if}

      {#if p.workCount === 0}
        <div class="empty">
          <svg width="48" height="48" viewBox="0 0 48 48" aria-hidden="true"><rect x="1.5" y="1.5" width="45" height="45" fill="none" stroke="var(--ink)" stroke-width="2" stroke-dasharray="6 5" /></svg>
          <p>{t('problem.empty')}</p>
        </div>
      {/if}

      {#if p.works.length}
        <div class="seg" role="group">
          <button class:on={view === 'files'} aria-pressed={view === 'files'} onclick={() => (view = 'files')}>{t('problem.viewFiles')}</button>
          <button class:on={view === 'cards'} aria-pressed={view === 'cards'} onclick={() => (view = 'cards')}>{t('problem.viewCards')}</button>
        </div>
      {/if}

      {#if view === 'files'}
      <ul class="files">
        {#each p.works as w}
          {@const wm = member(house, w.by)}
          <li>
            <a href="#/{house.id}/archive/{encodeURIComponent(w.id)}">
              <WorkGlyph kind={w.kind} size={20} />
              <span class="fname"><span class="real">{fileName(w.id)}</span><span class="human">{w.title}</span></span>
              <span class="fby"><Shape role={wm.role} size={9} />{wm.name}<AiMark m={wm} /> · {when(w.when)}</span>
              <AccessMark a={access(house, w.id)} />
            </a>
          </li>
        {/each}
      </ul>
      {:else}
      <div class="grid">
        {#if feature}
          {@const fm = member(house, feature.by)}
          <article class="tile feature">
            {#if image || feature.text}
              <div class="preview">
                {#if image === 'fake:parking'}
                  <ParkingMap />
                {:else if image}
                  <img src={blobUrl(house.id, feature.id, image)} alt={feature.title} />
                {:else}
                  <blockquote>{feature.text}</blockquote>
                {/if}
              </div>
            {/if}
            <div class="tile-foot">
              <div>
                <div class="kind label">{t(`kind.${feature.kind}`)} · {t('problem.latest')}</div>
                <div class="t">{feature.title}</div>
                {#if feature.note}<div class="note">{feature.note}</div>{/if}
              </div>
              <div class="by"><Shape role={fm.role} size={10} />{fm.name}<AiMark m={fm} /><span>{when(feature.when)}</span></div>
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
            <div class="t hyphenate">{w.title}</div>
            {#if w.note}<div class="note hyphenate">{w.note}</div>{/if}
            <div class="by"><Shape role={wm.role} size={10} /><b>{wm.name}</b><AiMark m={wm} /><span>{when(w.when)}</span></div>
          </article>
        {/each}
      </div>
      {/if}

      <section class="talk">
        <div class="talk-head">
          <h2>{t('talk.title')}</h2>
          <p class="consequence">{t('talk.lede')}</p>
        </div>
        {#if p.notes.length}
          <Notes {house} notes={p.notes} ondelete={me.role !== 'guest' ? remove : undefined} />
        {:else}
          <p class="quiet none">{t('talk.none')}</p>
        {/if}
        <div class="talk-write" data-write><Write placeholder={t('talk.placeholder')} hint={me.role === 'guest' ? t('write.hintGuest') : ''} onsubmit={write} /></div>
      </section>
    </section>
  </div>
</div>

{/if}

<style>
  .edit { margin-top: 16px; }
  .small { min-height: 40px; font-size: 13px; padding: 6px 16px; }
  .wrap > .saved { margin: 20px 0 0; }
  .pedit { display: grid; gap: 10px; max-width: 52rem; margin-top: 28px; padding: clamp(16px, 2.5vw, 24px); border: 2px solid var(--ink); }
  .pedit .save { display: flex; flex-wrap: wrap; gap: 10px; }
  .pedit .primary:disabled { background: transparent; color: var(--ink-3); border-color: var(--ink-3); cursor: default; }
  .manage { display: grid; gap: 10px; padding-bottom: 24px; margin-bottom: 28px; border-bottom: 1px solid var(--ink); }
  .states { display: grid; grid-template-columns: 1fr; border: 1.5px solid var(--ink); }
  .states button { display: flex; padding: 9px 12px; background: var(--paper); border: 0; border-top: 1px solid var(--ink); cursor: pointer; color: var(--ink); text-align: left; }
  .states button:first-child { border-top: 0; }
  .states button:hover { background: var(--paper-deep); }
  .states button.on { background: var(--ink); color: var(--paper); }
  .states button.on :global(i) { border-color: var(--paper); }
  .states button.on :global(.status.solved i), .states button.on :global(.status.active i::before) { background: var(--paper); }
  .states button.on :global(span) { color: var(--paper); }
  .states :global(button.on[aria-busy])::after { color: var(--paper); }
  select { font: inherit; font-size: 15px; padding: 9px 12px; border: 1.5px solid var(--ink); background: var(--paper); color: var(--ink); border-radius: 0; min-width: 0; }
  .lead { font-size: 17px; line-height: 1.8; margin-bottom: 22px; white-space: pre-line; }
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
  .para p { margin-top: 8px; font-size: 15px; line-height: 1.85; white-space: pre-line; }
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

  .picker { display: none; }
  .compose { display: grid; gap: 10px; margin-bottom: 28px; }
  .title-in { border: 1.5px solid var(--ink); background: var(--paper); padding: 10px 14px; font: inherit; font-size: 18px; font-weight: 700; color: var(--ink); }
  .save { display: flex; flex-wrap: wrap; gap: 10px; }
  .saved { margin-bottom: 16px; padding: 10px 14px; background: var(--ink); color: var(--paper); font-size: 14px; }
  .seg { display: flex; justify-content: flex-end; margin-bottom: 12px; }
  .seg button { padding: 6px 14px; border: 1.5px solid var(--ink); margin-left: -1.5px; font-size: 13px; font-weight: 500; }
  .seg button.on { background: var(--ink); color: var(--paper); }
  .files { list-style: none; border-top: 2px solid var(--ink); }
  .files a {
    display: flex; align-items: center; gap: 12px;
    padding: 12px 8px; border-bottom: 1px solid var(--rule);
    color: inherit; text-decoration: none;
  }
  .files a:hover { background: var(--paper-deep); }
  .files :global(svg) { flex: none; }
  .fname { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; overflow-wrap: anywhere; }
  .fname .real { font-weight: 600; font-size: 15px; }
  .fname .human { font-size: 13px; color: var(--ink-3); }
  .fby { display: inline-flex; align-items: center; gap: 6px; font-size: 12px; color: var(--ink-3); white-space: nowrap; }
  @media (max-width: 640px) { .fby { display: none; } }
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
  .preview img { display: block; max-width: 100%; }
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
