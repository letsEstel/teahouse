<script lang="ts">
  import Shape from './Shape.svelte'
  import { member, type House, type Note } from '../data'
  import { t, L, when } from '../i18n/index.svelte'
  // 写下来的话：像信纸上的段落一样排，不做气泡。
  let { house, notes }: { house: House; notes: Note[] } = $props()
</script>

<ol class="notes">
  {#each notes as n}
    {@const m = member(house, n.by)}
    <li>
      <div class="who">
        <Shape role={m.role} size={10} />
        <b>{L(m.name)}</b>
        <span class="role">{t(`role.${m.role}`)}</span>
        <span class="when">{when(n.when)}</span>
      </div>
      <p>{L(n.text)}</p>
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
  p { margin-top: 6px; font-size: 15px; line-height: 1.8; max-width: 40em; white-space: pre-line; }
</style>
