<script lang="ts">
  import Shape from './Shape.svelte'
  import { self, pad, type House } from '../api/view'
  import { i18n, langs, setLang, t, type Lang } from '../i18n/index.svelte'
  import { theme, themes, setTheme } from './theme.svelte'
  import { api } from '../api/backend'
  import type { HouseSummary, ArchiveNode } from '../api/types'

  // ≥1024：左侧竖栏；<1024：顶栏，茶馆列表收进下拉。
  let { house, houses, section, seated = true }: { house: House; houses: HouseSummary[]; seated?: boolean; section: 'house' | 'board' | 'problems' | 'proposals' | 'archive' } = $props()
  let me = $derived(self(house))
  let open = $state(false)
  // 档案里的文件数（只读本地，很快）
  let files = $state(0)
  const leaves = (n: ArchiveNode): number => (n.children ? n.children.reduce((a, c) => a + leaves(c), 0) : 1)
  $effect(() => {
    api('archive', { house: house.id, view: 'dirs' }).then((root) => (files = leaves(root)))
  })
  // 茶馆超过 4 间时，宽屏侧栏默认只列出当前这间，点"全部"展开（展开后列表可滚动）
  const MANY = 4
  let many = $derived(houses.length > MANY)
  let expanded = $state(false)

  let nav = $derived([
    { key: 'house', no: '01', name: t('nav.seats'), href: `#/${house.id}/house`, count: `${house.members.length}/${house.capacity}`, badge: false },
    { key: 'board', no: '02', name: t('nav.board'), href: `#/${house.id}/board`, count: pad(house.board.length), badge: false },
    { key: 'problems', no: '03', name: t('nav.problems'), href: `#/${house.id}/problems`, count: pad(house.problems.length), badge: false },
    { key: 'proposals', no: '04', name: t('nav.proposals'), href: `#/${house.id}/proposals`, count: pad(house.proposals.length), badge: house.proposals.length > 0 },
    { key: 'archive', no: '05', name: t('nav.archive'), href: `#/${house.id}/archive`, count: pad(files), badge: false },
  ])
  const short: Record<Lang, string> = { zh: '中', en: 'EN', de: 'DE' }
  const full: Record<Lang, string> = { zh: '中文', en: 'English', de: 'Deutsch' }
</script>

<aside>
  <a class="mark" href="#/{house.id}/house" aria-label={t('app.name')}>
    <svg width="44" height="44" viewBox="0 0 44 44" aria-hidden="true">
      <rect x="1.5" y="1.5" width="41" height="41" fill="none" stroke="var(--ink)" stroke-width="3" />
      <circle cx="22" cy="22" r="10" fill="var(--ink)" />
      <rect x="17" y="39" width="10" height="5" fill="var(--paper)" />
    </svg>
    <div class="mark-text">
      <div class="word">{t('app.name')}</div>
      <div class="label">{t('app.sub')}</div>
    </div>
  </a>

  <div class="houses">
    <div class="label head">{t('rail.houses')}</div>
    <button class="switch" aria-expanded={open} aria-label={t('rail.switch')} onclick={() => (open = !open)}>
      <span>{house.name}</span><i class:up={open}></i>
    </button>
    <ul class:open class:folded={many && !expanded} class:scroll={many && expanded}>
      {#each houses as h}
        <li class:current={h.id === house.id}>
          <a href="#/{h.id}/house" class:on={h.id === house.id} onclick={() => { open = false; expanded = false }}>
            <i></i><span>{h.name}</span>
          </a>
        </li>
        {#if h.id === house.id}
          <!-- 当前茶馆的同步状态，紧跟在名字下面 -->
          <li class="current syncli">
            <p class="syncline {house.sync.state}" title={house.sync.message ?? ''}>
              <i></i>{house.sync.state === 'ahead' ? t('sync.short.ahead', { n: house.sync.unpushed }) : t(`sync.short.${house.sync.state}`)}
            </p>
          </li>
        {/if}
      {/each}
      <li class="join"><a href="#/join" onclick={() => (open = false)}>+ {t('rail.join')}</a></li>
    </ul>
    {#if many}
      <button class="more" aria-expanded={expanded} onclick={() => (expanded = !expanded)}>
        {expanded ? t('rail.collapse') : t('rail.showAll', { n: houses.length })}<i class:up={expanded}></i>
      </button>
    {/if}
  </div>

  <nav aria-label={t('nav.label')}>
    {#each nav as n}
      <a href={n.href} class:on={section === n.key} data-tour={n.key === 'problems' ? 'nav-problems' : undefined}>
        <span class="no num">{n.no}</span>
        <span class="nm">{n.name}</span>
        {#if n.badge}<em class="badge">{n.count}</em>{:else}<span class="ct num">{n.count}</span>{/if}
      </a>
    {/each}
  </nav>

  <div class="foot">
    <div class="self">
      {#if seated}
        <Shape role={me.role} size={16} />
        <div>
          <div class="self-name">{me.name}</div>
          <div class="label">{t('self.you', { role: t(`role.${me.role}`) })}</div>
        </div>
      {:else}
        <Shape role={null} size={16} />
        <div>
          <div class="self-name">{t('seat.readonly')}</div>
          <div class="label">{t('seat.leftShort')}</div>
        </div>
      {/if}
    </div>
    <div class="switches">
      <div class="langs" role="group" aria-label={t('rail.lang')}>
        {#each langs as l}
          <button class:on={i18n.lang === l} aria-pressed={i18n.lang === l} title={full[l]} lang={l} onclick={() => setLang(l)}>{short[l]}</button>
        {/each}
      </div>
      <div class="langs themes" role="group" aria-label={t('theme.label')}>
        {#each themes as m}
          <button class:on={theme.mode === m} aria-pressed={theme.mode === m} title={t(`theme.${m}`)} aria-label={t(`theme.${m}`)} onclick={() => setTheme(m)}>
            <i class="glyph {m}"></i>
          </button>
        {/each}
      </div>
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
    padding: 36px 0 28px;
    background: var(--paper);
    z-index: 5;
    overflow-y: auto;
    --in: clamp(20px, 2.2vw, 32px);
  }
  .mark { display: flex; gap: 14px; align-items: center; padding: 0 var(--in); }
  .mark svg { flex: none; }
  .word { font-family: var(--display); font-weight: var(--heavy); font-size: 22px; line-height: 1.1; letter-spacing: var(--track-wide); white-space: nowrap; }
  .mark .label { font-size: 10px; margin-top: 3px; }

  .houses { margin: 48px 0 24px; padding: 0 var(--in); position: relative; }
  .head { margin-bottom: 8px; }
  /* 同步状态：一个小几何记号 + 一句话，只用墨色 */
  .syncline { display: flex; align-items: center; gap: 8px; margin: 0 0 6px 19px; font-size: 12px; color: var(--ink-2); }
  .syncline i { flex: none; width: 9px; height: 9px; border: 1.5px solid var(--ink); border-radius: 50%; position: relative; overflow: hidden; }
  .syncline.synced i { background: var(--ink); }
  .syncline.ahead i::before { content: ''; position: absolute; left: 0; top: 0; bottom: 0; width: 50%; background: var(--ink); }
  .syncline.behind i::before { content: ''; position: absolute; right: 0; top: 0; bottom: 0; width: 50%; background: var(--ink); }
  .syncline.error i { border-radius: 0; background: var(--ink); }
  .syncline:not(.synced) { color: var(--ink); font-weight: 700; }
  .switch { display: none; }
  ul { list-style: none; }
  ul a { display: flex; align-items: baseline; gap: 10px; padding: 5px 0; font-size: 14px; color: var(--ink-2); line-height: 1.35; }
  ul a i { flex: none; width: 9px; height: 9px; border: 1.5px solid var(--ink); transform: translateY(-1px); }
  ul a:hover { color: var(--ink); }
  ul a.on { color: var(--ink); font-family: var(--display); font-weight: 700; font-size: 17px; }
  ul a.on i { background: var(--ink); }
  ul.folded li:not(.current):not(.join) { display: none; }
  .join a { font-size: 12px; font-weight: 500; padding-top: 8px; }
  ul.scroll { max-height: 34vh; overflow-y: auto; border-bottom: 1px solid var(--rule); padding-bottom: 4px; }
  .more { display: inline-flex; align-items: center; gap: 8px; margin-top: 8px; font-size: 12px; font-weight: 500; color: var(--ink-2); border-bottom: 1px solid var(--ink-3); }
  .more:hover { color: var(--ink); }
  .more i { width: 0; height: 0; border-left: 4px solid transparent; border-right: 4px solid transparent; border-top: 5px solid currentColor; transition: transform 200ms var(--ease); }
  .more i.up { transform: rotate(180deg); }

  nav { border-top: 2px solid var(--ink); }
  nav a {
    display: grid;
    grid-template-columns: 26px minmax(0, 1fr) auto;
    align-items: baseline;
    gap: 6px;
    padding: 16px var(--in);
    border-bottom: 1px solid var(--ink);
    transition: background 200ms var(--ease);
  }
  nav a:hover { background: var(--paper-deep); }
  nav a.on { background: var(--ink); color: var(--paper); }
  .no { font-size: 12px; font-weight: 500; color: var(--ink-3); }
  .on .no { color: var(--paper); }
  .nm { font-family: var(--display); font-size: 16px; font-weight: 500; letter-spacing: var(--track-wide); white-space: nowrap; }
  .on .nm { font-weight: 700; }
  .ct { font-size: 13px; color: var(--ink-2); }
  .on .ct { color: var(--paper); }
  .on .badge { background: var(--paper); color: var(--ink); }

  .foot { margin-top: auto; padding: 28px var(--in) 0; display: flex; flex-direction: column; gap: 20px; }
  .self { display: flex; align-items: center; gap: 14px; }
  .self-name { font-family: var(--display); font-weight: 700; font-size: 15px; line-height: 1.2; }
  .self .label { font-size: 10px; }
  .langs { display: flex; }
  .langs button {
    font-family: var(--latin); font-size: 12px; font-weight: 500; letter-spacing: 0.08em;
    min-width: 40px; height: 30px; padding: 0 8px; border: 1.5px solid var(--ink); margin-right: -1.5px;
    text-align: center;
  }
  .langs button[lang='zh'] { font-family: 'Noto Sans SC', sans-serif; }
  .langs button.on { background: var(--ink); color: var(--paper); }
  .switches { display: flex; flex-wrap: wrap; gap: 8px; }
  .themes button { min-width: 34px; display: grid; place-items: center; }
  /* 自动 = 半满圆，浅 = 空心圆，深 = 实心圆 */
  .glyph { display: block; width: 12px; height: 12px; border-radius: 50%; border: 1.5px solid currentColor; }
  .glyph.dark { background: currentColor; }
  .glyph.auto { overflow: hidden; }
  .glyph.auto::before { content: ''; display: block; width: 50%; height: 100%; background: currentColor; }

  /* 窄窗口：侧栏变成顶栏 */
  @media (max-width: 1023.98px) {
    aside {
      position: sticky;
      top: 0;
      width: auto;
      inset: auto;
      border-right: 0;
      border-bottom: 2px solid var(--ink);
      padding: 0;
      overflow: visible;
      display: grid;
      grid-template-columns: auto minmax(0, 1fr) auto;
      grid-template-areas: 'mark houses foot' 'nav nav nav';
      align-items: center;
      --in: 20px;
    }
    .mark { grid-area: mark; padding: 12px var(--in); gap: 10px; }
    .mark svg { width: 32px; height: 32px; }
    .mark .label { display: none; }
    .word { font-size: 18px; }

    .houses { grid-area: houses; margin: 0; padding: 0 0 0 var(--in); border-left: 1px solid var(--ink); align-self: stretch; display: flex; }
    .head { display: none; }
    .syncli { display: none !important; }
    .switch {
      display: flex; align-items: center; gap: 10px; width: 100%; min-width: 0;
      font-family: var(--display); font-weight: 700; font-size: 16px;
    }
    .switch span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
    .switch i { flex: none; width: 0; height: 0; border-left: 6px solid transparent; border-right: 6px solid transparent; border-top: 8px solid var(--ink); transition: transform 200ms var(--ease); }
    .switch i.up { transform: rotate(180deg); }
    ul {
      display: none;
      position: absolute; top: 100%; left: -1px; min-width: 280px;
      background: var(--paper); border: 2px solid var(--ink); padding: 10px var(--in); z-index: 6;
    }
    ul.open { display: block; max-height: min(60vh, 420px); overflow-y: auto; }
    /* 窄屏下拉里总是列出全部茶馆 */
    ul.folded li:not(.current) { display: list-item; }
    ul.scroll { border-bottom: 2px solid var(--ink); }
    .more { display: none; }
    .join a { border-top: 1px solid var(--rule); margin-top: 4px; padding-top: 10px; }
    ul a { padding: 8px 0; }

    nav { grid-area: nav; display: grid; grid-template-columns: repeat(5, minmax(0, 1fr)); border-top: 1px solid var(--ink); }
    nav a { grid-template-columns: minmax(0, 1fr) auto; border-bottom: 0; padding: 12px var(--in); }
    nav a + a { border-left: 1px solid var(--ink); }
    .no { display: none; }
    .nm { font-size: 15px; }

    .foot { grid-area: foot; margin: 0; padding: 0 var(--in); flex-direction: row; align-items: center; gap: 16px; }
    .self .label { display: none; }
    .self { gap: 8px; }
    .self-name { font-size: 14px; }
    .switches { flex-wrap: nowrap; }
  }
  /* 手机：三行，不吸顶（屏幕太矮）。第一行标志 + 茶馆切换，第二行你是谁 + 语言/外观，第三行四个栏目 */
  @media (max-width: 639.98px) {
    aside {
      position: static;
      grid-template-columns: auto minmax(0, 1fr);
      grid-template-areas: 'mark houses' 'foot foot' 'nav nav';
    }
    .word { white-space: nowrap; }
    .switch { padding: 8px 0; }
    .switch span { white-space: normal; line-height: 1.25; }
    .foot { justify-content: space-between; padding: 8px var(--in); border-top: 1px solid var(--ink); }
    .self-name { display: block; }
    .switches { gap: 6px; }
    .langs button { min-width: 34px; height: 28px; padding: 0 6px; }
    .themes button { min-width: 30px; }
    nav a { padding: 12px 10px; justify-content: center; grid-template-columns: auto; gap: 2px; text-align: center; }
    .ct { display: none; }
    nav a .badge { justify-self: center; font-size: 11px; }
    .nm { font-size: 14px; letter-spacing: 0; }
    ul { min-width: min(280px, calc(100vw - 32px)); }
  }
</style>
