<script lang="ts">
  import Room from '../lib/Room.svelte'
  import Shape from '../lib/Shape.svelte'
  import AiMark from '../lib/AiMark.svelte'
  import { fit } from '../lib/fit'
  import Invite from '../lib/Invite.svelte'
  import { member, self, host, pad, eventText, nameLines, type House, type Role } from '../api/view'
  import { t, when, date, i18n } from '../i18n/index.svelte'

  let { house }: { house: House } = $props()
  let me = $derived(self(house))
  let owner = $derived(host(house))
  let works = $derived(house.problems.reduce((n, p) => n + p.workCount, 0))
  let free = $derived(house.capacity - house.members.length)
  let notice = $derived(house.board[0])
  let inviting = $state(false)
  const count = (r: Role) => house.members.filter((m) => m.role === r).length

  // 每个角色在首页的最主要动作
  let action = $derived(
    me.role === 'host' ? { label: t('act.invite'), hint: t('act.inviteHint', { n: free }), href: undefined }
    : me.role === 'steward' ? { label: t('act.newProblem'), hint: t('act.newProblemHint'), href: `#/${house.id}/problems/new` }
    : { label: t('act.newProposal'), hint: t('act.newProposalHint'), href: `#/${house.id}/propose` }
  )
</script>

<div class="wrap">
  <section class="intro">
    <div class="label">{t('house.founded', { date: date(house.founded) })}</div>
    <h1 use:fit={i18n.lang}>
      {#each nameLines(house.name, i18n.lang) as line}<span>{line}</span>{/each}
    </h1>

    <div class="sign">
      <i class="bar"></i>
      <p>{house.signboard ?? ''}</p>
      <div class="sign-by">
        <Shape role="host" size={10} />
        <span>{t('house.signBy', { name: owner.name })}</span>
        <!-- "修改招牌"：引擎还没有改招牌的命令，按钮先不放（不留没有动作的按钮） -->
      </div>
    </div>

    <div class="action" data-tour="action" data-write>
      {#if action.href}
        <a class="primary" href={action.href}><i>+</i><span>{action.label}</span></a>
      {:else}
        <button class="primary" aria-expanded={inviting} onclick={() => (inviting = !inviting)}><i>+</i><span>{action.label}</span></button>
      {/if}
      <p class="consequence">{action.hint}</p>
    </div>
    {#if inviting && me.role === 'host'}<Invite {house} onclose={() => (inviting = false)} />{/if}

    <div class="stats">
      <a href="#/{house.id}/problems"><b class="num">{pad(house.problems.length)}</b><span>{t('stat.problems')}</span></a>
      <a href="#/{house.id}/problem/0001"><b class="num">{pad(works)}</b><span>{t('stat.works')}</span></a>
      <a href="#/{house.id}/proposals" class:todo={house.proposals.length > 0}><b class="num">{pad(house.proposals.length)}</b><span>{t('stat.pending')}</span></a>
    </div>

    {#if notice}
      {@const nm = member(house, notice.by)}
      <a class="notice" href="#/{house.id}/board">
        <div class="label nh">{t('board.latest')}</div>
        <div class="nt">{notice.title}</div>
        <p class="nx">{notice.text}</p>
        <div class="nb">
          <Shape role={nm.role} size={10} />
          <span>{t('board.postedBy', { name: nm.name })}</span>
          <span class="when">{when(notice.when)}</span>
          <span class="more">{t('board.replies', { n: notice.replies.length })} →</span>
        </div>
      </a>
    {/if}

    <div class="recent">
      <div class="label head">{t('house.recent')}</div>
      <ul>
        {#each house.recent as e}
          {@const m = member(house, e.by)}
          <li>
            <Shape role={m.role} size={10} />
            <span class="who">{m.name}<AiMark {m} /></span>
            <span class="what">{eventText(e)}</span>
            <span class="when">{when(e.when)}</span>
          </li>
        {/each}
      </ul>
    </div>
  </section>

  <section class="seats">
    <div data-tour="room"><Room {house} /></div>
    <div class="legend">
      <span><Shape role="host" size={12} />{t('role.host')} <em class="num">{count('host')}</em></span>
      <span><Shape role="steward" size={12} />{t('role.steward')} · {t('role.stewardNote')} <em class="num">{count('steward')}</em></span>
      <span><Shape role="guest" size={12} />{t('role.guest')} <em class="num">{count('guest')}</em></span>
      <span><Shape role={null} size={12} />{t('role.empty')} <em class="num">{free}</em></span>
    </div>
  </section>
</div>

<style>
  .wrap {
    display: grid;
    grid-template-columns: minmax(0, 1fr) clamp(300px, 36vw, 540px);
    gap: clamp(40px, 5vw, 72px);
    padding: var(--pad-top) var(--pad-x) 56px;
    min-height: 100vh;
  }
  .intro { display: flex; flex-direction: column; min-width: 0; }

  h1 {
    font-family: var(--display);
    font-weight: var(--heavy);
    font-size: clamp(48px, 5.8vw, 84px);
    line-height: 1.04;
    letter-spacing: var(--track);
    margin-top: 22px;
    white-space: nowrap;
    hyphens: none;
  }
  h1 span { display: block; width: max-content; }

  .sign { margin-top: 28px; max-width: 34em; }
  .bar { display: block; width: 72px; height: 8px; background: var(--ink); margin-bottom: 18px; }
  .sign p { font-size: clamp(16px, 1.3vw, 19px); line-height: 1.6; letter-spacing: 0.02em; }
  .sign-by { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; margin-top: 10px; font-size: 13px; color: var(--ink-2); }

  .action { display: flex; flex-wrap: wrap; align-items: center; gap: 14px 24px; margin-top: 36px; }
  .action .consequence { flex: 1 1 16em; }

  .stats {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    margin-top: 40px;
    border-top: 2px solid var(--ink);
  }
  .stats a { padding: 16px 16px 14px 0; display: flex; flex-direction: column; min-width: 0; transition: background 200ms var(--ease); }
  .stats a + a { padding-left: 18px; border-left: 1px solid var(--ink); }
  .stats a:hover { background: var(--paper-deep); }
  .stats b { font-size: clamp(40px, 3.9vw, 56px); font-weight: 300; line-height: 1; letter-spacing: -0.02em; }
  .stats span { font-size: 14px; margin-top: 8px; font-weight: 500; line-height: 1.3; }
  /* 待办：整格反白，不用红色 */
  .stats a.todo { background: var(--ink); color: var(--paper); }

  .notice { display: block; margin-top: 40px; padding: 18px 22px 16px; border: 2px solid var(--ink); transition: background 200ms var(--ease); }
  .notice:hover { background: var(--paper-deep); }
  .nh { margin-bottom: 8px; }
  .nt { font-family: var(--display); font-weight: 700; font-size: 19px; line-height: 1.35; letter-spacing: var(--track); }
  .nx { margin-top: 6px; font-size: 14px; line-height: 1.7; color: var(--ink-2); display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
  .nb { display: flex; flex-wrap: wrap; align-items: center; gap: 4px 10px; margin-top: 12px; font-size: 12px; color: var(--ink-2); }
  .nb .when { font-family: var(--latin); color: var(--ink-3); }
  .nb .more { margin-left: auto; font-weight: 700; color: var(--ink); }

  .recent { margin-top: auto; padding-top: 40px; }
  .head { padding-bottom: 10px; border-bottom: 1px solid var(--ink); }
  ul { list-style: none; }
  li {
    display: grid;
    grid-template-columns: 10px auto minmax(0, 1fr) auto;
    gap: 12px;
    align-items: baseline;
    padding: 10px 0;
    border-bottom: 1px solid var(--rule);
    font-size: 14px;
    line-height: 1.45;
  }
  li :global(svg) { align-self: center; }
  .who { font-weight: 700; white-space: nowrap; }
  .what { color: var(--ink-2); }
  .when { font-family: var(--latin); font-size: 12px; color: var(--ink-3); white-space: nowrap; text-align: right; }

  .seats { display: flex; flex-direction: column; padding-top: 8px; min-width: 0; }
  .seats > div:first-child { width: 100%; }
  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 10px 24px;
    margin-top: 52px;
    font-size: 13px;
    font-weight: 500;
  }
  .legend span { display: inline-flex; align-items: center; gap: 8px; white-space: nowrap; }
  .legend em { font-style: normal; color: var(--ink-3); }

  /* 窄窗口：座位图放到内容上方 */
  @media (max-width: 1023.98px) {
    .wrap { grid-template-columns: minmax(0, 1fr); min-height: 0; gap: 48px; }
    .seats { order: -1; max-width: 520px; }
    .recent { margin-top: 0; }
  }
  @media (max-width: 560px) {
    li { grid-template-columns: 10px minmax(0, 1fr) auto; }
    .who { grid-column: 2; grid-row: 1; }
    .when { grid-column: 3; grid-row: 1; }
    .what { grid-column: 2 / -1; grid-row: 2; }
  }
</style>
