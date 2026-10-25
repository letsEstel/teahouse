<script lang="ts">
  import { api, write } from '../api/backend'
  import { t, date } from '../i18n/index.svelte'
  import type { DeviceInfo, House } from '../api/types'

  // 我的设备（docs/DEVICES.md）：同一个座位坐在几台设备上。
  // 在旧设备上"添加一台设备"拿到配对码，在新设备的"加入一间茶馆"里粘贴。
  // 移除：只能移除自己的另一台设备，不能移除正在用的这台，也不能移除最后一台（法则第 19 条）。
  // 移除和配对都对这台设备所在的全部茶馆生效，所以这里不按茶馆分开显示。
  let { house }: { house: House } = $props()
  const MAX = 5   // 法则第 18 条：每人最多 5 台

  // 新设备是 Relay 代写进座位表的（法则第 19 条），本机要先同步才看得到：所以先 sync 再读
  let devices = $state<DeviceInfo[] | null>(null)
  async function load() {
    await api('sync', { house: house.id }).catch(() => null)
    devices = await api('devices', { house: house.id }).catch(() => devices ?? [])
  }
  $effect(() => { load() })

  // ---- 移除：先点"移除"，再点"确定移除"，和删除留言一样 ----
  let confirming = $state<string | null>(null)
  let removing = $state(false)
  const canRemove = (d: DeviceInfo) => !d.thisDevice && (devices?.length ?? 0) > 1
  async function remove(d: DeviceInfo) {
    removing = true
    const r = await write('unpair', { device: d.fingerprint })
    removing = false
    confirming = null
    if (r) await load()
  }

  // ---- 添加一台设备：生成配对码，10 分钟内有效，只能用一次 ----
  let newName = $state('')
  let code = $state<string | null>(null)
  let expires = $state(0)
  let now = $state(Date.now())
  let copied = $state(false)
  let pairing = $state(false)
  async function pair() {
    pairing = true
    const r = await write('pair', { house: house.id, name: newName.trim() || undefined })
    pairing = false
    if (!r) return
    code = r
    copied = false
    expires = Date.now() + 10 * 60_000
  }
  $effect(() => {
    if (!code) return
    const timer = setInterval(() => (now = Date.now()), 1000)
    return () => clearInterval(timer)
  })
  // 配对码显示期间，每 5 秒看一次新设备到了没有；到了就说一声，收起配对码
  let joined = $state<string | null>(null)
  $effect(() => {
    if (!code || expired) return
    const known = new Set((devices ?? []).map((d) => d.fingerprint))
    const timer = setInterval(async () => {
      await load()
      const fresh = devices?.find((d) => !known.has(d.fingerprint))
      if (fresh) {
        joined = fresh.name || t('devices.first')
        code = null
      }
    }, 5000)
    return () => clearInterval(timer)
  })
  let left = $derived(Math.max(0, expires - now))
  let expired = $derived(!!code && left === 0)
  const mmss = (ms: number) => `${Math.floor(ms / 60_000)}:${String(Math.floor(ms / 1000) % 60).padStart(2, '0')}`

  // 复制的是一整句话：新设备的"加入一间茶馆"会从里面认出配对码，也看得出这是配对码而不是邀请
  let message = $derived(code ? t('devices.copyText', { code }) : '')
  let codeBox = $state<HTMLElement>()
  async function copy() {
    try {
      await navigator.clipboard.writeText(message)
    } catch {
      // 剪贴板不可用时退回到选中文字
      const range = document.createRange()
      if (codeBox) { range.selectNodeContents(codeBox); getSelection()?.removeAllRanges(); getSelection()?.addRange(range) }
      document.execCommand('copy')
    }
    copied = true
  }
</script>

<div class="wrap">
  <header>
    <div class="label">{t('devices.kicker')}</div>
    <h1>{t('devices.title')}</h1>
    <p class="lede">{t('devices.lede')}</p>
  </header>

  <section class="list">
    <div class="head">
      <h2>{t('devices.mine')}</h2>
      {#if devices}<span class="num count">{devices.length}/{MAX}</span>{/if}
    </div>
    {#if devices}
      <ul>
        {#each devices as d (d.fingerprint)}
          <li class:this={d.thisDevice}>
            <svg class="glyph" width="20" height="20" viewBox="0 0 20 20" aria-hidden="true">
              <rect x="4" y="1" width="12" height="18" fill={d.thisDevice ? 'var(--ink)' : 'none'} stroke="var(--ink)" stroke-width="2" />
            </svg>
            <div class="what">
              <div class="name">
                {d.name || t('devices.first')}
                {#if d.thisDevice}<span class="here">{t('devices.this')}</span>{/if}
              </div>
              <div class="added">{t('devices.added', { date: date(d.added, 'short') })}</div>
            </div>
            <div class="act">
              {#if canRemove(d)}
                {#if confirming === d.fingerprint}
                  <button class="secondary small sure" disabled={removing} onclick={() => remove(d)}>{t('devices.removeSure')}</button>
                  <button class="link" onclick={() => (confirming = null)}>{t('devices.keep')}</button>
                {:else}
                  <button class="secondary small" onclick={() => (confirming = d.fingerprint)}>{t('devices.remove')}</button>
                {/if}
              {:else}
                <span class="why">{d.thisDevice ? t('devices.cantThis') : t('devices.cantLast')}</span>
              {/if}
            </div>
          </li>
          {#if confirming === d.fingerprint}
            <li class="warn"><p>{t('devices.removeHint', { name: d.name || t('devices.first') })}</p></li>
          {/if}
        {/each}
      </ul>
    {/if}
  </section>

  <section class="add">
    <h2>{t('devices.add')}</h2>
    {#if joined}<p class="joined" role="status">{t('devices.joined', { name: joined })}</p>{/if}
    <p class="consequence">{t('devices.addHint')}</p>
    {#if devices && devices.length >= MAX}
      <p class="full">{t('devices.full', { n: MAX })}</p>
    {:else if !code || expired}
      <div class="ask">
        <label class="field">
          <span class="label">{t('devices.newName')}</span>
          <input bind:value={newName} placeholder={t('devices.newNamePh')} maxlength="32" />
        </label>
        <button class="primary" disabled={pairing} onclick={pair}><i>+</i><span>{t(expired ? 'devices.again' : 'devices.makeCode')}</span></button>
      </div>
      {#if expired}<p class="full">{t('devices.expired')}</p>{/if}
    {:else}
      <div class="code" aria-live="polite">
        <div class="label">{t('devices.code')}</div>
        <div class="value num" bind:this={codeBox}>{code}</div>
        <div class="row">
          <button class="primary" onclick={copy}><span>{copied ? t('devices.copied') : t('devices.copy')}</span></button>
          <span class="timer num" class:low={left < 60_000}>{t('devices.left', { time: mmss(left) })}</span>
        </div>
        <p class="steps">{t('devices.paste')}</p>
      </div>
    {/if}
  </section>
</div>

<style>
  .wrap { padding: var(--pad-top) var(--pad-x) 80px; max-width: calc(52rem + 2 * var(--pad-x)); }
  header { padding-bottom: clamp(20px, 3vw, 32px); border-bottom: 2px solid var(--ink); }
  h1 { font-family: var(--display); font-size: clamp(36px, 6vw, 72px); line-height: 1.05; font-weight: 700; margin: 8px 0 14px; letter-spacing: var(--track); }
  .lede { max-width: 36em; color: var(--ink-2); line-height: 1.8; }
  h2 { font-family: var(--display); font-size: 22px; font-weight: 700; }

  .list { margin-top: clamp(24px, 4vw, 40px); }
  .head { display: flex; align-items: baseline; gap: 12px; padding-bottom: 10px; border-bottom: 1px solid var(--ink); }
  .count { font-size: 13px; color: var(--ink-3); }
  ul { list-style: none; }
  li { display: flex; align-items: center; gap: 14px; padding: 14px 4px; border-bottom: 1px solid var(--rule); }
  .glyph { flex: none; }
  .what { flex: 1; min-width: 0; }
  .name { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; font-weight: 700; font-size: 16px; overflow-wrap: anywhere; }
  /* "本机"：和待办数字一样的黑底白字小块，不用颜色 */
  .here { background: var(--ink); color: var(--paper); font-size: 11px; font-weight: 600; padding: 2px 6px; letter-spacing: 0.04em; }
  .added { font-size: 13px; color: var(--ink-3); margin-top: 2px; }
  .act { display: flex; flex-wrap: wrap; justify-content: flex-end; align-items: center; gap: 8px 12px; }
  .why { font-size: 12px; color: var(--ink-3); text-align: right; max-width: 16em; }
  .sure { background: var(--ink); color: var(--paper); }
  .link { font-size: 13px; text-decoration: underline; text-underline-offset: 3px; }
  .warn { border-bottom: 1px solid var(--rule); padding: 0 4px 14px; }
  .warn p { font-size: 13px; color: var(--ink-2); line-height: 1.7; }

  .add { margin-top: clamp(32px, 5vw, 56px); padding-top: 20px; border-top: 2px solid var(--ink); }
  .add .consequence { margin: 6px 0 18px; max-width: 34em; }
  .ask { display: flex; flex-wrap: wrap; align-items: flex-end; gap: 14px; }
  .field { display: grid; gap: 6px; }
  .field input { border: 1.5px solid var(--ink); background: var(--paper); padding: 10px 12px; font: inherit; color: var(--ink); width: min(16rem, 100%); }
  .full { margin-top: 12px; font-size: 14px; color: var(--ink-2); }
  .joined { margin: 8px 0 14px; padding: 10px 14px; background: var(--ink); color: var(--paper); font-size: 14px; }

  /* 配对码：整段显示，大一些，一眼能对上 */
  .code { border: 2px solid var(--ink); padding: clamp(16px, 3vw, 28px); display: grid; gap: 14px; }
  .value { font-family: var(--latin); font-size: clamp(18px, 3.2vw, 28px); font-weight: 600; letter-spacing: 0.04em; overflow-wrap: anywhere; user-select: all; }
  .row { display: flex; flex-wrap: wrap; align-items: center; gap: 12px 20px; }
  .timer { font-size: 14px; color: var(--ink-2); }
  .timer.low { color: var(--ink); font-weight: 700; }
  .steps { font-size: 14px; line-height: 1.8; color: var(--ink-2); }

  @media (max-width: 640px) {
    li { flex-wrap: wrap; }
    .act { width: 100%; justify-content: flex-start; padding-left: 34px; }
    .why { text-align: left; }
  }
</style>
