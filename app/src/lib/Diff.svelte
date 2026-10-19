<script lang="ts">
  import type { DiffLine } from '../api/view'
  import {} from '../i18n/index.svelte'
  // 差异视图（提案审阅、档案里的"来历"共用）：
  // 新增 = 墨黑实线标记 + 墨色正文；删除 = 删除线 + 灰色。不借用角色色。
  let { lines, muted = false }: { lines: DiffLine[]; muted?: boolean } = $props()
</script>

<div class="lines" class:muted>
  {#each lines as l, i}
    <div class="line {l.kind}">
      <span class="ln num">{String(i + 1).padStart(2, '0')}</span>
      <span class="tx">{l.text || ' '}</span>
    </div>
  {/each}
</div>

<style>
  .lines { padding: 6px 0 4px; }
  .line {
    display: grid;
    grid-template-columns: 34px 16px minmax(0, 1fr);
    align-items: baseline;
    font-size: 15px;
    line-height: 1.75;
    padding: 3px 0;
    position: relative;
  }
  .ln { font-size: 11px; color: var(--ink-3); }
  .tx { grid-column: 3; white-space: pre-wrap; }
  .line.add::before { content: ''; position: absolute; left: 34px; top: 5px; bottom: 5px; width: 5px; background: var(--ink); }
  .line.add .tx { font-weight: 500; }
  .line.del .tx { color: var(--ink-3); text-decoration: line-through; text-decoration-thickness: 1.5px; }
  .muted .line.add .tx { color: var(--ink-3); font-weight: 400; }
  .muted .line.add::before { background: var(--ink-3); }
</style>
