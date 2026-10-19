<script lang="ts">
  import Shape from '../lib/Shape.svelte'
  import StatusMark from '../lib/StatusMark.svelte'
  import ParkingMap from '../lib/ParkingMap.svelte'
  import FileGlyph from '../lib/FileGlyph.svelte'
  import Diff from '../lib/Diff.svelte'
  import { member, self, pad, type House } from '../data'
  import { archive, type AFile, type HistEntry } from '../data/archive'
  import { t, L, when, date, i18n } from '../i18n/index.svelte'

  // 档案：茶馆里所有文件原原本本放在这里。左边文件树（按目录 / 按问题），右边预览和"来历"。
  // 目录用人话命名，不显示仓库里的原始路径。窄窗口是"文件树 → 文件"两级页面。
  let { house, id }: { house: House; id?: string } = $props()
  let me = $derived(self(house))
  let files = $derived(archive(house))
  let file = $derived(files.find((f) => f.id === id) ?? files[0])

  type Node = { key: string; label: string; file?: AFile; children?: Node[] }

  let mode = $state<'dir' | 'problem'>('dir')
  let closed = $state<Record<string, boolean>>({})
  let openDiff = $state<number | null>(null)
  let exporting = $state(false)

  const problemLabel = (pid: string) => {
    const p = house.problems.find((x) => x.id === pid)
    return p ? `${p.id} · ${L(p.title)}` : pid
  }
  function fileLabel(f: AFile, inProblem = false): string {
    switch (f.kind) {
      case 'problem': return inProblem ? t('archive.problemFile') : problemLabel(f.problem)
      case 'notice': return L(f.notice!.title)
      case 'note': {
        const who = L(member(house, f.note!.by).name)
        const head = inProblem ? `${f.problem === 'board' ? t('archive.reply') : t('archive.note')} · ` : ''
        return `${head}${who} · ${date(f.note!.when, 'short')}`
      }
      case 'members': return t('archive.members')
      default: return L(f.work!.title)
    }
  }

  let tree = $derived.by((): Node[] => {
    void i18n.lang
    const of = (pred: (f: AFile) => boolean, inProblem = false) =>
      files.filter(pred).map((f) => ({ key: f.id, label: fileLabel(f, inProblem), file: f }))
    const isWork = (f: AFile) => !!f.work
    const meta: Node = { key: 'meta', label: t('archive.meta'), children: of((f) => f.kind === 'members') }
    if (mode === 'dir') {
      const withWorks = house.problems.filter((p) => p.works.length)
      const withNotes = house.problems.filter((p) => p.notes?.length)
      return [
        { key: 'problems', label: t('archive.problems'), children: of((f) => f.kind === 'problem') },
        {
          key: 'works', label: t('archive.works'),
          children: withWorks.map((p) => ({ key: `works-${p.id}`, label: problemLabel(p.id), children: of((f) => isWork(f) && f.problem === p.id) })),
        },
        { key: 'board', label: t('archive.board'), children: of((f) => f.kind === 'notice') },
        {
          key: 'talk', label: t('archive.talk'),
          children: [
            ...withNotes.map((p) => ({ key: `talk-${p.id}`, label: problemLabel(p.id), children: of((f) => f.kind === 'note' && f.problem === p.id) })),
            ...house.board.filter((n) => n.replies.length).map((n) => ({
              key: `talk-b-${n.id}`, label: t('archive.noticePrefix', { title: L(n.title) }),
              children: of((f) => f.kind === 'note' && f.notice?.id === n.id),
            })),
          ],
        },
        meta,
      ]
    }
    return [
      ...house.problems.map((p) => ({
        key: `p-${p.id}`, label: problemLabel(p.id),
        children: of((f) => f.problem === p.id, true),
      })),
      {
        key: 'pb', label: t('archive.board'),
        children: house.board.flatMap((n) => [
          ...of((f) => f.kind === 'notice' && f.notice!.id === n.id),
          ...of((f) => f.kind === 'note' && f.notice?.id === n.id, true),
        ]),
      },
      meta,
    ]
  })

  const ancestors = (nodes: Node[], target: string, path: string[] = []): string[] | null => {
    for (const n of nodes) {
      if (n.file?.id === target) return path
      if (n.children) {
        const r = ancestors(n.children, target, [...path, n.key])
        if (r) return r
      }
    }
    return null
  }
  let crumbs = $derived.by(() => {
    const keys = ancestors(tree, file.id) ?? []
    const labels: string[] = []
    let level = tree
    for (const k of keys) {
      const n = level.find((x) => x.key === k)
      if (!n) break
      labels.push(n.label)
      level = n.children ?? []
    }
    return labels
  })

  function size(n: number) {
    const f = new Intl.NumberFormat(i18n.lang === 'zh' ? 'zh-CN' : i18n.lang, { maximumFractionDigits: 1 })
    return n < 1024 ? `${n} B` : n < 1024 * 1024 ? `${f.format(n / 1024)} KB` : `${f.format(n / 1024 / 1024)} MB`
  }
  const cells = (row: string) => row.trim().split(/\s{2,}|　+/)

  function forWhat(e: HistEntry) {
    return e.problem === 'board' ? t('archive.forBoard') : e.problem === 'meta' ? t('archive.forMeta') : t('archive.for', { problem: problemLabel(e.problem) })
  }
  function actText(e: HistEntry) {
    if (e.act === 'open' || e.act === 'markActive' || e.act === 'markSolved') return t(`ev.${e.act}`)
    if (e.act === 'seat') return t('archive.act.seat', { name: L(member(house, e.who!).name) })
    return t(`archive.act.${e.act}`)
  }
  let history = $derived([...file.history].reverse())

  // 文本文件可以真的下载（M0 内容）；图片等没有真实文件
  function text(f: AFile): string | null {
    if (f.kind === 'problem') {
      const p = f.problemRef!
      return [`# ${L(p.title)}`, '', L(p.summary), ...p.body.flatMap((b) => ['', `## ${L(b.heading)}`, L(b.text)])].join('\n')
    }
    if (f.kind === 'notice') return `# ${L(f.notice!.title)}\n\n${L(f.notice!.text)}`
    if (f.kind === 'note') return L(f.note!.text)
    if (f.work?.lines) return f.work.lines.map(L).join('\n')
    return null
  }
  function download(f: AFile) {
    const body = text(f)
    if (body === null) return
    const a = document.createElement('a')
    a.href = URL.createObjectURL(new Blob([body], { type: 'text/plain;charset=utf-8' }))
    a.download = `${fileLabel(f).replace(/[\\/:*?"<>|]/g, '_')}.${f.kind === 'table' ? 'csv' : 'md'}`
    a.click()
    URL.revokeObjectURL(a.href)
  }
</script>

{#snippet branch(nodes: Node[], depth: number)}
  <ul class="branch" class:root={depth === 0}>
    {#each nodes as n (n.key)}
      <li>
        {#if n.file}
          <a class="row file" class:on={n.file.id === file.id} href="#/{house.id}/archive/{n.file.id}" style:--d={depth} data-file={n.file.id}>
            <FileGlyph kind={n.file.kind} />
            <span class="name">{n.label}</span>
          </a>
        {:else}
          <button class="row folder" style:--d={depth} aria-expanded={!closed[n.key]} onclick={() => (closed[n.key] = !closed[n.key])}>
            <FileGlyph kind="folder" open={!closed[n.key]} />
            <span class="name">{n.label}</span>
            <span class="count num">{n.children?.length ?? 0}</span>
          </button>
          {#if !closed[n.key] && n.children?.length}{@render branch(n.children, depth + 1)}{/if}
        {/if}
      </li>
    {/each}
  </ul>
{/snippet}

<div class="wrap" class:detail={id !== undefined}>
  <header class="top">
    <div>
      <div class="label">05 · {t('nav.archive')}</div>
      <h1>{t('archive.title')}</h1>
      <p class="lede">{t('archive.lede')} <span class="num">{t('archive.files', { n: files.length })}</span></p>
    </div>
    <div class="export">
      <button class="secondary" aria-expanded={exporting} onclick={() => (exporting = !exporting)}>
        <i class="box"></i>{t('archive.export')}
      </button>
      <p class="tagline">{t('archive.exportLine')}</p>
      {#if exporting}<p class="consequence note">{t('archive.exportNote')}</p>{/if}
    </div>
  </header>

  <div class="cols">
    <aside class="tree">
      <div class="seg" role="group">
        <button class:on={mode === 'dir'} aria-pressed={mode === 'dir'} onclick={() => (mode = 'dir')}>{t('archive.byDir')}</button>
        <button class:on={mode === 'problem'} aria-pressed={mode === 'problem'} onclick={() => (mode = 'problem')}>{t('archive.byProblem')}</button>
      </div>
      <nav aria-label={t('archive.title')}>{@render branch(tree, 0)}</nav>
    </aside>

    <section class="view">
      <a class="back label" href="#/{house.id}/archive">← {t('archive.back')}</a>
      {#if crumbs.length}<div class="crumbs label">{crumbs.join(' / ')}</div>{/if}

      <div class="fhead">
        <FileGlyph kind={file.kind} />
        <h2>{fileLabel(file)}</h2>
        <span class="meta">{t(`archive.kind.${file.kind}`)} · {size(file.size)}</span>
      </div>

      <div class="tools">
        {#if file.kind !== 'note' && file.kind !== 'members'}
          {#if me.role === 'guest'}
            <a class="primary small" href="#/{house.id}/propose/{file.problem === 'meta' ? '' : file.problem}"><i>+</i><span>{t('archive.proposeEdit')}</span></a>
            <span class="consequence">{t('archive.proposeHint')}</span>
          {:else if file.problem !== 'board' && file.problem !== 'meta'}
            <a class="secondary small" href="#/{house.id}/problem/{file.problem}">{t('archive.edit')}</a>
          {/if}
        {/if}
        <button class="secondary small" onclick={() => download(file)} disabled={text(file) === null} title={text(file) === null ? t('archive.noFile') : ''}>↓ {t('archive.download')}</button>
      </div>

      <article class="content">
        {#if file.kind === 'problem'}
          {@const p = file.problemRef!}
          <StatusMark status={p.status} />
          <h3>{L(p.title)}</h3>
          <p class="sum">{L(p.summary)}</p>
          {#each p.body as b}
            <h4>{L(b.heading)}</h4>
            <p>{L(b.text)}</p>
          {/each}
        {:else if file.kind === 'notice'}
          <h3>{L(file.notice!.title)}</h3>
          <p>{L(file.notice!.text)}</p>
        {:else if file.kind === 'note'}
          {@const m = member(house, file.note!.by)}
          <div class="who"><Shape role={m.role} size={10} /><b>{L(m.name)}</b><span>{when(file.note!.when)}</span></div>
          <p>{L(file.note!.text)}</p>
        {:else if file.kind === 'members'}
          <table>
            <thead><tr><th>{t('archive.colName')}</th><th>{t('archive.colRole')}</th><th>{t('archive.colJoined')}</th></tr></thead>
            <tbody>
              {#each house.members as m}
                <tr><td><span class="cell-who"><Shape role={m.role} size={10} />{L(m.name)}</span></td><td>{t(`role.${m.role}`)}</td><td>{date(m.joined, 'short')}</td></tr>
              {/each}
            </tbody>
          </table>
        {:else if file.kind === 'table'}
          {#if file.work?.lines}
            {@const rows = file.work.lines.map((l) => cells(L(l)))}
            <table>
              <thead><tr>{#each rows[0] as c}<th>{c}</th>{/each}</tr></thead>
              <tbody>{#each rows.slice(1) as r}<tr>{#each r as c}<td>{c}</td>{/each}</tr>{/each}</tbody>
            </table>
          {:else}
            <div class="binary"><FileGlyph kind="table" /><p>{t('archive.fileOnly')}</p></div>
          {/if}
        {:else if file.kind === 'image'}
          {#if file.work?.preview && 'chart' in file.work.preview}
            <ParkingMap />
          {:else}
            <div class="binary image"><svg viewBox="0 0 120 80" aria-hidden="true"><rect x="1" y="1" width="118" height="78" fill="none" stroke="var(--ink)" stroke-width="2" /><circle cx="88" cy="24" r="8" fill="var(--ink)" /><polygon points="10,72 44,26 78,72" fill="var(--ink)" /></svg><p>{t('archive.fileOnly')}</p></div>
          {/if}
        {:else}
          {@const w = file.work!}
          <h3>{L(w.title)}</h3>
          {#if w.lines}
            {#each w.lines as l}<p class="line">{L(l) || ' '}</p>{/each}
          {:else if w.preview && 'excerpt' in w.preview}
            <p class="pre">{L(w.preview.excerpt)}</p>
          {:else if w.note}
            <p>{L(w.note)}</p>
          {/if}
        {/if}
      </article>

      <section class="history">
        <div class="hhead">
          <h3>{t('archive.history')}</h3>
          <p class="consequence">{t('archive.histHint')}</p>
        </div>
        <ol>
          {#each history as e, i}
            {@const m = member(house, e.by)}
            <li>
              <button class="entry" class:open={openDiff === i} aria-expanded={openDiff === i} onclick={() => (openDiff = openDiff === i ? null : i)}>
                <span class="num idx">{pad(history.length - i)}</span>
                <span class="what">
                  <span class="by"><Shape role={m.role} size={10} /><b>{L(m.name)}</b> {actText(e)}</span>
                  <span class="for">{forWhat(e)}</span>
                </span>
                <span class="when">{when(e.when)}</span>
              </button>
              {#if openDiff === i}
                <div class="diff">
                  {#if e.diff?.length}<Diff lines={e.diff} />{:else}<p class="quiet">{t('archive.noDiff')}</p>{/if}
                </div>
              {/if}
            </li>
          {/each}
        </ol>
      </section>
    </section>
  </div>
</div>

<style>
  .wrap { padding: var(--pad-top) var(--pad-x) 80px; }
  .top { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: flex-end; gap: 24px 48px; padding-bottom: 24px; border-bottom: 2px solid var(--ink); }
  h1 { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(48px, 5.8vw, 84px); line-height: 1; margin-top: 22px; letter-spacing: var(--track); }
  .lede { margin-top: 16px; color: var(--ink-2); font-size: 16px; }
  .lede .num { margin-left: 8px; color: var(--ink-3); font-size: 14px; }
  .export { display: flex; flex-direction: column; align-items: flex-end; gap: 8px; max-width: 26em; text-align: right; }
  .export .secondary { gap: 12px; }
  .box { display: block; width: 14px; height: 14px; border: 2px solid currentColor; border-top-width: 5px; }
  .tagline { font-family: var(--display); font-weight: 700; font-size: 15px; letter-spacing: var(--track); }
  .note { text-align: right; }

  .cols { display: grid; grid-template-columns: clamp(250px, 24vw, 340px) minmax(0, 1fr); gap: clamp(28px, 3.5vw, 56px); margin-top: 28px; }

  .tree { min-width: 0; }
  .seg { display: flex; margin-bottom: 16px; }
  .seg button { flex: 1; padding: 8px 10px; border: 1.5px solid var(--ink); margin-right: -1.5px; font-size: 13px; font-weight: 500; text-align: center; }
  .seg button.on { background: var(--ink); color: var(--paper); }
  .branch { list-style: none; }
  .branch.root { border-top: 1px solid var(--ink); }
  .row {
    display: flex; align-items: center; gap: 10px; width: 100%;
    padding: 7px 8px 7px calc(8px + var(--d) * 18px);
    font-size: 14px; line-height: 1.35; text-align: left;
    border-bottom: 1px solid var(--rule);
    position: relative;
  }
  /* 细线：每一层缩进前画一条竖线 */
  .row::before { content: ''; position: absolute; left: calc(var(--d) * 18px - 4px); top: 0; bottom: 0; width: 1px; background: var(--rule); }
  .row:hover { background: var(--paper-deep); }
  .folder { font-weight: 700; }
  .folder .count { margin-left: auto; font-size: 11px; color: var(--ink-3); font-weight: 400; }
  .file.on { background: var(--ink); color: var(--paper); }
  .file.on :global(svg *) { fill: var(--paper); stroke: var(--paper); }
  .name { min-width: 0; overflow-wrap: anywhere; }

  .view { min-width: 0; }
  .back { display: none; margin-bottom: 14px; }
  .crumbs { margin-bottom: 10px; }
  .fhead { display: flex; flex-wrap: wrap; align-items: center; gap: 8px 14px; padding-bottom: 14px; border-bottom: 2px solid var(--ink); }
  .fhead :global(svg) { width: 16px; height: 16px; }
  h2 { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(22px, 2.2vw, 32px); line-height: 1.25; letter-spacing: var(--track); }
  .meta { margin-left: auto; font-size: 13px; color: var(--ink-2); white-space: nowrap; }
  .tools { display: flex; flex-wrap: wrap; align-items: center; gap: 10px 14px; padding: 14px 0; }
  .small { min-height: 40px; font-size: 13px; padding: 6px 16px; }
  .primary.small > i { width: 38px; font-size: 20px; }
  .primary.small > span { padding: 6px 16px; }
  .secondary.small:disabled { color: var(--ink-3); border-color: var(--ink-3); cursor: default; }
  .tools .consequence { font-size: 12px; }

  .content { padding: 20px 0 8px; max-width: 46em; }
  .content h3 { font-family: var(--display); font-weight: 700; font-size: 22px; line-height: 1.3; margin: 10px 0 12px; letter-spacing: var(--track); }
  .content h4 { margin: 18px 0 4px; font-size: 13px; font-weight: 700; letter-spacing: 0.06em; }
  .content p { font-size: 15px; line-height: 1.85; }
  .content .sum { color: var(--ink-2); }
  .content .line { line-height: 1.75; white-space: pre-wrap; }
  .content .pre { white-space: pre-line; border-left: 4px solid var(--ink); padding-left: 18px; }
  .who { display: flex; align-items: center; gap: 10px; font-size: 13px; margin-bottom: 8px; }
  .who span { color: var(--ink-3); font-family: var(--latin); }
  table { border-collapse: collapse; width: 100%; font-size: 14px; font-feature-settings: 'tnum' 1; }
  th, td { text-align: left; padding: 8px 12px 8px 0; border-bottom: 1px solid var(--rule); vertical-align: baseline; }
  th { font-weight: 700; border-bottom: 2px solid var(--ink); font-size: 13px; }
  .cell-who { display: inline-flex; align-items: center; gap: 8px; }
  .binary { display: flex; align-items: center; gap: 18px; padding: 24px; border: 2px dashed var(--ink); color: var(--ink-2); }
  .binary.image { flex-direction: column; align-items: flex-start; }
  .binary.image svg { width: min(100%, 320px); }

  .history { margin-top: 36px; }
  .hhead { display: flex; flex-wrap: wrap; align-items: baseline; gap: 6px 20px; padding-bottom: 12px; border-bottom: 2px solid var(--ink); }
  .hhead h3 { font-family: var(--display); font-weight: var(--heavy); font-size: 24px; letter-spacing: var(--track-wide); }
  .history ol { list-style: none; }
  .history li { border-bottom: 1px solid var(--rule); }
  .entry { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; gap: 16px; align-items: baseline; width: 100%; padding: 12px 0; text-align: left; }
  .entry:hover, .entry.open { background: var(--paper-deep); }
  .idx { font-size: 13px; color: var(--ink-3); padding-left: 6px; }
  .what { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .by { display: inline-flex; flex-wrap: wrap; align-items: center; gap: 4px 8px; font-size: 14px; }
  .for { font-size: 12px; color: var(--ink-2); }
  .when { font-family: var(--latin); font-size: 12px; color: var(--ink-3); white-space: nowrap; padding-right: 6px; }
  .diff { padding: 4px 0 14px; border-top: 1px solid var(--ink); }
  .quiet { color: var(--ink-2); font-size: 14px; padding: 10px 0; }

  @media (max-width: 1023.98px) {
    .cols { grid-template-columns: minmax(0, 1fr); }
    .wrap.detail .tree { display: none; }
    .wrap:not(.detail) .view { display: none; }
    .back { display: inline-block; }
    .export { align-items: flex-start; text-align: left; }
    .note { text-align: left; }
  }
</style>
