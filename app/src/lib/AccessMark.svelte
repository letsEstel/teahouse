<script lang="ts">
  // 文件旁边的权限标记：✎ 能直接改，↗ 改动会变成提案，锁 = 只能看。
  // 锁用几何画出来（方块 + 半圆），和其他图形一样只用墨色，不用彩色表情。
  import type { Access } from '../api/view'
  import { t } from '../i18n/index.svelte'

  let { a, label = false }: { a: Access; label?: boolean } = $props()
</script>

<span class="access {a}" title={t(`access.${a}Hint`)}>
  {#if a === 'read'}
    <svg width="10" height="12" viewBox="0 0 10 12" aria-hidden="true"><path d="M2.5 5.5V3.5a2.5 2.5 0 0 1 5 0v2" fill="none" stroke="currentColor" stroke-width="1.4" /><rect x="0.5" y="5.5" width="9" height="6" fill="currentColor" /></svg>
  {:else}
    <i aria-hidden="true">{a === 'write' ? '✎' : '↗'}</i>
  {/if}
  {#if label}<span class="txt">{t(`access.${a}`)}</span>{:else}<span class="sr">{t(`access.${a}`)}</span>{/if}
</span>

<style>
  .access {
    display: inline-flex;
    align-items: center;
    gap: 0.4em;
    flex: none;
    color: var(--ink);
    font-size: 0.8125rem;
    line-height: 1;
    white-space: nowrap;
  }
  .access i { font-style: normal; font-family: var(--latin); font-weight: 600; width: 12px; text-align: center; }
  .access svg { width: 12px; }
  .read { color: var(--ink-3); }
  .txt { font-size: 0.75rem; }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
</style>
