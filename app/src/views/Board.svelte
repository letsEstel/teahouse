<script lang="ts">
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
  async function reply(n: Notice, text: string) {
    if (!(await write('note', { house: house.id, notice: n.id, text }))) return false
    replying = null
    refresh()
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
          <p class="text">{n.text}</p>

          <div class="replies">
            <div class="label rh">{t('board.replies', { n: n.replies.length })}</div>
            {#if n.replies.length}<Notes {house} notes={n.replies} ondelete={canPost ? (note) => remove(n, note) : undefined} />{/if}
            {#if replying === n.id}
              <Write placeholder={t('board.replyPh')} rows={2} hint={writeHint} onsubmit={(text) => reply(n, text)} />
            {:else}
              <button class="reply" data-write onclick={() => (replying = n.id)}>{t('board.reply')} →</button>
            {/if}
          </div>
        </div>
      </article>
    {:else}
      <p class="none">{t('board.none')}</p>
    {/each}
  </section>
</div>

<style>
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
  .text { margin-top: 16px; font-size: 16px; line-height: 1.85; max-width: 38em; white-space: pre-line; }

  .replies { margin-top: 26px; padding-left: clamp(16px, 2vw, 28px); border-left: 1px solid var(--ink); max-width: 44em; display: flex; flex-direction: column; gap: 10px; }
  .rh { padding-bottom: 4px; }
  .reply { align-self: flex-start; font-size: 13px; font-weight: 700; border-bottom: 1.5px solid var(--ink); padding: 4px 0 2px; margin-top: 4px; }
  .none { padding: 28px 0; color: var(--ink-2); }

  @media (max-width: 1023.98px) {
    .wrap { grid-template-columns: minmax(0, 1fr); }
    header { position: static; }
  }
</style>
