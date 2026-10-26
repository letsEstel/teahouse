<script lang="ts">
  import Shape from '../lib/Shape.svelte'
  import MachineMark from '../lib/MachineMark.svelte'
  import StatusMark from '../lib/StatusMark.svelte'
  import ParkingMap from '../lib/ParkingMap.svelte'
  import FileGlyph from '../lib/FileGlyph.svelte'
  import Diff from '../lib/Diff.svelte'
  import AccessMark from '../lib/AccessMark.svelte'
  import Editor from '../lib/Editor.svelte'
  import MdImage from '../lib/MdImage.svelte'
  import { attachImage, imageUrl } from '../lib/images'
  import { member, self, pad, access, readReason, fileName, type House, type ArchiveNode, type HistoryEntry, type Status } from '../api/view'
  import { api, blobUrl, write as send, stageFile, exportTo, BackendError } from '../api/backend'
  import { t, when, date } from '../i18n/index.svelte'

  // 档案：茶馆里所有文件原原本本放在这里。左边文件树（按目录 / 按问题），右边预览和"来历"。
  // 树、文件原文和来历都来自引擎（archive / file / blob）；界面只负责排版，不显示仓库路径。
  // 窄窗口是"文件树 → 文件"两级页面。
  let { house, id }: { house: House; id?: string } = $props()
  let me = $derived(self(house))

  let mode = $state<'dirs' | 'problems'>('dirs')
  let root = $state<ArchiveNode | null>(null)
  $effect(() => {
    const view = mode
    api('archive', { house: house.id, view }).then((r) => { if (mode === view) root = r })
  })
  const leaves = (n: ArchiveNode): ArchiveNode[] => (n.children ? n.children.flatMap(leaves) : [n])
  let all = $derived(root ? leaves(root) : [])
  let selected = $derived(id ?? all[0]?.id)

  type FileData = { node: ArchiveNode; text?: string; history: HistoryEntry[] }
  let file = $state<FileData | null>(null)
  let image = $state<string | null>(null)
  let reloads = $state(0)
  $effect(() => {
    const target = selected
    void reloads
    editing = false
    if (!target) return
    api('file', { house: house.id, id: target }).then(async (f) => {
      image = f.node.kind === 'image' || f.node.kind === 'file' ? await api('blob', { house: house.id, id: target }).catch(() => null) : null
      if (selected === target) file = f
    })
  })

  let closed = $state<Record<string, boolean>>({})

  // ---- 编辑：成果文件（文档、方案、表格）在这里直接改；茶客的改动变成提案 ----
  let editing = $state(false)
  let draft = $state('')
  let saved = $state<'written' | 'proposed' | null>(null)
  let can = $derived(file ? access(house, file.node.id) : 'read')
  const isWork = (n: ArchiveNode) => n.id.startsWith('works/')
  const textual = (n: ArchiveNode) => n.kind === 'doc' || n.kind === 'plan' || n.kind === 'table'
  // 问题说明、公告也在这里直接改：只改正文（frontmatter 里的标题、状态、负责人不动），走 updateProblem / updateNotice
  const bodyOnly = (n: ArchiveNode) => n.kind === 'problem' || n.kind === 'notice'
  const stripFront = (s = '') => s.replace(/^---\n[\s\S]*?\n---\n?/, '').replace(/^\n+/, '')
  let editable = $derived(!!file && can !== 'read' && (isWork(file.node) ? textual(file.node) : bodyOnly(file.node)))
  function startEdit() {
    draft = file && bodyOnly(file.node) ? stripFront(file.text) : file?.text ?? ''
    saved = null
    editing = true
  }
  async function save() {
    if (!file) return
    const n = file.node
    const r = n.kind === 'problem' && n.problem ? await send('updateProblem', { house: house.id, problem: n.problem, text: draft })
      : n.kind === 'notice' ? await send('updateNotice', { house: house.id, notice: n.id, text: draft })
      : await send('updateWork', { house: house.id, id: n.id, text: draft })
    if (!r) return
    saved = r.becameProposal ? 'proposed' : 'written'
    reloads++
  }
  // 图片和附件：从手机的文件选择器换一个文件
  let picker = $state<HTMLInputElement>()
  async function replace(e: Event) {
    const f = (e.currentTarget as HTMLInputElement).files?.[0]
    if (!f || !file) return
    const r = await send('updateWork', { house: house.id, id: file.node.id, fromPath: await stageFile(f) })
    if (r) { saved = r.becameProposal ? 'proposed' : 'written'; reloads++ }
  }
  let openDiff = $state<number | null>(null)
  // 带走整间茶馆（export）：全部历史 + 可以直接打开的文件夹 + 这台设备上有的附件
  let exporting = $state(false)
  let exported = $state<{ path?: string; error?: string } | null>(null)
  async function takeAway() {
    if (exporting) return
    exporting = true; exported = null
    try {
      const path = await api('export', { house: house.id, to: exportTo })
      exported = { path }
      // 安卓：让"文件"应用看得到导出的文件（MainActivity.kt 的 scanFolder）
      ;(window as { TeahouseAndroid?: { scanFolder?: (p: string) => void } }).TeahouseAndroid?.scanFolder?.(path)
    }
    catch (e) { exported = { error: e instanceof BackendError ? e.error.message : String(e) } }
    exporting = false
  }

  // 目录名由界面翻译（引擎给代码 dir）；用户写的内容（标题、名字）直接用 label
  const label = (n: ArchiveNode) => (n.dir ? t(`archive.${n.dir}`) : n.kind === 'members' ? t('archive.members') : (n.label ?? ''))
  // "按问题"视图里，问题文件就在问题的文件夹里，标成"问题说明"
  const leafLabel = (n: ArchiveNode, depth: number) => (mode === 'problems' && depth > 0 && n.kind === 'problem' ? t('archive.problemFile') : label(n))

  const path = (n: ArchiveNode, target: string, trail: string[] = []): string[] | null => {
    if (!n.children) return n.id === target ? trail : null
    for (const c of n.children) {
      const r = path(c, target, c.children ? [...trail, label(c)] : trail)
      if (r) return r
    }
    return null
  }
  let crumbs = $derived(root && file ? path(root, file.node.id) ?? [] : [])

  function size(n = 0) {
    const f = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 })
    return n < 1024 ? `${n} B` : n < 1024 * 1024 ? `${f.format(n / 1024)} KB` : `${f.format(n / 1024 / 1024)} MB`
  }

  // ---- 把文件原文排出来 ----
  /** frontmatter + 正文。正文按空行分段；"## " 开头的是小标题 */
  function markdown(src = '') {
    const m = src.match(/^---\n([\s\S]*?)\n---\n?/)
    const front: Record<string, string> = {}
    for (const line of (m?.[1] ?? '').split('\n')) {
      const i = line.indexOf(':')
      if (i > 0) front[line.slice(0, i).trim()] = line.slice(i + 1).trim()
    }
    const blocks = (m ? src.slice(m[0].length) : src).split(/\n{2,}/).filter((b) => b.trim())
    return {
      front,
      blocks: blocks.map((b) => (b.startsWith('## ') ? { h: b.slice(3).split('\n')[0], p: b.split('\n').slice(1).join('\n') } : { h: '', p: b })),
    }
  }
  /** 一段文字按行切开：单独一行的 ![标题](地址) 是图片，相邻的其余行合成一段文字 */
  function pieces(p: string) {
    const out: { text?: string; img?: { alt: string; src: string } }[] = []
    for (const line of p.split('\n')) {
      const m = /^\s*!\[([^\]]*)\]\(([^)\s]+)\)\s*$/.exec(line)
      if (m) out.push({ img: { alt: m[1], src: m[2] } })
      else if (out.length && out[out.length - 1].text !== undefined) out[out.length - 1].text += '\n' + line
      else out.push({ text: line })
    }
    return out.filter((x) => x.img || x.text?.trim())
  }
  /** CSV → 行（支持引号） */
  function csv(src = '') {
    return src.split('\n').filter(Boolean).map((line) => {
      const out: string[] = []
      let cur = '', q = false
      for (let i = 0; i < line.length; i++) {
        const c = line[i]
        if (q) { if (c === '"' && line[i + 1] === '"') { cur += '"'; i++ } else if (c === '"') q = false; else cur += c }
        else if (c === '"') q = true
        else if (c === ',') { out.push(cur); cur = '' }
        else cur += c
      }
      out.push(cur)
      return out
    })
  }

  const problemLabel = (pid: string) => {
    const p = house.problems.find((x) => x.id === pid)
    return p ? `${p.id} · ${p.title}` : pid
  }
  function forWhat(e: HistoryEntry) {
    return e.problem === 'board' ? t('archive.forBoard') : e.problem === 'meta' ? t('archive.forMeta') : t('archive.for', { problem: problemLabel(e.problem) })
  }
  // 和"经过"一样：title 是提交说明的原话，有就直接显示；没有时才按动作给通用说法
  function actText(e: HistoryEntry) {
    if (e.title) return e.title
    switch (e.act) {
      case 'open-problem': return t('ev.open')
      case 'set-status': return t(e.status === 'solved' ? 'ev.markSolved' : 'ev.markActive')
      case 'seat': return t('ev.seat')
      case 'adopt': return t('archive.act.merge')
      case 'update-work':
      case 'update': return t('archive.act.update')
      default: return t('archive.act.add')
    }
  }
  let history = $derived(file ? [...file.history].reverse() : [])

  // 下载的就是文件原文
  function download() {
    if (!file?.text) return
    const a = document.createElement('a')
    a.href = URL.createObjectURL(new Blob([file.text], { type: 'text/plain;charset=utf-8' }))
    a.download = `${label(file.node).replace(/[\\/:*?"<>|]/g, '_')}.${file.node.kind === 'table' ? 'csv' : file.node.kind === 'members' ? 'toml' : 'md'}`
    a.click()
    URL.revokeObjectURL(a.href)
  }
</script>

{#snippet branch(nodes: ArchiveNode[], depth: number)}
  <ul class="branch" class:root={depth === 0}>
    {#each nodes as n (n.id)}
      <li>
        {#if !n.children}
          <a class="row file" class:on={n.id === selected} href="#/{house.id}/archive/{encodeURIComponent(n.id)}" style:--d={depth} data-file={n.id}>
            <FileGlyph kind={n.kind} />
            <span class="name"><span class="real">{fileName(n.id)}</span><span class="human">{leafLabel(n, depth)}</span></span>
            <AccessMark a={access(house, n.id)} />
          </a>
        {:else}
          <button class="row folder" style:--d={depth} aria-expanded={!closed[n.id]} onclick={() => (closed[n.id] = !closed[n.id])}>
            <FileGlyph kind="dir" open={!closed[n.id]} />
            <span class="name">{#if mode === 'dirs'}<span class="real">{fileName(n.id)}/</span>{/if}<span class:human={mode === 'dirs'}>{label(n)}</span></span>
            <span class="count num">{n.children.length}</span>
          </button>
          {#if !closed[n.id] && n.children.length}{@render branch(n.children, depth + 1)}{/if}
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
      <p class="lede">{t('archive.lede')} <span class="num">{t('archive.files', { n: all.length })}</span></p>
    </div>
    <div class="export">
      <button class="secondary" aria-busy={exporting || undefined} data-sending={t('archive.exporting')} onclick={takeAway}>
        <i class="box"></i>{t('archive.export')}
      </button>
      <p class="tagline">{t('archive.exportLine')}</p>
      {#if exported?.path}
        <div class="exported" role="status">
          <b>{t('archive.exported')}</b>
          <!-- 手机上的"下载"文件夹用人话写 -->
          <code>{exported.path.replace(/^\/storage\/emulated\/0\/Download\//, `${t('archive.downloads')}/`)}</code>
          <p>{t('archive.exportNote')}</p>
        </div>
      {:else if exported?.error}
        <p class="exported failed" role="alert">{t('archive.exportFailed')} {exported.error}</p>
      {/if}
    </div>
  </header>

  <div class="cols">
    <aside class="tree">
      <div class="seg" role="group">
        <button class:on={mode === 'dirs'} aria-pressed={mode === 'dirs'} onclick={() => (mode = 'dirs')}>{t('archive.byDir')}</button>
        <button class:on={mode === 'problems'} aria-pressed={mode === 'problems'} onclick={() => (mode = 'problems')}>{t('archive.byProblem')}</button>
      </div>
      <div class="legend" aria-label={t('access.legend')}>
        <AccessMark a="write" label /><AccessMark a="propose" label /><AccessMark a="read" label />
      </div>
      {#if root?.children}<nav aria-label={t('archive.title')}>{@render branch(root.children, 0)}</nav>{/if}
    </aside>

    {#if file}
    {@const node = file.node}
    <section class="view">
      <a class="back label" href="#/{house.id}/archive">← {t('archive.back')}</a>
      {#if crumbs.length}<div class="crumbs label">{crumbs.join(' / ')}</div>{/if}

      <div class="fhead">
        <FileGlyph kind={node.kind} />
        <h2>{label(node)}</h2>
        <span class="meta">{t(`archive.kind.${node.kind}`)} · {size(node.size)}</span>
      </div>

      <div class="tools">
        {#if editable}
          {#if !editing}
            <button class="primary small" data-write onclick={startEdit}><i><AccessMark a={can} /></i><span>{t(can === 'write' ? 'edit.start' : 'edit.startPropose')}</span></button>
            <span class="consequence" data-write>{t(can === 'write' ? 'access.writeHint' : 'access.proposeHint')}</span>
          {/if}
        {:else if isWork(node) && can !== 'read'}
          <button class="primary small" data-write onclick={() => picker?.click()}><i><AccessMark a={can} /></i><span>{t('edit.replaceFile')}</span></button>
          <input class="picker" type="file" bind:this={picker} onchange={replace} />
          <span class="consequence" data-write>{t(can === 'write' ? 'access.writeHint' : 'access.proposeHint')}</span>
        {:else if can === 'read' && readReason(node.id) !== 'role'}
          <span class="readonly"><AccessMark a="read" label /></span>
          <span class="consequence">{t(readReason(node.id) === 'seats' ? 'access.readSeats' : 'access.readTalk')}</span>
        {:else if node.kind !== 'note' && node.kind !== 'members'}
          {#if me.role === 'guest'}
            <a class="primary small" data-write href="#/{house.id}/propose/{node.problem === 'meta' ? '' : node.problem}"><i>+</i><span>{t('archive.proposeEdit')}</span></a>
            <span class="consequence" data-write>{t('archive.proposeHint')}</span>
          {:else if node.problem && node.problem !== 'board' && node.problem !== 'meta'}
            <a class="secondary small" data-write href="#/{house.id}/problem/{node.problem}">{t('archive.edit')}</a>
          {/if}
        {/if}
        <button class="secondary small" onclick={download} disabled={!file.text} title={file.text ? '' : t('archive.noFile')}>↓ {t('archive.download')}</button>
      </div>

      {#if saved}
        <p class="saved" role="status">{t(saved === 'written' ? 'edit.written' : 'edit.proposed')}</p>
      {/if}

      {#if editing}
        <div class="editing">
          <Editor value={draft} markdown={node.kind !== 'table'} onchange={(v) => (draft = v)}
            onimage={node.kind !== 'table' && node.problem ? (f) => attachImage(house.id, node.problem!, f) : undefined}
            resolveImage={(src) => imageUrl(house.id, src)} />
          <div class="save">
            <button class="primary" data-write onclick={save}><span>{t(can === 'write' ? 'edit.save' : 'edit.saveProposal')}</span></button>
            <button class="secondary" onclick={() => (editing = false)}>{t('edit.cancel')}</button>
          </div>
        </div>
      {:else}
      <article class="content">
        {#if node.kind === 'members'}
          <table>
            <thead><tr><th>{t('archive.colName')}</th><th>{t('archive.colRole')}</th><th>{t('archive.colJoined')}</th></tr></thead>
            <tbody>
              {#each house.members as m}
                <tr><td><span class="cell-who"><Shape role={m.role} size={10} />{m.name}<MachineMark {m} /></span></td><td>{t(`role.${m.role}`)}</td><td>{date(m.joined, 'short')}</td></tr>
              {/each}
            </tbody>
          </table>
        {:else if node.kind === 'table'}
          {#if file.text}
            {@const rows = csv(file.text)}
            <table>
              <thead><tr>{#each rows[0] as c}<th>{c}</th>{/each}</tr></thead>
              <tbody>{#each rows.slice(1) as r}<tr>{#each r as c}<td>{c}</td>{/each}</tr>{/each}</tbody>
            </table>
          {:else}
            <div class="binary"><FileGlyph kind="table" /><p>{t('archive.fileOnly')}</p></div>
          {/if}
        {:else if node.kind === 'image' || node.kind === 'file'}
          {#if image === 'fake:parking'}
            <ParkingMap />
          {:else if image}
            <img class="img" src={blobUrl(house.id, node.id, image)} alt={label(node)} />
          {:else}
            <div class="binary image"><svg viewBox="0 0 120 80" aria-hidden="true"><rect x="1" y="1" width="118" height="78" fill="none" stroke="var(--ink)" stroke-width="2" /><circle cx="88" cy="24" r="8" fill="var(--ink)" /><polygon points="10,72 44,26 78,72" fill="var(--ink)" /></svg><p>{t('archive.fileOnly')}</p></div>
          {/if}
        {:else if node.kind === 'note'}
          {@const first = file.history[0]}
          {#if first}
            {@const m = member(house, first.by)}
            <div class="who"><Shape role={m.role} size={10} /><b>{m.name}</b><MachineMark {m} /><span>{when(first.when)}</span></div>
          {/if}
          <p class="pre-plain">{file.text}</p>
        {:else}
          {@const md = markdown(file.text)}
          {#if md.front.status}<StatusMark status={md.front.status as Status} />{/if}
          {#if md.front.title}<h3>{md.front.title}</h3>{/if}
          {#if md.front.note}<p class="sum">{md.front.note}</p>{/if}
          {#each md.blocks as b, i}
            {#if b.h}<h4>{b.h}</h4>{/if}
            {#if b.p}
              <!-- 只有一张图片的行显示成图片，其余照常是文字 -->
              {#each pieces(b.p) as piece}
                {#if piece.img}<MdImage house={house.id} src={piece.img.src} alt={piece.img.alt} />
                {:else}<p class:sum={node.kind === 'problem' && i === 0 && !b.h} class="pre-plain">{piece.text}</p>{/if}
              {/each}
            {/if}
          {/each}
        {/if}
      </article>
      {/if}

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
                  <span class="by"><Shape role={m.role} size={10} /><b>{m.name}</b><MachineMark {m} /> {actText(e)}</span>
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
    {/if}
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
  .readonly { display: inline-flex; align-items: center; min-height: 40px; padding: 0 14px; border: 1.5px solid var(--ink); font-size: 13px; font-weight: 600; }
  .exported { display: grid; gap: 6px; padding: 12px 14px; border: 2px solid var(--ink); font-size: 13px; line-height: 1.6; text-align: left; }
  .exported code { font-family: var(--latin); font-size: 12px; overflow-wrap: anywhere; background: var(--paper-deep); padding: 4px 6px; }
  .exported p { color: var(--ink-2); }
  .box { display: block; width: 14px; height: 14px; border: 2px solid currentColor; border-top-width: 5px; }
  .tagline { font-family: var(--display); font-weight: 700; font-size: 15px; letter-spacing: var(--track); }

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
  .name { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; overflow-wrap: anywhere; }
  /* 真实的文件名在上，人话的标题在下（灰） */
  .real { font-weight: 500; }
  .folder .real { font-weight: 700; }
  .human { font-size: 12px; font-weight: 400; color: var(--ink-3); }
  .file.on .human, .file.on :global(.access) { color: var(--paper); }
  .legend { display: flex; flex-wrap: wrap; gap: 6px 16px; margin-bottom: 12px; }

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
  .editing { padding: 16px 0 8px; max-width: 46em; }
  .save { display: flex; flex-wrap: wrap; gap: 10px; margin-top: 14px; }
  .saved { margin-top: 4px; padding: 10px 14px; background: var(--ink); color: var(--paper); font-size: 14px; }
  .picker { display: none; }
  .content h3 { font-family: var(--display); font-weight: 700; font-size: 22px; line-height: 1.3; margin: 10px 0 12px; letter-spacing: var(--track); }
  .content h4 { margin: 18px 0 4px; font-size: 13px; font-weight: 700; letter-spacing: 0.06em; }
  .content p { font-size: 15px; line-height: 1.85; }
  .content .sum { color: var(--ink-2); }
  .content .pre-plain { white-space: pre-line; }
  .img { display: block; max-width: 100%; }
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
  }
</style>
