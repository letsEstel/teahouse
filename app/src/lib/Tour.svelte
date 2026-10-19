<script lang="ts">
  import { self, type House } from '../api/view'
  import { t } from '../i18n/index.svelte'

  // 第一次进入一间茶馆时的 3 步引导：座位 → 问题 → 你能做什么。
  // ?tour=off 关闭；?tour=1|2|3 强制显示某一步（截图用）。
  let { house }: { house: House } = $props()
  // 组件按茶馆 {#key} 重建，所以这里只读一次 house 是对的
  const key = (() => `teahouse.tour.${house.id}`)()
  const q = new URLSearchParams(location.search).get('tour')

  function seen() {
    try { return localStorage.getItem(key) === 'done' } catch { return false }
  }
  let step = $state(q === 'off' ? 0 : q && +q >= 1 && +q <= 3 ? +q : seen() ? 0 : 1)

  const targets = ['', '[data-tour=room]', '[data-tour=nav-problems]', '[data-tour=action]']
  let rect = $state<DOMRect | null>(null)
  let cardH = $state(0)
  let vw = $state(innerWidth)
  let vh = $state(innerHeight)

  function measure() {
    vw = innerWidth; vh = innerHeight
    const el = step ? document.querySelector(targets[step]) : null
    rect = el ? el.getBoundingClientRect() : null
  }
  $effect(() => {
    if (!step) return
    const el = document.querySelector(targets[step])
    el?.scrollIntoView({ block: 'center' })
    // 等入座动画和字体落定
    const id = setTimeout(measure, 60)
    addEventListener('resize', measure)
    addEventListener('scroll', measure, true)
    return () => { clearTimeout(id); removeEventListener('resize', measure); removeEventListener('scroll', measure, true) }
  })

  function close() {
    step = 0
    try { localStorage.setItem(key, 'done') } catch {}
  }

  const gap = 20, edge = 16
  let w = $derived(Math.min(380, vw - edge * 2))
  let pos = $derived.by(() => {
    if (!rect) return { left: edge, top: edge }
    const h = cardH || 220
    const clampTop = (y: number) => Math.max(edge, Math.min(y, vh - h - edge))
    const clampLeft = (x: number) => Math.max(edge, Math.min(x, vw - w - edge))
    if (rect.right + gap + w <= vw - edge) return { left: rect.right + gap, top: clampTop(rect.top) }
    if (rect.left - gap - w >= edge) return { left: rect.left - gap - w, top: clampTop(rect.top) }
    if (rect.bottom + gap + h <= vh - edge) return { left: clampLeft(rect.left), top: rect.bottom + gap }
    if (rect.top - gap - h >= edge) return { left: clampLeft(rect.left), top: rect.top - gap - h }
    return { left: clampLeft(rect.left), top: vh - h - edge }
  })
  let body = $derived(step === 3 ? t(`tour.3.${self(house).role}`) : t(`tour.${step}.body`))
</script>

{#if step && rect}
  <div class="veil" aria-hidden="true"></div>
  <div class="spot" style:left="{rect.left - 8}px" style:top="{rect.top - 8}px" style:width="{rect.width + 16}px" style:height="{rect.height + 16}px"></div>
  <div class="card" role="dialog" aria-modal="true" aria-labelledby="tour-title" bind:clientHeight={cardH}
    style:left="{pos.left}px" style:top="{pos.top}px" style:width="{w}px">
    <div class="steps" aria-label={t('tour.step', { i: step })}>
      {#each [1, 2, 3] as i}<i class:on={i <= step}></i>{/each}
      <span class="label">{t('tour.step', { i: step })}</span>
    </div>
    <h2 id="tour-title">{t(`tour.${step}.title`)}</h2>
    <p>{body}</p>
    <div class="row">
      <button class="skip" onclick={close}>{t('tour.skip')}</button>
      <button class="next" onclick={() => (step === 3 ? close() : step++)}>{step === 3 ? t('tour.done') : t('tour.next')}</button>
    </div>
  </div>
{/if}

<style>
  .veil { position: fixed; inset: 0; z-index: 20; }
  /* 聚光：一圈很宽的米白描边盖住其余部分，目标处保持清楚 */
  .spot {
    position: fixed;
    z-index: 21;
    border: 3px solid var(--ink);
    outline: 200vmax solid color-mix(in srgb, var(--paper) 78%, transparent);
    pointer-events: none;
    transition: all 360ms var(--ease);
  }
  .card {
    position: fixed;
    z-index: 22;
    background: var(--paper);
    border: 2px solid var(--ink);
    padding: 22px 24px 20px;
    animation: in 360ms var(--ease) both;
  }
  .steps { display: flex; align-items: center; gap: 6px; }
  .steps i { width: 22px; height: 6px; border: 1.5px solid var(--ink); }
  .steps i.on { background: var(--ink); }
  .steps .label { margin-left: 10px; }
  h2 { font-family: var(--display); font-weight: var(--heavy); font-size: 24px; line-height: 1.25; margin: 16px 0 8px; letter-spacing: var(--track); }
  p { font-size: 15px; line-height: 1.65; }
  .row { display: flex; justify-content: space-between; align-items: center; margin-top: 20px; gap: 12px; }
  .skip { font-size: 13px; font-weight: 500; color: var(--ink-2); border-bottom: 1px solid var(--ink-3); }
  .next { background: var(--ink); color: var(--paper); font-weight: 700; padding: 10px 22px; border: 2px solid var(--ink); letter-spacing: var(--track-wide); }
  .next:hover { background: var(--paper); color: var(--ink); }
  @keyframes in { from { opacity: 0; transform: translateY(6px); } to { opacity: 1; transform: none; } }
</style>
