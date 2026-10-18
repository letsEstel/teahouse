<script lang="ts">
  import Shape from '../lib/Shape.svelte'
  import StatusMark from '../lib/StatusMark.svelte'
  import WorkGlyph from '../lib/WorkGlyph.svelte'
  import { diffLines } from '../lib/diff'
  import { self, boss, type House, type WorkKind } from '../data'
  import { t, L } from '../i18n/index.svelte'

  // 发起提案（茶客的核心动作）：面向哪个问题 → 改什么 → 为什么 → 交给老板。
  // arg：问题编号，或 "new" 表示提议一个新问题。
  let { house, arg }: { house: House; arg?: string } = $props()
  let me = $derived(self(house))
  let owner = $derived(boss(house))

  let target = $state<string | null>(null)     // 问题编号或 'new'
  let mode = $state<'modify' | 'add'>('modify')
  let workId = $state<string | null>(null)
  let body = $state('')                         // 正文（修改或新增）
  let newTitle = $state('')
  let newKind = $state<WorkKind>('doc')
  let reason = $state('')
  let sent = $state(false)

  $effect.pre(() => {
    target = arg === 'new' || house.problems.some((p) => p.id === arg) ? arg! : null
  })

  let problem = $derived(house.problems.find((p) => p.id === target))
  let works = $derived(problem?.works ?? [])
  let work = $derived(works.find((w) => w.id === workId))
  // 这个问题没有可改的成果时，只能添加
  let effMode = $derived(target === 'new' ? 'new' : works.length ? mode : 'add')
  let original = $derived(work?.lines?.map(L) ?? [])

  function pickProblem(id: string) {
    target = id
    workId = null
    body = ''
  }
  function pickWork(id: string) {
    workId = id
    const w = works.find((x) => x.id === id)
    body = w?.lines ? w.lines.map(L).join('\n') : ''
  }
  function setMode(m: 'modify' | 'add') {
    mode = m
    workId = null
    body = ''
  }

  let preview = $derived(effMode === 'modify' && work?.lines ? diffLines(original, body.split('\n')) : [])
  let changed = $derived(
    effMode === 'new' ? newTitle.trim() !== '' && body.trim() !== ''
    : effMode === 'add' ? newTitle.trim() !== '' && body.trim() !== ''
    : !!work && (work.lines ? preview.some((l) => l.kind !== 'same') : false)
  )
  let missing = $derived([
    !target && t('propose.needProblem'),
    target && !changed && t('propose.needChange'),
    !reason.trim() && t('propose.needReason'),
  ].filter(Boolean) as string[])

  const kinds: WorkKind[] = ['doc', 'plan', 'bill', 'image']
</script>

<div class="wrap">
  <header>
    <a class="back label" href="#/{house.id}/proposals">← {t('prop.back')}</a>
    <h1>{t('propose.title')}</h1>
    <p class="lede">{t('propose.lede', { name: L(owner.name) })}</p>
    {#if me.role !== 'guest'}<p class="consequence role-note">{t('propose.notGuest', { role: t(`role.${me.role}`) })}</p>{/if}
  </header>

  {#if sent}
    <section class="done">
      <i></i>
      <div>
        <h2>{t('propose.done', { name: L(owner.name) })}</h2>
        <p class="consequence">{t('propose.doneSub')}</p>
        <div class="done-actions">
          <a class="primary" href="#/{house.id}/proposals"><i>→</i><span>{t('propose.viewAll')}</span></a>
          <button class="secondary" onclick={() => { sent = false; reason = ''; body = ''; newTitle = ''; workId = null }}>{t('propose.another')}</button>
        </div>
      </div>
    </section>
  {:else}
    <form onsubmit={(e) => { e.preventDefault(); if (!missing.length) sent = true }}>
      <!-- 01 面向哪个问题 -->
      <section class="step">
        <div class="sn num">01</div>
        <div class="sc">
          <h2>{t('propose.step1')}</h2>
          <div class="choices" role="radiogroup" aria-label={t('propose.step1')}>
            {#each house.problems.filter((p) => p.status !== 'solved') as p}
              <button type="button" role="radio" aria-checked={target === p.id} class="choice" class:on={target === p.id} data-problem={p.id} onclick={() => pickProblem(p.id)}>
                <span class="num cid">{p.id}</span>
                <span class="ctitle">{L(p.title)}</span>
                <StatusMark status={p.status} label={false} />
              </button>
            {/each}
            <button type="button" role="radio" aria-checked={target === 'new'} class="choice new" class:on={target === 'new'} onclick={() => pickProblem('new')}>
              <span class="num cid">+</span>
              <span class="ctitle">{t('propose.newProblem')}</span>
            </button>
          </div>
        </div>
      </section>

      <!-- 02 改什么 -->
      <section class="step" class:off={!target}>
        <div class="sn num">02</div>
        <div class="sc">
          <h2>{t('propose.step2')}</h2>
          {#if target === 'new'}
            <label class="field">
              <span class="label">{t('propose.newProblemTitle')}</span>
              <input bind:value={newTitle} />
            </label>
            <label class="field">
              <span class="label">{t('propose.editor')}</span>
              <textarea name="body" rows="5" bind:value={body} placeholder={t('propose.newProblemBody')}></textarea>
            </label>
          {:else if target}
            {#if works.length}
              <div class="seg" role="group">
                <button type="button" class:on={effMode === 'modify'} onclick={() => setMode('modify')}>{t('propose.modify')}</button>
                <button type="button" class:on={effMode === 'add'} onclick={() => setMode('add')}>{t('propose.add')}</button>
              </div>
            {/if}

            {#if effMode === 'modify'}
              <div class="label sub">{t('propose.pickWork')}</div>
              <div class="works" role="radiogroup" aria-label={t('propose.pickWork')}>
                {#each works as w}
                  <button type="button" role="radio" aria-checked={workId === w.id} class="work" class:on={workId === w.id} data-work={w.id} onclick={() => pickWork(w.id)}>
                    <WorkGlyph kind={w.kind} size={22} />
                    <span>{L(w.title)}</span>
                  </button>
                {/each}
              </div>
              {#if work}
                {#if work.lines}
                  <div class="editor">
                    <label class="field">
                      <span class="label">{t('propose.editor')}</span>
                      <textarea name="body" rows={Math.max(6, original.length + 2)} bind:value={body}></textarea>
                    </label>
                    <div class="field">
                      <span class="label">{t('propose.preview')}</span>
                      <div class="preview">
                        {#if preview.some((l) => l.kind !== 'same')}
                          {#each preview as l}
                            <div class="pl {l.kind}">{l.text || ' '}</div>
                          {/each}
                        {:else}
                          <p class="quiet">{t('propose.noChange')}</p>
                        {/if}
                      </div>
                    </div>
                  </div>
                {:else}
                  <div class="upload">
                    <p>{t('propose.fileOnly')}</p>
                    <button type="button" class="secondary">{t('propose.upload')}</button>
                  </div>
                {/if}
              {/if}
            {:else}
              <label class="field">
                <span class="label">{t('propose.workTitle')}</span>
                <input bind:value={newTitle} />
              </label>
              <div class="field">
                <span class="label">{t('propose.kind')}</span>
                <div class="kinds" role="radiogroup" aria-label={t('propose.kind')}>
                  {#each kinds as k}
                    <button type="button" role="radio" aria-checked={newKind === k} class:on={newKind === k} onclick={() => (newKind = k)}>
                      <WorkGlyph kind={k} size={20} />{t(`kind.${k}`)}
                    </button>
                  {/each}
                </div>
              </div>
              <label class="field">
                <span class="label">{t('propose.editor')}</span>
                <textarea name="body" rows="6" bind:value={body}></textarea>
              </label>
            {/if}
          {/if}
        </div>
      </section>

      <!-- 03 为什么 -->
      <section class="step" class:off={!target}>
        <div class="sn num">03</div>
        <div class="sc">
          <h2>{t('propose.step3')}</h2>
          <textarea name="reason" class="reason" rows="2" bind:value={reason} placeholder={t('propose.reasonPh')} aria-label={t('propose.step3')}></textarea>
        </div>
      </section>

      <div class="submit">
        <div class="to">
          <Shape role="boss" size={12} />
          <div>
            <p class="consequence">{t('propose.submitHint', { name: L(owner.name) })}</p>
            {#if missing.length}<p class="missing">{t('propose.need', { what: missing.join(' · ') })}</p>{/if}
          </div>
        </div>
        <div class="buttons">
          <a class="secondary" href="#/{house.id}/proposals">{t('propose.cancel')}</a>
          <button type="submit" class="primary send" disabled={missing.length > 0}><i>→</i><span>{t('propose.submit')}</span></button>
        </div>
      </div>
    </form>
  {/if}
</div>

<style>
  .wrap { padding: var(--pad-top) var(--pad-x) 80px; max-width: 1240px; }
  header { padding-bottom: 28px; border-bottom: 2px solid var(--ink); }
  .back { display: inline-block; margin-bottom: 18px; }
  h1 { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(44px, 5vw, 76px); line-height: 1.05; letter-spacing: var(--track); }
  .lede { margin-top: 14px; font-size: 16px; color: var(--ink-2); max-width: 40em; }
  .role-note { margin-top: 10px; border-left: 4px solid var(--ink); padding-left: 14px; color: var(--ink); }

  .step { display: grid; grid-template-columns: clamp(64px, 8vw, 120px) minmax(0, 1fr); gap: clamp(16px, 2.5vw, 36px); padding: 32px 0; border-bottom: 1px solid var(--ink); }
  .step.off { opacity: 0.4; pointer-events: none; }
  .sn { font-size: clamp(44px, 5vw, 72px); font-weight: 300; line-height: 0.85; letter-spacing: -0.03em; }
  .sc { min-width: 0; display: flex; flex-direction: column; gap: 14px; }
  h2 { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(20px, 1.8vw, 26px); letter-spacing: var(--track); line-height: 1.3; }

  .choices { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, 300px), 1fr)); border-top: 1px solid var(--ink); border-left: 1px solid var(--ink); }
  .choice { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 14px; padding: 14px 16px; border-right: 1px solid var(--ink); border-bottom: 1px solid var(--ink); transition: background 200ms var(--ease); }
  .choice:hover { background: var(--paper-deep); }
  .choice.on { background: var(--ink); color: var(--paper); }
  .choice.on :global(.status i) { border-color: var(--paper); }
  .choice.on :global(.status i::before), .choice.on :global(.solved i) { background: var(--paper); }
  .cid { font-size: 20px; font-weight: 300; min-width: 2.4em; }
  .ctitle { font-weight: 700; line-height: 1.4; }
  .new .ctitle { font-weight: 500; }

  .seg { display: inline-flex; align-self: flex-start; }
  .seg button { padding: 8px 18px; border: 1.5px solid var(--ink); margin-right: -1.5px; font-weight: 500; font-size: 14px; }
  .seg button.on { background: var(--ink); color: var(--paper); }
  .sub { margin-top: 4px; }

  .works { display: flex; flex-wrap: wrap; gap: 8px; }
  .work { display: inline-flex; align-items: center; gap: 10px; padding: 8px 14px 8px 10px; border: 1.5px solid var(--ink); font-size: 14px; font-weight: 500; text-align: left; }
  .work:hover { background: var(--paper-deep); }
  .work.on { background: var(--ink); color: var(--paper); }
  .work.on :global(svg *) { stroke: var(--paper); }
  .work.on :global(svg [fill='var(--ink)']) { fill: var(--paper); }

  .field { display: flex; flex-direction: column; gap: 8px; min-width: 0; }
  input, textarea {
    font: inherit; color: var(--ink); background: transparent; width: 100%;
    border: 2px solid var(--ink); padding: 12px 14px; line-height: 1.7; outline: none;
  }
  input { font-family: var(--display); font-weight: 700; font-size: 17px; }
  textarea { font-size: 15px; resize: vertical; }
  input:focus, textarea:focus { outline: 3px solid var(--ink); outline-offset: 2px; }
  ::placeholder { color: var(--ink-3); opacity: 1; }

  .editor { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 20px; }
  .preview { border-top: 2px solid var(--ink); padding: 10px 0; }
  .pl { font-size: 15px; line-height: 1.75; padding: 2px 0 2px 16px; position: relative; white-space: pre-wrap; }
  .pl.add { font-weight: 500; }
  .pl.add::before { content: ''; position: absolute; left: 0; top: 5px; bottom: 5px; width: 5px; background: var(--ink); }
  .pl.del { color: var(--ink-3); text-decoration: line-through; text-decoration-thickness: 1.5px; }
  .quiet { color: var(--ink-2); font-size: 14px; }

  .upload { display: flex; flex-wrap: wrap; align-items: center; gap: 16px 24px; padding: 24px; border: 2px dashed var(--ink); }
  .upload p { flex: 1 1 18em; color: var(--ink-2); }

  .kinds { display: flex; flex-wrap: wrap; }
  .kinds button { display: inline-flex; align-items: center; gap: 8px; padding: 8px 14px 8px 10px; border: 1.5px solid var(--ink); margin-right: -1.5px; font-size: 14px; font-weight: 500; }
  .kinds button.on { background: var(--ink); color: var(--paper); }
  .kinds button.on :global(svg *) { stroke: var(--paper); }
  .kinds button.on :global(svg [fill='var(--ink)']) { fill: var(--paper); }

  .reason { font-size: 16px; }

  .submit { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 20px 32px; padding: 28px 0 0; }
  .to { display: flex; gap: 12px; align-items: flex-start; flex: 1 1 20em; }
  .to :global(svg) { margin-top: 4px; }
  .missing { margin-top: 6px; font-size: 13px; font-weight: 700; }
  .buttons { display: flex; flex-wrap: wrap; gap: 12px; }
  .send:disabled { background: transparent; color: var(--ink-3); border-color: var(--ink-3); cursor: default; }
  .send:disabled > i { border-right-color: var(--ink-3); }

  .done { display: flex; gap: 24px; align-items: flex-start; padding: 40px 0; animation: land 520ms var(--ease) both; }
  .done > i { width: 56px; height: 56px; background: var(--ink); flex: none; }
  .done h2 { font-size: clamp(24px, 2.4vw, 34px); }
  .done-actions { display: flex; flex-wrap: wrap; gap: 12px; margin-top: 24px; }
  @keyframes land { from { opacity: 0; transform: translateX(-12px); } to { opacity: 1; transform: none; } }

  @media (max-width: 1100px) { .editor { grid-template-columns: minmax(0, 1fr); } }
  @media (max-width: 640px) { .step { grid-template-columns: minmax(0, 1fr); } }
</style>
