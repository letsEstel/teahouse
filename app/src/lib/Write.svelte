<script lang="ts">
  import { t } from '../i18n/index.svelte'
  // 写字的地方：有输入框，但不是聊天。写完点"写进茶馆"，每一条都是一次提交。
  let {
    placeholder,
    title = false,
    titlePlaceholder = '',
    rows = 3,
    hint = '',
    onsubmit,
  }: {
    placeholder: string
    title?: boolean
    titlePlaceholder?: string
    rows?: number
    hint?: string
    onsubmit: (text: string, title: string) => void
  } = $props()

  let text = $state('')
  let head = $state('')
  let ready = $derived(text.trim().length > 0 && (!title || head.trim().length > 0))

  function submit(e: SubmitEvent) {
    e.preventDefault()
    if (!ready) return
    onsubmit(text.trim(), head.trim())
    text = ''
    head = ''
  }
</script>

<form class="write" onsubmit={submit}>
  {#if title}
    <input class="head" bind:value={head} placeholder={titlePlaceholder} aria-label={titlePlaceholder} />
  {/if}
  <textarea bind:value={text} {placeholder} aria-label={placeholder} {rows}></textarea>
  <div class="foot">
    <p class="consequence">{hint || t('write.hint')}</p>
    <button class="submit" type="submit" disabled={!ready}>{t('write.submit')}</button>
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
