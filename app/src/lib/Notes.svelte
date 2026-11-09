<script lang="ts">
  import Markdown from './Markdown.svelte'
  import MdImage from './MdImage.svelte'
  import FileGlyph from './FileGlyph.svelte'
  import Shape from './Shape.svelte'
  import MachineMark from './MachineMark.svelte'
  import { member, type House, type Note } from '../api/view'
  import { t, when, date } from '../i18n/index.svelte'
  // 写下来的话：像信纸上的段落一样排，不做气泡。
  // 一条留言一个作者，写下不改；老板和伙计可以删除（管理）。
  // 顺序是进入茶馆的先后（引擎给出）；显示写下的时间 writtenAt。
  // 写下的时间早于前一条的（离线写、晚到的），用灰色小字注明"写于 …"。
  let { house, notes, ondelete }: { house: House; notes: Note[]; ondelete?: (n: Note) => void } = $props()
  let confirming = $state<Note | null>(null)
  /** 正文里没插的附件，单独列在下面 */
  const loose = (n: Note) => (n.attachments ?? []).filter((a) => !n.text.includes(a.id))
</script>

<ol class="notes">
  {#each notes as n, i}
    {@const m = member(house, n.by)}
    {@const late = i > 0 && n.writtenAt < notes[i - 1].writtenAt}
    <li>
      <div class="who">
        <Shape role={m.role} size={10} />
        <b>{m.name}</b><MachineMark {m} />
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
      {#if n.text}<div class="text"><Markdown text={n.text} house={house.id} /></div>{/if}
      <!-- 附件（引擎 #117）：正文里没插的图片在下面显示；文件列出来，点进档案能存到这台设备 -->
      {#if loose(n).length}
        <div class="atts">
          {#each loose(n) as a}
            {#if a.kind === 'image'}
              <div class="img"><MdImage house={house.id} src={a.id} alt={a.title} /></div>
            {:else}
              <a class="file" href="#/{house.id}/archive/{encodeURIComponent(a.id)}"><FileGlyph kind="file" /><span>{a.title}</span></a>
            {/if}
          {/each}
        </div>
      {/if}
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
  .text { margin-top: 6px; font-size: 15px; line-height: 1.8; max-width: 40em; }
  .atts { display: flex; flex-wrap: wrap; gap: 10px; margin-top: 10px; }
  .img { max-width: min(100%, 360px); }
  .file { display: inline-flex; align-items: center; gap: 8px; border: 1px solid var(--ink); padding: 6px 10px; font-size: 13px; color: inherit; text-decoration: none; max-width: 100%; }
  .file :global(svg) { width: 18px; height: 18px; flex: none; }
  .file span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .file:hover { background: var(--paper-deep); }
</style>
