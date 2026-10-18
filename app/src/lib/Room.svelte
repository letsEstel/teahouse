<script lang="ts">
  import Shape from './Shape.svelte'
  import { members, house, type Member } from '../data/fake'

  let { size = 560 }: { size?: number } = $props()

  // 5×5 网格的外圈共 16 格；底边正中留作门，剩下 15 格就是 15 个座位。
  // 从上边正中（主位）开始顺时针排座。
  const ring: [number, number][] = [
    [0, 2], [0, 3], [0, 4], [1, 4], [2, 4], [3, 4], [4, 4], [4, 3],
    [4, 1], [4, 0], [3, 0], [2, 0], [1, 0], [0, 0], [0, 1],
  ]
  const order = [...members].sort((a, b) => rank(a) - rank(b))
  function rank(m: Member) { return m.role === 'boss' ? 0 : m.role === 'heir' ? 1 : 2 }

  const seats = ring.slice(0, house.capacity).map(([r, c], i) => ({ r, c, m: order[i] ?? null, i }))

  const wall = 8
  let cell = $derived(size / 5)
  let glyph = $derived(Math.round(cell * 0.34))
</script>

<div class="room" style:width="{size}px" style:height="{size}px" style:--cell="{cell}px" style:--wall="{wall}px">
  <!-- 墙：四段，门开在底边正中 -->
  <i class="wall top"></i>
  <i class="wall left"></i>
  <i class="wall right"></i>
  <i class="wall bottom-l"></i>
  <i class="wall bottom-r"></i>
  <span class="door label">门</span>

  <!-- 茶桌 -->
  <div class="table">
    <div class="count num"><b>{members.length}</b><span>/{house.capacity}</span></div>
    <div class="label">座位</div>
  </div>

  {#each seats as s}
    <div
      class="seat"
      class:empty={!s.m}
      style:left="{s.c * cell}px"
      style:top="{s.r * cell}px"
      style:animation-delay="{120 + s.i * 45}ms"
    >
      <Shape role={s.m?.role ?? null} size={s.m ? glyph : glyph * 0.62} />
      <span class="name">{s.m ? s.m.name : '空座'}</span>
    </div>
  {/each}
</div>

<style>
  .room { position: relative; flex: none; }

  .wall { position: absolute; background: var(--ink); }
  .top { left: 0; right: 0; top: 0; height: var(--wall); }
  .left { left: 0; top: 0; bottom: 0; width: var(--wall); }
  .right { right: 0; top: 0; bottom: 0; width: var(--wall); }
  .bottom-l { left: 0; bottom: 0; height: var(--wall); width: calc(var(--cell) * 2); }
  .bottom-r { right: 0; bottom: 0; height: var(--wall); width: calc(var(--cell) * 2); }
  .door { position: absolute; bottom: -26px; left: 50%; transform: translateX(-50%); letter-spacing: 0.3em; }

  .table {
    position: absolute;
    left: calc(var(--cell) * 1.25);
    top: calc(var(--cell) * 1.25);
    width: calc(var(--cell) * 2.5);
    height: calc(var(--cell) * 2.5);
    border: 2px solid var(--ink);
    border-radius: 50%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
  }
  .count { line-height: 1; display: flex; align-items: baseline; }
  .count b { font-size: 88px; font-weight: 300; letter-spacing: -0.03em; }
  .count span { font-size: 28px; font-weight: 300; color: var(--ink-3); margin-left: 4px; }

  .seat {
    position: absolute;
    width: var(--cell);
    height: var(--cell);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    animation: sit 520ms var(--ease) both;
  }
  .name { font-size: 13px; font-weight: 500; line-height: 1; }
  .empty .name { color: var(--ink-3); font-weight: 400; }
  .seat :global(svg) { transition: transform 240ms var(--ease); }
  .seat:not(.empty):hover :global(svg) { transform: scale(1.12); }

  /* 入座：从无到有，只动一次 */
  @keyframes sit {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: none; }
  }
</style>
