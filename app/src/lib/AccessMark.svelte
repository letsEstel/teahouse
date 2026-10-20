<script lang="ts">
  // 文件旁边的权限标记：✎ 能直接改，↗ 改动会变成提案，锁 = 只能看。
  // 锁用几何画出来（方块 + 半圆），和其他图形一样只用墨色，不用彩色表情。
  import type { Access } from '../api/view'
  import { t } from '../i18n/index.svelte'

  let { a, label = false }: { a: Access; label?: boolean } = $props()
</script>

<!-- 不显示文字时，标记本身就是一个带名字的图形（读屏软件念"能直接改"等） -->
<span class="access {a}" title={t(`access.${a}Hint`)} role={label ? undefined : 'img'} aria-label={label ? undefined : t(`access.${a}`)}>
  {#if a === 'read'}
    <svg width="10" height="12" viewBox="0 0 10 12" aria-hidden="true"><path d="M2.5 5.5V3.5a2.5 2.5 0 0 1 5 0v2" fill="none" stroke="currentColor" stroke-width="1.4" /><rect x="0.5" y="5.5" width="9" height="6" fill="currentColor" /></svg>
  {:else if a === 'write'}
    <!-- 铅笔：一根斜放的长方形，加一个三角形笔尖 -->
    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><path d="M8.6 0.9l2.5 2.5-6.6 6.6-2.5-2.5z" fill="currentColor" /><path d="M1.5 8l2.5 2.5-3.3 0.8z" fill="currentColor" /></svg>
  {:else}
    <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><path d="M2 10L10 2M4 2h6v6" fill="none" stroke="currentColor" stroke-width="1.6" /></svg>
  {/if}
  {#if label}<span class="txt">{t(`access.${a}`)}</span>{/if}
</span>

<style>
  .access {
    display: inline-flex;
    align-items: center;
    gap: 0.4em;
    flex: none;
    color: inherit;
    font-size: 0.8125rem;
    line-height: 1;
    white-space: nowrap;
  }
  .access svg { width: 12px; }
  .read { color: var(--ink-3); }
  .txt { font-size: 0.75rem; }
</style>
