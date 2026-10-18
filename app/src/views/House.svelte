<script lang="ts">
  import Room from '../lib/Room.svelte'
  import Shape from '../lib/Shape.svelte'
  import { house, members, problems, proposals, recent, byId } from '../data/fake'

  const works = problems.reduce((n, p) => n + p.works.length, 0)
  const count = (r: string) => members.filter((m) => m.role === r).length
  const pad = (n: number) => String(n).padStart(2, '0')
</script>

<div class="wrap">
  <section class="left">
    <div class="label">茶馆 · 开张于 {house.founded}</div>
    <h1>梧桐里<br />三号院</h1>
    <i class="bar"></i>
    <p class="motto">{house.motto}</p>

    <div class="stats">
      <a href="#/problems"><b class="num">{pad(problems.length)}</b><span>问题</span></a>
      <a href="#/problem/0001"><b class="num">{works}</b><span>成果</span></a>
      <a href="#/proposal/p1" class="hot"><b class="num">{pad(proposals.length)}</b><span>提案待审</span></a>
    </div>

    <div class="recent">
      <div class="label head">最近</div>
      <ul>
        {#each recent as e}
          {@const m = byId(e.by)}
          <li>
            <Shape role={m.role} size={10} />
            <span class="who">{m.name}</span>
            <span class="what">{e.text}</span>
            <span class="when">{e.when}</span>
          </li>
        {/each}
      </ul>
    </div>
  </section>

  <section class="right">
    <Room size={540} />
    <div class="legend">
      <span><Shape role="boss" size={12} />老板 <em class="num">{count('boss')}</em></span>
      <span><Shape role="heir" size={12} />伙计 · 继承人 <em class="num">{count('heir')}</em></span>
      <span><Shape role="guest" size={12} />茶客 <em class="num">{count('guest')}</em></span>
      <span><Shape role={null} size={12} />空座 <em class="num">{house.capacity - members.length}</em></span>
    </div>
  </section>
</div>

<style>
  .wrap {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 540px;
    gap: 72px;
    padding: 64px 72px 56px 64px;
    min-height: 100vh;
  }
  .left { display: flex; flex-direction: column; }

  h1 {
    font-family: var(--han);
    font-weight: 900;
    font-size: 84px;
    line-height: 1.04;
    letter-spacing: 0.02em;
    margin-top: 22px;
  }
  .bar { display: block; width: 72px; height: 8px; background: var(--red); margin: 30px 0 18px; }
  .motto { font-size: 18px; color: var(--ink-2); letter-spacing: 0.06em; }

  .stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    margin-top: 48px;
    border-top: 2px solid var(--ink);
  }
  .stats a { padding: 16px 0 0; display: flex; flex-direction: column; }
  .stats a + a { padding-left: 20px; border-left: 1px solid var(--ink); }
  .stats b { font-size: 56px; font-weight: 300; line-height: 1; letter-spacing: -0.02em; }
  .stats span { font-size: 14px; margin-top: 8px; font-weight: 500; }
  .stats .hot b { color: var(--red); }

  .recent { margin-top: auto; padding-top: 40px; }
  .head { padding-bottom: 10px; border-bottom: 1px solid var(--ink); }
  ul { list-style: none; }
  li {
    display: grid;
    grid-template-columns: 10px 64px 1fr auto;
    gap: 12px;
    align-items: center;
    padding: 10px 0;
    border-bottom: 1px solid color-mix(in srgb, var(--ink) 18%, transparent);
    font-size: 14px;
  }
  .who { font-weight: 700; }
  .what { color: var(--ink-2); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .when { font-family: var(--latin); font-size: 12px; color: var(--ink-3); }

  .right { display: flex; flex-direction: column; align-items: flex-start; padding-top: 8px; }
  .legend {
    display: flex;
    gap: 28px;
    margin-top: 56px;
    font-size: 13px;
    font-weight: 500;
  }
  .legend span { display: inline-flex; align-items: center; gap: 8px; }
  .legend em { font-style: normal; color: var(--ink-3); }
</style>
