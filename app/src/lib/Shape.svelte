<script lang="ts">
  import type { Role } from '../data'
  // 老板 = 红方，伙计 = 黄三角，茶客 = 蓝圆，空座 = 墨线圆
  let { role = null, size = 14 }: { role?: Role | null; size?: number } = $props()
</script>

<svg width={size} height={size} viewBox="0 0 20 20" aria-hidden="true">
  {#if role === 'boss'}
    <rect x="1" y="1" width="18" height="18" fill="var(--red)" />
  {:else if role === 'heir'}
    <polygon points="10,1 19.5,19 0.5,19" fill="var(--yellow)" />
  {:else if role === 'guest'}
    <circle cx="10" cy="10" r="9" fill="var(--blue)" />
  {:else}
    <circle cx="10" cy="10" r="8.5" fill="none" stroke="var(--ink)" stroke-width="1.2" />
  {/if}
</svg>

<style>
  svg { display: block; flex: none; }
</style>
