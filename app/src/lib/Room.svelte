<script lang="ts">
  import Shape from './Shape.svelte'
  import { L, t } from '../i18n/index.svelte'
  import type { House, Member } from '../data'

  // 房间按容器宽度缩放（宽高相等），座位、字号都用容器单位 cqi。
  let { house }: { house: House } = $props()

  // 5×5 网格的外圈共 16 格；底边正中留作门，剩下 15 格就是 15 个座位。
  // 从上边正中（主位）开始顺时针排座。
  const ring: [number, number][] = [
    [0, 2], [0, 3], [0, 4], [1, 4], [2, 4], [3, 4], [4, 4], [4, 3],
    [4, 1], [4, 0], [3, 0], [2, 0], [1, 0], [0, 0], [0, 1],
  ]
  const rank = (m: Member) => (m.role === 'boss' ? 0 : m.role === 'heir' ? 1 : 2)
  let seats = $derived.by(() => {
    const order = [...house.members].sort((a, b) => rank(a) - rank(b))
    return ring.slice(0, house.capacity).map(([r, c], i) => ({ r, c, m: order[i] ?? null, i }))
  })
</script>

<div class="room" role="img" aria-label="{house.members.length}/{house.capacity} {t('room.label')}">
  <div class="box">
    <!-- 墙：四段，门开在底边正中 -->
    <i class="wall top"></i>
    <i class="wall left"></i>
    <i class="wall right"></i>
    <i class="wall bottom-l"></i>
    <i class="wall bottom-r"></i>
    <span class="door label">{t('room.door')}</span>

    <!-- 茶桌 -->
    <div class="table">
      <div class="count num"><b>{house.members.length}</b><span>/{house.capacity}</span></div>
      <div class="label">{t('room.label')}</div>
    </div>

    {#each seats as s}
      <div
        class="seat"
        class:empty={!s.m}
        style:left="{s.c * 20}%"
        style:top="{s.r * 20}%"
        style:animation-delay="{120 + s.i * 45}ms"
      >
        <span class="glyph"><Shape role={s.m?.role ?? null} size={20} /></span>
        <span class="name">{s.m ? L(s.m.name) : t('role.empty')}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .room { width: 100%; container-type: inline-size; }
  .box { position: relative; width: 100%; aspect-ratio: 1; --wall: max(5px, 1.5cqi); }

  .wall { position: absolute; background: var(--ink); }
  .top { left: 0; right: 0; top: 0; height: var(--wall); }
  .left { left: 0; top: 0; bottom: 0; width: var(--wall); }
  .right { right: 0; top: 0; bottom: 0; width: var(--wall); }
  .bottom-l { left: 0; bottom: 0; height: var(--wall); width: 40%; }
  .bottom-r { right: 0; bottom: 0; height: var(--wall); width: 40%; }
  .door { position: absolute; top: calc(100% + 10px); left: 50%; transform: translateX(-50%); letter-spacing: 0.3em; }

  .table {
    position: absolute;
    left: 25%;
    top: 25%;
    width: 50%;
    height: 50%;
    border: 2px solid var(--ink);
    border-radius: 50%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
  }
  .count { line-height: 1; display: flex; align-items: baseline; }
  .count b { font-size: 16cqi; font-weight: 300; letter-spacing: -0.03em; }
  .count span { font-size: 5cqi; font-weight: 300; color: var(--ink-3); margin-left: 0.15em; }

  .seat {
    position: absolute;
    width: 20%;
    height: 20%;
    padding: 0 3px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1.8cqi;
    animation: sit 520ms var(--ease) both;
  }
  .glyph :global(svg) { width: 6.8cqi; height: 6.8cqi; transition: transform 240ms var(--ease); }
  .empty .glyph :global(svg) { width: 4.2cqi; height: 4.2cqi; }
  .name {
    font-size: clamp(11px, 2.4cqi, 13px);
    font-weight: 500;
    line-height: 1.15;
    text-align: center;
    max-width: 100%;
    hyphens: none;
  }
  .empty .name { color: var(--ink-3); font-weight: 400; }
  .seat:not(.empty):hover :global(svg) { transform: scale(1.12); }

  /* 入座：从无到有，只动一次 */
  @keyframes sit {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: none; }
  }
</style>
