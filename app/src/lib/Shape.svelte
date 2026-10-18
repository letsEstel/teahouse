<script lang="ts">
  import type { Role } from '../data'
  // 老板 = 红方，伙计 = 黄三角（带细墨线），茶客 = 蓝圆，空座 = 墨线圆
  let { role = null, size = 14 }: { role?: Role | null; size?: number } = $props()
</script>

<svg width={size} height={size} viewBox="0 0 20 20" aria-hidden="true">
  {#if role === 'boss'}
    <rect x="1" y="1" width="18" height="18" fill="var(--red)" />
  {:else if role === 'heir'}
    <!-- 黄色本身在米白底上对比度不足：加一圈细墨线（任何尺寸都是 1px），颜色不变 -->
    <polygon points="10,1.6 19,18.6 1,18.6" fill="var(--yellow)" stroke="var(--ink)" stroke-width="1" stroke-linejoin="miter" vector-effect="non-scaling-stroke" />
  {:else if role === 'guest'}
    <circle cx="10" cy="10" r="9" fill="var(--blue)" />
  {:else}
    <circle cx="10" cy="10" r="8.5" fill="none" stroke="var(--ink)" stroke-width="1.2" />
  {/if}
</svg>

<style>
  svg { display: block; flex: none; }
</style>
