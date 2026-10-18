<script lang="ts">
  import Shape from './Shape.svelte'
  import { house, members, proposals, problems, byId, me, roleName } from '../data/fake'

  let { section }: { section: 'house' | 'problems' | 'proposals' } = $props()
  const self = byId(me)

  const nav = [
    { key: 'house', no: '01', name: '座位', href: '#/house', count: `${members.length}/${house.capacity}` },
    { key: 'problems', no: '02', name: '问题', href: '#/problems', count: String(problems.length).padStart(2, '0') },
    { key: 'proposals', no: '03', name: '提案', href: '#/proposal/p1', count: String(proposals.length).padStart(2, '0'), alert: true },
  ] as const
</script>

<aside>
  <a class="mark" href="#/house" aria-label="茶馆">
    <svg width="44" height="44" viewBox="0 0 44 44">
      <rect x="1.5" y="1.5" width="41" height="41" fill="none" stroke="var(--ink)" stroke-width="3" />
      <circle cx="22" cy="22" r="10" fill="var(--ink)" />
      <rect x="17" y="39" width="10" height="5" fill="var(--paper)" />
    </svg>
    <div>
      <div class="word">茶馆</div>
      <div class="label">Teahouse</div>
    </div>
  </a>

  <div class="house">
    <div class="label">当前茶馆</div>
    <div class="house-name">{house.name}</div>
  </div>

  <nav>
    {#each nav as n}
      <a href={n.href} class:on={section === n.key}>
        <span class="no num">{n.no}</span>
        <span class="nm">{n.name}</span>
        <span class="ct num" class:alert={'alert' in n}>{n.count}</span>
      </a>
    {/each}
  </nav>

  <div class="self">
    <Shape role={self.role} size={16} />
    <div>
      <div class="self-name">{self.name}</div>
      <div class="label">你 · {roleName[self.role]}</div>
    </div>
  </div>
</aside>

<style>
  aside {
    position: fixed;
    inset: 0 auto 0 0;
    width: var(--rail);
    border-right: 2px solid var(--ink);
    display: flex;
    flex-direction: column;
    padding: 40px 0 36px;
    background: var(--paper);
    z-index: 2;
  }
  .mark { display: flex; gap: 14px; align-items: center; padding: 0 32px; }
  .word { font-family: var(--han); font-weight: 900; font-size: 22px; line-height: 1.1; letter-spacing: 0.08em; }
  .mark .label { font-size: 10px; margin-top: 3px; }

  .house { margin: 64px 0 28px; padding: 0 32px; }
  .house-name { font-family: var(--han); font-weight: 700; font-size: 18px; margin-top: 6px; line-height: 1.3; }

  nav { border-top: 2px solid var(--ink); }
  nav a {
    display: grid;
    grid-template-columns: 36px 1fr auto;
    align-items: baseline;
    padding: 18px 32px 18px 32px;
    border-bottom: 1px solid var(--ink);
    position: relative;
    transition: background 200ms var(--ease);
  }
  nav a:hover { background: var(--paper-deep); }
  nav a.on { background: var(--ink); color: var(--paper); }
  .no { font-size: 12px; font-weight: 500; color: var(--ink-3); }
  .on .no { color: var(--paper); opacity: 0.6; }
  .nm { font-family: var(--han); font-size: 17px; font-weight: 500; letter-spacing: 0.1em; }
  .on .nm { font-weight: 700; }
  .ct { font-size: 13px; color: var(--ink-2); }
  .on .ct { color: var(--paper); }
  .ct.alert { background: var(--red); color: var(--paper); padding: 0 7px; font-weight: 500; }

  .self { margin-top: auto; padding: 0 32px; display: flex; align-items: center; gap: 14px; }
  .self-name { font-family: var(--han); font-weight: 700; font-size: 15px; line-height: 1.2; }
  .self .label { font-size: 10px; }
</style>
