<script lang="ts">
  // 成果预览：《车位分布图（实测）》——42 个车位，红色为外来车辆长期占用
  const taken = new Set([3, 4, 11, 17, 22, 30, 31, 38])
  const rows = 3, cols = 14
  const w = 26, h = 40, gap = 4
  const slots = Array.from({ length: rows * cols }, (_, i) => ({
    i, x: (i % cols) * (w + gap), y: Math.floor(i / cols) * (h + 34),
  }))
  const W = cols * (w + gap) - gap
  const H = (rows - 1) * (h + 34) + h + 16
</script>

<svg viewBox="-2 -2 {W + 4} {H + 4}" width="100%" aria-label="车位分布图">
  {#each slots as s}
    <rect x={s.x} y={s.y} width={w} height={h}
      fill={taken.has(s.i) ? 'var(--red)' : 'none'}
      stroke={taken.has(s.i) ? 'var(--red)' : 'var(--ink)'} stroke-width="1.5" />
    <text x={s.x + w / 2} y={s.y + h + 13} text-anchor="middle">{String(s.i + 1).padStart(2, '0')}</text>
  {/each}
</svg>

<style>
  text { font-family: var(--latin); font-size: 9px; fill: var(--ink-3); }
</style>
