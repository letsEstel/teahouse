<script lang="ts">
  import type { WorkKind } from '../api/view'
  // 成果类型的几何符号，只用墨色线条
  let { kind, size = 40 }: { kind: WorkKind; size?: number } = $props()
</script>

<svg width={size} height={size} viewBox="0 0 40 40" aria-hidden="true">
  {#if kind === 'doc'}
    <rect x="8" y="4" width="24" height="32" fill="none" stroke="var(--ink)" stroke-width="2" />
    <line x1="13" y1="13" x2="27" y2="13" stroke="var(--ink)" stroke-width="2" />
    <line x1="13" y1="19" x2="27" y2="19" stroke="var(--ink)" stroke-width="2" />
    <line x1="13" y1="25" x2="22" y2="25" stroke="var(--ink)" stroke-width="2" />
  {:else if kind === 'table'}
    <rect x="4" y="8" width="32" height="24" fill="none" stroke="var(--ink)" stroke-width="2" />
    <line x1="4" y1="16" x2="36" y2="16" stroke="var(--ink)" stroke-width="2" />
    <line x1="16" y1="16" x2="16" y2="32" stroke="var(--ink)" stroke-width="2" />
    <line x1="26" y1="16" x2="26" y2="32" stroke="var(--ink)" stroke-width="2" />
  {:else if kind === 'plan'}
    <circle cx="14" cy="14" r="9" fill="none" stroke="var(--ink)" stroke-width="2" />
    <rect x="18" y="18" width="18" height="18" fill="none" stroke="var(--ink)" stroke-width="2" />
  {:else if kind === 'link'}
    <!-- 链接：两个相扣的圆环 -->
    <circle cx="15" cy="20" r="9" fill="none" stroke="var(--ink)" stroke-width="2" />
    <circle cx="25" cy="20" r="9" fill="none" stroke="var(--ink)" stroke-width="2" />
  {:else if kind === 'file'}
    <!-- 附件：方框，一半实心 -->
    <rect x="6" y="6" width="28" height="28" fill="none" stroke="var(--ink)" stroke-width="2" />
    <rect x="6" y="6" width="14" height="28" fill="var(--ink)" />
  {:else}
    <rect x="4" y="4" width="32" height="32" fill="none" stroke="var(--ink)" stroke-width="2" />
    <circle cx="27" cy="13" r="4" fill="var(--ink)" />
    <polygon points="6,34 18,16 30,34" fill="var(--ink)" />
  {/if}
</svg>

<style>
  svg { display: block; flex: none; }
</style>
