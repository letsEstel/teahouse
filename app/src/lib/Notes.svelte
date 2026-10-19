<script lang="ts">
  import Shape from './Shape.svelte'
  import { member, type House, type Note } from '../api/view'
  import { t, when, date } from '../i18n/index.svelte'
  // 写下来的话：像信纸上的段落一样排，不做气泡。
  // 一条留言一个作者，写下不改；老板和伙计可以删除（管理）。
  // 顺序是进入茶馆的先后（引擎给出）；显示写下的时间 writtenAt。
  // 写下的时间早于前一条的（离线写、晚到的），用灰色小字注明"写于 …"。
  let { house, notes, ondelete }: { house: House; notes: Note[]; ondelete?: (n: Note) => void } = $props()
  let confirming = $state<Note | null>(null)
</script>

<ol class="notes">
  {#each notes as n, i}
    {@const m = member(house, n.by)}
    {@const late = i > 0 && n.writtenAt < notes[i - 1].writtenAt}
    <li>
      <div class="who">
        <Shape role={m.role} size={10} />
        <b>{m.name}</b>
        <span class="role">{t(`role.${m.role}`)}</span>
        <span class="when">{late ? t('note.writtenAt', { date: date(n.writtenAt, 'short') }) : when(n.writtenAt)}</span>
        {#if ondelete}
          {#if confirming === n}
            <button class="del sure" data-write title={t('note.deleteHint')} onclick={() => { ondelete(n); confirming = null }}>{t('note.confirm')}</button>
          {:else}
            <button class="del" data-write title={t('note.deleteHint')} onclick={() => (confirming = n)}>{t('note.delete')}</button>
          {/if}
        {/if}
      </div>
      <p>{n.text}</p>
    </li>
  {/each}
</ol>

<style>
  .notes { list-style: none; }
  li { padding: 14px 0 16px; border-bottom: 1px solid var(--rule); }
  li:first-child { padding-top: 4px; }
  .who { display: flex; flex-wrap: wrap; align-items: center; gap: 4px 10px; font-size: 13px; }
  .who b { font-weight: 700; }
  .role { color: var(--ink-2); }
  .when { margin-left: auto; font-family: var(--latin); font-size: 12px; color: var(--ink-3); }
  .del { font-size: 12px; color: var(--ink-3); border-bottom: 1px solid transparent; }
  .del:hover { color: var(--ink); border-bottom-color: var(--ink); }
  .del.sure { color: var(--paper); background: var(--ink); padding: 0 6px; }
  p { margin-top: 6px; font-size: 15px; line-height: 1.8; max-width: 40em; white-space: pre-line; }
</style>
