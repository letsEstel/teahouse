<script lang="ts">
  // 设计陈列页（docs/DESIGN.md）：所有颜色、字体、符号和组件的各种状态排在一页上。
  // 只在开发和假数据里打开（#/<茶馆>/design），截图脚本也拍它：改了样式，一张图就看出改坏了哪里。
  import Shape from '../lib/Shape.svelte'
  import Room from '../lib/Room.svelte'
  import Mark from '../lib/Mark.svelte'
  import StatusMark from '../lib/StatusMark.svelte'
  import AccessMark from '../lib/AccessMark.svelte'
  import MachineMark from '../lib/MachineMark.svelte'
  import WorkGlyph from '../lib/WorkGlyph.svelte'
  import Diff from '../lib/Diff.svelte'
  import Markdown from '../lib/Markdown.svelte'
  import type { House, Role, Status, WorkKind } from '../api/view'
  import { t } from '../i18n/index.svelte'

  let { house }: { house: House } = $props()
  const colors = ['--paper', '--paper-deep', '--ink', '--ink-2', '--ink-3', '--rule', '--red', '--yellow', '--blue']
  let values = $state<Record<string, string>>({})
  $effect(() => {
    const s = getComputedStyle(document.documentElement)
    values = Object.fromEntries(colors.map((c) => [c, s.getPropertyValue(c).trim()]))
  })
  const roles: (Role | null)[] = ['host', 'steward', 'guest', null]
  const statuses: Status[] = ['open', 'active', 'solved']
  const kinds: WorkKind[] = ['doc', 'plan', 'table', 'image', 'file', 'link']
  const machine = { id: 'ai-1', name: '1号', role: 'steward' as Role, joined: '', kind: 'machine' as const, agent: 'Claude Code' }
  const sample = '## 一节\n\n正文里有**加粗**、`代码`和[链接](https://example.org)。\n\n- 第一条\n- 第二条\n\n> 引用的一段话。\n\n| 甲 | 乙 |\n|---|---|\n| 1 | 2 |'
</script>

<div class="wrap">
  <div class="label">DESIGN.md</div>
  <h1>{t('design.title')}</h1>
  <p class="consequence">{t('design.lead')}</p>

  <section>
    <h2><span class="num">01</span>{t('design.colors')}</h2>
    <div class="swatches">
      {#each colors as c}
        <div class="sw"><i style:background="var({c})"></i><code>{c}</code><span class="num">{values[c] ?? ''}</span></div>
      {/each}
    </div>
  </section>

  <section>
    <h2><span class="num">02</span>{t('design.type')}</h2>
    <p class="display">{house.name}</p>
    <p class="label">04 · {t('nav.proposals')}</p>
    <p>{t('design.body')}</p>
    <p class="num big">0001 0027 15/15</p>
    <p class="consequence">{t('prop.adoptHint')}</p>
  </section>

  <section>
    <h2><span class="num">03</span>{t('design.shapes')}</h2>
    <div class="row">
      {#each [12, 20, 40] as s}
        <div class="group">{#each roles as r}<Shape role={r} size={s} />{/each}</div>
      {/each}
    </div>
    <div class="room"><Room {house} /></div>
  </section>

  <section>
    <h2><span class="num">04</span>{t('design.marks')}</h2>
    <div class="row">{#each statuses as s}<StatusMark status={s} />{/each}</div>
    <div class="row"><AccessMark a="write" label /><AccessMark a="propose" label /><AccessMark a="read" label /></div>
    <div class="row"><span>1号<MachineMark m={machine} /></span><em class="badge">4</em></div>
    <div class="row">{#each kinds as k}<span class="glyph"><WorkGlyph kind={k} size={32} /><span class="label">{t(`kind.${k}`)}</span></span>{/each}</div>
  </section>

  <section>
    <h2><span class="num">05</span>{t('design.buttons')}</h2>
    <div class="row">
      <button class="primary"><i>+</i><span>{t('act.newProblem')}</span></button>
      <button class="secondary">{t('prop.reject')}</button>
      <button class="secondary" disabled>{t('prop.reject')}</button>
    </div>
    <div class="row">
      <button class="primary" aria-busy="true" data-sending={t('send.sending')}><i>+</i><span>{t('act.newProblem')}</span></button>
      <button class="secondary" aria-busy="true" data-sending={t('send.slow')}>{t('prop.reject')}</button>
    </div>
  </section>

  <section>
    <h2><span class="num">06</span>{t('design.diff')}</h2>
    <Diff lines={[{ kind: 'same', text: '院内全部 42 个地面车位。' }, { kind: 'del', text: '07:00–19:00 为日间时段。' }, { kind: 'add', text: '07:00–18:30 为日间时段。' }]} />
    <div class="md"><Markdown text={sample} house={house.id} /></div>
  </section>

  <section>
    <h2><span class="num">07</span>{t('design.logo')}</h2>
    <div class="row logos">{#each [16, 24, 32, 44, 96] as s}<span class="logo"><Mark size={s} /><span class="num">{s}</span></span>{/each}</div>
  </section>
</div>

<style>
  .wrap { padding: var(--pad-top) var(--pad-x) 80px; max-width: 1100px; }
  h1 { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(44px, 5vw, 72px); line-height: 1.05; letter-spacing: var(--track); margin: 10px 0 12px; }
  section { padding: 28px 0; border-top: 2px solid var(--ink); margin-top: 28px; }
  h2 { display: flex; gap: 14px; align-items: baseline; font-family: var(--display); font-weight: var(--heavy); font-size: 22px; margin-bottom: 18px; }
  h2 .num { font-size: 14px; font-weight: 500; color: var(--ink-2); }
  .swatches { display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 14px; }
  .sw { display: grid; gap: 4px; font-size: 13px; }
  .sw i { display: block; height: 56px; border: 1px solid var(--ink); }
  .sw code { font-weight: 700; }
  .display { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(40px, 5vw, 64px); line-height: 1.1; letter-spacing: var(--track); margin-bottom: 10px; }
  section > p { margin-bottom: 10px; }
  .big { font-size: 32px; }
  .row { display: flex; flex-wrap: wrap; gap: 14px 28px; align-items: center; margin-bottom: 16px; }
  .group { display: flex; gap: 10px; align-items: center; }
  .room { max-width: 420px; margin-top: 12px; }
  .glyph { display: inline-flex; flex-direction: column; align-items: center; gap: 4px; }
  .secondary:disabled { color: var(--ink-3); border-color: var(--ink-3); cursor: default; }
  .md { max-width: 40em; margin-top: 20px; }
  .logos { align-items: flex-end; }
  .logo { display: inline-flex; flex-direction: column; align-items: center; gap: 6px; font-size: 12px; color: var(--ink-2); }
</style>
