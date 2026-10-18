<script lang="ts">
  import { t } from '../i18n/index.svelte'
  // 成果预览：《车位分布图（实测）》——42 个车位，实心为外来车辆长期占用。
  // 内容里的填充一律附图例，不借用角色色。
  const taken = new Set([3, 4, 11, 17, 22, 30, 31, 38])
  const rows = 3, cols = 14
  const w = 26, h = 40, gap = 4
  const slots = Array.from({ length: rows * cols }, (_, i) => ({
    i, x: (i % cols) * (w + gap), y: Math.floor(i / cols) * (h + 34),
  }))
  const W = cols * (w + gap) - gap
  const H = (rows - 1) * (h + 34) + h + 16
</script>

<figure>
  <svg viewBox="-2 -2 {W + 4} {H + 4}" width="100%" role="img" aria-label={t('parking.label')}>
    {#each slots as s}
      <rect x={s.x} y={s.y} width={w} height={h}
        fill={taken.has(s.i) ? 'var(--ink)' : 'none'}
        stroke="var(--ink)" stroke-width="1.5" />
      <text x={s.x + w / 2} y={s.y + h + 13} text-anchor="middle">{String(s.i + 1).padStart(2, '0')}</text>
    {/each}
  </svg>
  <figcaption>
    <span><i class="on"></i>{t('parking.taken')} <b class="num">{taken.size}</b></span>
    <span><i></i>{t('parking.other')} <b class="num">{rows * cols - taken.size}</b></span>
  </figcaption>
</figure>

<style>
  text { font-family: var(--latin); font-size: 9px; fill: var(--ink-3); }
  figcaption { display: flex; flex-wrap: wrap; gap: 8px 28px; margin-top: 14px; font-size: 13px; font-weight: 500; }
  figcaption span { display: inline-flex; align-items: center; gap: 8px; }
  figcaption i { width: 10px; height: 16px; border: 1.5px solid var(--ink); }
  figcaption i.on { background: var(--ink); }
  figcaption b { font-weight: 400; color: var(--ink-3); }
</style>
