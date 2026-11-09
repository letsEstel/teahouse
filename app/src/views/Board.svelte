<script lang="ts">
  import type { NoteAttachment } from '../api/types'
  import Markdown from '../lib/Markdown.svelte'
  import Shape from '../lib/Shape.svelte'
  import Notes from '../lib/Notes.svelte'
  import Write from '../lib/Write.svelte'
  import { member, self, pad, type House, type Notice, type Note } from '../api/view'
  import { write } from '../api/backend'
  import { t, when } from '../i18n/index.svelte'

  // 公告板：老板和伙计张贴，所有人可以在下面回应。每条公告是 board/ 下的一个文件。
  // 写入都走引擎（postNotice / note），写完 refresh() 重新读取茶馆。
  let { house, refresh }: { house: House; refresh: () => void } = $props()
  let me = $derived(self(house))
  let canPost = $derived(me.role !== 'guest')
  let posting = $state(false)
  let replying = $state<string | null>(null)
  let notices = $derived(house.board)

  async function post(text: string, title: string) {
    if (!(await write('postNotice', { house: house.id, title, text }))) return false
    posting = false
    refresh()
  }
  // 茶客的留言直接写进茶馆；老板和伙计可以删除留言
  let writeHint = $derived(me.role === 'guest' ? t('write.hintGuest') : '')
  // 删除留言（老板、伙计）：removeNote，然后重新读取
  async function remove(_n: Notice, note: Note) {
    if (await write('removeNote', { house: house.id, id: note.id })) refresh()
  }
  async function reply(n: Notice, text: string, attachments: NoteAttachment[]) {
    if (!(await write('note', { house: house.id, notice: n.id, text, attachments: attachments.length ? attachments : undefined }))) return false
    replying = null
    refresh()
  }
  // 回应的收纳（0032）：默认只看最近 3 条，其余收起；最新在上还是在下，每台设备记住
  const SHOWN = 3
  // 长公告（0019）：正文太长时只露出前几行，点“展开全文”再看全部。
  // 旧公告（不是最新那条）的回应整个收起，只留一行“看 N 条回应”；最新那条照旧露出最近 3 条
  const long = (text: string) => text.length > 280 || text.split('\n').length > 7
  let unfolded = $state(new Set<string>())
  const toggleBody = (id: string) => (unfolded = unfolded.has(id) ? new Set([...unfolded].filter((x) => x !== id)) : new Set([...unfolded, id]))
  const folded = (text: string, id: string) => long(text) && !unfolded.has(id)
  let repliesOpen = $state(new Set<string>())
  const openReplies = (id: string) => (repliesOpen = new Set([...repliesOpen, id]))
  let expanded = $state(new Set<string>())
  const expand = (id: string) => (expanded = new Set([...expanded, id]))
  const collapse = (id: string) => (expanded = new Set([...expanded].filter((x) => x !== id)))
  let newestFirst = $state((() => { try { return localStorage.getItem('teahouse.replyOrder') !== 'oldest' } catch { return true } })())
  function setOrder(newest: boolean) {
    newestFirst = newest
    try { localStorage.setItem('teahouse.replyOrder', newest ? 'newest' : 'oldest') } catch {}
  }
</script>

<div class="wrap">
  <header>
    <div class="label">02 · {t('nav.board')}</div>
    <h1>{t('board.title')}</h1>
    {#if canPost}
      {#if posting}
        <div class="compose" data-write>
          <Write title titlePlaceholder={t('board.titlePh')} placeholder={t('board.bodyPh')} rows={5} onsubmit={post} />
        </div>
      {:else}
        <div class="action" data-write>
          <button class="primary" onclick={() => (posting = true)}><i>+</i><span>{t('board.post')}</span></button>
          <p class="consequence">{t('board.postHint')}</p>
        </div>
      {/if}
    {:else}
      <div class="action" data-write>
        <a class="primary" href="#/{house.id}/propose/board"><i>+</i><span>{t('board.suggest')}</span></a>
        <p class="consequence">{t('board.suggestHint')}</p>
      </div>
      <p class="consequence guest" data-write>{t('board.guestNote')}</p>
    {/if}
  </header>

  <section class="list">
    {#each notices as n, i (n.id)}
      {@const m = member(house, n.by)}
      <article class:latest={i === 0}>
        <div class="no num">{pad(notices.length - i)}</div>
        <div class="body">
          {#if i === 0}<div class="label tag">{t('board.latest')}</div>{/if}
          <h2>{n.title}</h2>
          <div class="by">
            <Shape role={m.role} size={10} />
            <span>{t('board.postedBy', { name: m.name })} · {t(`role.${m.role}`)}</span>
            <span class="when">{when(n.when)}</span>
          </div>
          <!-- 公告正文按 Markdown 显示（0037），和问题正文一样 -->
          <div class="text" class:folded={folded(n.text, n.id)}><Markdown text={n.text} house={house.id} /></div>
          {#if long(n.text)}
            <button class="more body-more" aria-expanded={!folded(n.text, n.id)} onclick={() => toggleBody(n.id)}>{t(folded(n.text, n.id) ? 'board.readMore' : 'board.readLess')}</button>
          {/if}

          {#if i > 0 && n.replies.length && !repliesOpen.has(n.id)}
            <button class="more replies-more" onclick={() => openReplies(n.id)}>{t('board.showReplies', { n: n.replies.length })}</button>
            {#if replying === n.id}
              <Write attach placeholder={t('board.replyPh')} rows={2} hint={writeHint} onsubmit={(text, _t, files) => reply(n, text, files)} />
            {:else}
              <button class="reply" data-write onclick={() => (replying = n.id)}>{t('board.reply')} →</button>
            {/if}
          {:else}
          <div class="replies">
            <div class="label rh">{t('board.replies', { n: n.replies.length })}</div>
            {#if n.replies.length > 1}
              <div class="order" role="group" aria-label={t('board.order')}>
                <button class:on={newestFirst} aria-pressed={newestFirst} onclick={() => setOrder(true)}>{t('board.newest')}</button>
                <button class:on={!newestFirst} aria-pressed={!newestFirst} onclick={() => setOrder(false)}>{t('board.oldest')}</button>
              </div>
            {/if}
            {#if n.replies.length}
              {@const all = newestFirst ? [...n.replies].reverse() : n.replies}
              {@const open = expanded.has(n.id) || all.length <= SHOWN}
              {@const shown = open ? all : newestFirst ? all.slice(0, SHOWN) : all.slice(-SHOWN)}
              {#if !open && !newestFirst}<button class="more" onclick={() => expand(n.id)}>{t('board.more', { n: all.length - SHOWN })}</button>{/if}
              <Notes {house} notes={shown} ondelete={canPost ? (note) => remove(n, note) : undefined} />
              {#if !open && newestFirst}<button class="more" onclick={() => expand(n.id)}>{t('board.more', { n: all.length - SHOWN })}</button>{/if}
              {#if open && all.length > SHOWN}<button class="more" onclick={() => collapse(n.id)}>{t('board.less')}</button>{/if}
            {/if}
            {#if replying === n.id}
              <Write attach placeholder={t('board.replyPh')} rows={2} hint={writeHint} onsubmit={(text, _t, files) => reply(n, text, files)} />
            {:else}
              <button class="reply" data-write onclick={() => (replying = n.id)}>{t('board.reply')} →</button>
            {/if}
          </div>
          {/if}
        </div>
      </article>
    {:else}
      <p class="none">{t('board.none')}</p>
    {/each}
  </section>
</div>

<style>
  .order { display: flex; }
  .order button { padding: 3px 10px; border: 1px solid var(--ink); font-size: 12px; }
  .order button + button { border-left: 0; }
  .order button.on { background: var(--ink); color: var(--paper); }
  .more { align-self: flex-start; font-size: 13px; font-weight: 700; text-decoration: underline; text-underline-offset: 3px; }
  .wrap {
    display: grid;
    grid-template-columns: clamp(260px, 26vw, 400px) minmax(0, 1fr);
    gap: clamp(32px, 4.4vw, 72px);
    padding: var(--pad-top) var(--pad-x) 80px;
  }
  header { position: sticky; top: var(--pad-top); align-self: start; display: flex; flex-direction: column; gap: 28px; }
  h1 {
    font-family: var(--display); font-weight: var(--heavy); font-size: clamp(48px, 5vw, 76px);
    line-height: 1; letter-spacing: var(--track); margin-top: -6px;
  }
  .action { display: flex; flex-direction: column; align-items: flex-start; gap: 10px; }
  .guest { font-size: 14px; color: var(--ink); border-left: 4px solid var(--ink); padding-left: 16px; }

  .list { min-width: 0; border-top: 2px solid var(--ink); }
  article { display: grid; grid-template-columns: auto minmax(0, 1fr); gap: clamp(20px, 2.5vw, 36px); padding: 28px 0 32px; border-bottom: 1px solid var(--ink); }
  .no { font-size: clamp(40px, 3.6vw, 56px); font-weight: 300; line-height: 0.9; min-width: 1.4em; }
  .latest .no { font-weight: 400; }
  .tag { display: inline-block; background: var(--ink); color: var(--paper); padding: 2px 8px; margin-bottom: 12px; }
  h2 { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(22px, 2vw, 30px); line-height: 1.25; letter-spacing: var(--track); }
  .by { display: flex; flex-wrap: wrap; align-items: center; gap: 4px 10px; margin-top: 10px; font-size: 13px; color: var(--ink-2); }
  .by .when { font-family: var(--latin); color: var(--ink-3); }
  .text { margin-top: 16px; font-size: 16px; line-height: 1.85; max-width: 38em; }
  /* 收起时只露前 6 行，下面一道虚线表示“还有”，不用渐变 */
  .text.folded { max-height: calc(1.85em * 6); overflow: hidden; border-bottom: 1.5px dashed var(--ink); }
  .body-more { margin-top: 10px; }
  .replies-more { display: block; margin-top: 20px; }

  .replies { margin-top: 26px; padding-left: clamp(16px, 2vw, 28px); border-left: 1px solid var(--ink); max-width: 44em; display: flex; flex-direction: column; gap: 10px; }
  .rh { padding-bottom: 4px; }
  .reply { align-self: flex-start; font-size: 13px; font-weight: 700; border-bottom: 1.5px solid var(--ink); padding: 4px 0 2px; margin-top: 4px; }
  .none { padding: 28px 0; color: var(--ink-2); }

  @media (max-width: 1023.98px) {
    .wrap { grid-template-columns: minmax(0, 1fr); }
    header { position: static; }
  }
</style>
