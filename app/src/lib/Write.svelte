<script lang="ts">
  import { t } from '../i18n/index.svelte'
  import FileGlyph from './FileGlyph.svelte'
  import { prepareFile } from './images'
  import type { NoteAttachment } from '../api/types'
  // 写字的地方：有输入框，但不是聊天。写完点"写进茶馆"，每一条都是一次提交。
  let {
    placeholder,
    title = false,
    titlePlaceholder = '',
    rows = 3,
    hint = '',
    attach = false,
    onsubmit,
  }: {
    placeholder: string
    title?: boolean
    titlePlaceholder?: string
    rows?: number
    hint?: string
    /** 能不能插图片和文件（留言，引擎 #117） */
    attach?: boolean
    /** 返回 false 表示没写进去（出错了），这时保留输入框里的内容 */
    onsubmit: (text: string, title: string, attachments: NoteAttachment[]) => void | boolean | Promise<void | boolean>
  } = $props()

  let text = $state('')
  let head = $state('')
  // 选好的附件：图片在正文里插一行 ![名字](<本机路径>)，写进去时引擎换成真正的地址；文件只列在下面
  // 输入框里只放一个短记号（图1、图2），不露出本机路径；写进去时再换成路径，引擎再换成附件的地址
  let files = $state<(NoteAttachment & { image: boolean; label: string; key: string })[]>([])
  let counter = 0
  let picker = $state<HTMLInputElement>()
  let area = $state<HTMLTextAreaElement>()
  let adding = $state(false)
  let ready = $derived((text.trim().length > 0 || files.length > 0) && (!title || head.trim().length > 0))
  const line = (f: { label: string; key: string }) => `![${f.label}](${f.key})`
  async function pick(e: Event) {
    const input = e.currentTarget as HTMLInputElement
    const chosen = [...(input.files ?? [])].slice(0, 10 - files.length)
    input.value = ''
    adding = true
    for (const f of chosen) {
      const staged = await prepareFile(f)
      if (!staged) continue
      const image = staged.file.type.startsWith('image/')
      const item = { fromPath: staged.path, name: staged.file.name, image, label: staged.file.name.replace(/\.[^.]+$/, ''), key: t('write.imageKey', { n: ++counter }) }
      files = [...files, item]
      if (image) {
        const at = area?.selectionStart ?? text.length
        const before = text.slice(0, at), after = text.slice(at)
        text = `${before}${before && !before.endsWith('\n') ? '\n' : ''}${line(item)}\n${after}`
      }
    }
    adding = false
  }
  function drop(f: (typeof files)[number]) {
    files = files.filter((x) => x !== f)
    if (f.image) text = text.replace(line(f) + '\n', '').replace(line(f), '')
  }

  let busy = $state(false)
  async function submit(e: SubmitEvent) {
    e.preventDefault()
    if (!ready || busy) return
    busy = true
    let body = text.trim()
    for (const f of files) if (f.image) body = body.split(`](${f.key})`).join(`](${f.fromPath})`)
    const ok = await onsubmit(body, head.trim(), files.map(({ fromPath, name }) => ({ fromPath, name })))
    busy = false
    if (ok === false) return
    text = ''
    head = ''
    files = []
    counter = 0
  }
</script>

<form class="write" onsubmit={submit}>
  {#if title}
    <input class="head" bind:value={head} placeholder={titlePlaceholder} aria-label={titlePlaceholder} />
  {/if}
  <textarea bind:this={area} bind:value={text} {placeholder} aria-label={placeholder} {rows}></textarea>
  {#if files.length}
    <ul class="files">
      {#each files as f}
        <li><FileGlyph kind={f.image ? 'image' : 'file'} /><span>{f.name}</span><button type="button" class="x" aria-label={t('write.removeFile', { name: f.name ?? '' })} onclick={() => drop(f)}>×</button></li>
      {/each}
    </ul>
  {/if}
  <div class="foot">
    {#if attach}
      <input bind:this={picker} type="file" multiple hidden onchange={pick} />
      <button type="button" class="attach" disabled={adding || files.length >= 10} onclick={() => picker?.click()}>{adding ? t('edit.attaching') : t('write.attach')}</button>
    {/if}
    <p class="consequence">{hint || t('write.hint')}</p>
    <button class="submit" type="submit" disabled={!ready || busy}>{t('write.submit')}</button>
  </div>
</form>

<style>
  .write { display: flex; flex-direction: column; border: 2px solid var(--ink); background: var(--paper); }
  .head, textarea {
    font: inherit;
    color: var(--ink);
    background: transparent;
    border: 0;
    padding: 14px 18px;
    outline: none;
    width: 100%;
  }
  .head { font-family: var(--display); font-weight: 700; font-size: 18px; border-bottom: 1px solid var(--ink); }
  textarea { font-size: 15px; line-height: 1.75; resize: vertical; min-height: 4.5em; }
  ::placeholder { color: var(--ink-3); opacity: 1; }
  .write:focus-within { outline: 3px solid var(--ink); outline-offset: 2px; }
  .foot { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 10px 20px; border-top: 1px solid var(--ink); padding-left: 18px; }
  .foot .consequence { flex: 1 1 16em; padding: 8px 0; font-size: 12px; }
  .attach { font-size: 13px; font-weight: 700; border-bottom: 1.5px solid var(--ink); padding: 2px 0; white-space: nowrap; }
  .attach:disabled { color: var(--ink-3); border-color: var(--ink-3); }
  .files { list-style: none; display: flex; flex-wrap: wrap; gap: 8px; padding: 0 18px 12px; }
  .files li { display: inline-flex; align-items: center; gap: 8px; max-width: 100%; border: 1px solid var(--ink); padding: 4px 4px 4px 8px; font-size: 13px; }
  .files li :global(svg) { width: 16px; height: 16px; flex: none; }
  .files span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
  .x { width: 24px; height: 24px; font-size: 16px; line-height: 1; }
  .x:hover { background: var(--ink); color: var(--paper); }
  .submit {
    align-self: stretch;
    padding: 12px 22px;
    background: var(--ink);
    color: var(--paper);
    font-weight: 700;
    letter-spacing: var(--track-wide);
    transition: background 200ms var(--ease), color 200ms var(--ease);
  }
  .submit:disabled { background: transparent; color: var(--ink-3); border-left: 1px solid var(--ink); cursor: default; }
  .submit:not(:disabled):hover { background: var(--paper-deep); color: var(--ink); }
</style>
