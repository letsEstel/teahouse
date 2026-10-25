<script lang="ts">
  import { write, inApp, demoMode, setDemo } from '../api/backend'
  import { t } from '../i18n/index.svelte'
  import type { HouseSummary } from '../api/types'

  // 入座：只问名字、粘贴邀请（CLIENT-API 第 2 节，INVITE.md）。
  // 同一个框也接受配对码（DEVICES.md）：从自己另一台设备上拿来的码，这台设备坐进同一个座位。
  // 两种码长得一样，Relay 才分得出来；但第 02 步问的不一样——邀请问"你叫什么"，配对码问"这台设备叫什么"。
  // 所以：粘贴的文字里写着"配对码"（"添加一台设备"复制出来的就是这样一句话）时自动切换，也可以手动勾选。
  // 老板发来的邀请是一整串"邀请码@Relay 地址"；也认得整段聊天消息里夹着的邀请码和 teahouse:// 链接。
  // 界面上不出现"密钥"：这台设备会自动生成一把只属于你的钥匙，由引擎处理。
  let { houses, onjoined }: { houses: HouseSummary[]; onjoined: (s: HouseSummary) => void } = $props()

  let pasted = $state('')
  let relayInput = $state('')
  let name = $state('')
  let busy = $state(false)
  /** 用户手动选的：null = 看粘贴的文字自动判断 */
  let pairingChoice = $state<boolean | null>(null)

  function parse(src: string) {
    const code = src.match(/TH(?:-[0-9A-Z]{4}){4}/i)?.[0]?.toUpperCase() ?? ''
    let relay = ''
    const after = code ? src.slice(src.toUpperCase().indexOf(code) + code.length).match(/^@([\w.-]+(?::\d+)?)/) : null
    const link = src.match(/teahouse:\/\/([\w.@-]+(?::\d+)?)/i)
    if (after) relay = after[1]
    else if (link) relay = link[1]
    if (relay && !relay.includes('@')) relay = `teahouse@${relay}`
    return { code, relay }
  }
  let found = $derived(parse(pasted))
  let pairing = $derived(pairingChoice ?? /配对码|pairing code|Kopplungscode/i.test(pasted))
  // 配对码时替用户猜一个设备名（手机 / 电脑），可以改
  $effect(() => {
    if (pairing && !name.trim()) name = t(/Android|iPhone|iPad/i.test(navigator.userAgent) ? 'join.guessPhone' : 'join.guessComputer')
  })
  let relay = $derived(found.relay || (relayInput.trim() ? (relayInput.includes('@') ? relayInput.trim() : `teahouse@${relayInput.trim()}`) : ''))
  let missing = $derived([!found.code && t('join.needCode'), found.code && !relay && t('join.needRelay'), !name.trim() && t('join.needName')].filter(Boolean) as string[])

  async function join(e: SubmitEvent) {
    e.preventDefault()
    if (missing.length || busy) return
    busy = true
    const seat = await write('redeem', { relay, code: found.code, name: name.trim() })
    busy = false
    if (seat) onjoined(seat)
  }
</script>

<div class="wrap">
  {#if houses.length}<a class="back label" href="#/{houses[0].id}/house">← {t('join.back')}</a>{/if}
  <header>
    <svg class="door" width="64" height="64" viewBox="0 0 64 64" aria-hidden="true">
      <rect x="2" y="2" width="60" height="60" fill="none" stroke="var(--ink)" stroke-width="4" />
      <rect x="24" y="56" width="16" height="8" fill="var(--paper)" />
      <circle cx="32" cy="30" r="9" fill="none" stroke="var(--ink)" stroke-width="3" />
    </svg>
    <h1>{t('join.title')}</h1>
    <p class="lede">{t('join.lede')}</p>
  </header>

  <form onsubmit={join}>
    <section class="step">
      <div class="sn num">01</div>
      <div class="sc">
        <label for="invite"><h2>{t('join.invite')}</h2></label>
        <textarea id="invite" name="invite" rows="3" bind:value={pasted} placeholder={t('join.invitePh')}></textarea>
        <label class="tick">
          <input type="checkbox" checked={pairing} onchange={(e) => (pairingChoice = e.currentTarget.checked)} />
          <span>{t('join.pairing')}</span>
        </label>
        {#if pairing && found.code}<p class="seen"><i></i>{t('join.pairingSeen')}</p>{/if}
        {#if found.code}
          <p class="seen">
            <i></i>{t(pairing ? 'join.seenPair' : 'join.seenCode', { code: found.code })}{#if found.relay}<br />{t('join.seenRelay', { relay: found.relay.replace(/^teahouse@/, '') })}{/if}
          </p>
        {/if}
        {#if found.code && !found.relay}
          <label class="field">
            <span class="label">{t('join.relay')}</span>
            <input name="relay" bind:value={relayInput} placeholder="relay.example.org" />
            <span class="hint">{t('join.relayHint')}</span>
          </label>
        {/if}
      </div>
    </section>

    <section class="step">
      <div class="sn num">02</div>
      <div class="sc">
        <label for="name"><h2>{t(pairing ? 'join.deviceName' : 'join.name')}</h2></label>
        <input id="name" name="name" bind:value={name} autocomplete={pairing ? 'off' : 'nickname'} maxlength={pairing ? 32 : undefined} />
        <p class="hint">{t(pairing ? 'join.deviceNameHint' : 'join.nameHint')}</p>
      </div>
    </section>

    {#if inApp && !demoMode}
      <!-- 还没拿到邀请时，先用假数据看看（只在打包好的应用里出现） -->
      <div class="demo">
        <button type="button" class="secondary" onclick={() => setDemo(true)}>{t('demo.try')}</button>
        <p class="hint">{t('demo.tryHint')}</p>
      </div>
    {/if}

    <div class="submit">
      <div>
        <p class="consequence">{t('join.consequence')}</p>
        {#if missing.length}<p class="missing">{t('propose.need', { what: missing.join(' · ') })}</p>{/if}
      </div>
      <button type="submit" class="primary" disabled={missing.length > 0 || busy}>
        <i>→</i><span>{busy ? t('join.busy') : t('join.submit')}</span>
      </button>
    </div>
  </form>
</div>

<style>
  .wrap { padding: var(--pad-top) var(--pad-x) 80px; max-width: 980px; }
  .back { display: inline-block; margin-bottom: 24px; }
  header { padding-bottom: 28px; border-bottom: 2px solid var(--ink); }
  .door { display: block; margin-bottom: 24px; }
  h1 { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(48px, 6vw, 88px); line-height: 1; letter-spacing: var(--track); }
  .lede { margin-top: 16px; font-size: 17px; color: var(--ink-2); max-width: 32em; line-height: 1.7; }

  .step { display: grid; grid-template-columns: clamp(64px, 8vw, 120px) minmax(0, 1fr); gap: clamp(16px, 2.5vw, 36px); padding: 32px 0; border-bottom: 1px solid var(--ink); }
  .sn { font-size: clamp(44px, 5vw, 72px); font-weight: 300; line-height: 0.85; letter-spacing: -0.03em; }
  .sc { min-width: 0; display: flex; flex-direction: column; gap: 12px; }
  h2 { font-family: var(--display); font-weight: var(--heavy); font-size: clamp(20px, 1.8vw, 26px); letter-spacing: var(--track); line-height: 1.3; }
  input, textarea {
    font: inherit; color: var(--ink); background: transparent; width: 100%;
    border: 2px solid var(--ink); padding: 12px 14px; line-height: 1.6; outline: none;
  }
  textarea { font-family: var(--latin); font-size: 16px; resize: vertical; letter-spacing: 0.04em; }
  #name { font-family: var(--display); font-weight: 700; font-size: 20px; max-width: 18em; }
  input:focus, textarea:focus { outline: 3px solid var(--ink); outline-offset: 2px; }
  ::placeholder { color: var(--ink-3); opacity: 1; }
  .seen { display: flex; gap: 10px; align-items: baseline; font-size: 14px; font-family: var(--latin); line-height: 1.7; }
  .seen i { flex: none; width: 10px; height: 10px; background: var(--ink); transform: translateY(-1px); }
  .field { display: flex; flex-direction: column; gap: 6px; max-width: 28em; }
  .hint { font-size: 13px; color: var(--ink-2); }
  .tick { display: flex; align-items: center; gap: 10px; font-size: 14px; cursor: pointer; }
  .tick input { width: 18px; height: 18px; padding: 0; accent-color: var(--ink); flex: none; }

  .demo { display: flex; flex-wrap: wrap; align-items: center; gap: 10px 20px; padding: 24px 0; border-bottom: 1px solid var(--ink); }
  .submit { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 20px 32px; padding-top: 28px; }
  .submit > div { flex: 1 1 20em; }
  .missing { margin-top: 6px; font-size: 13px; font-weight: 700; }
  .primary:disabled { background: transparent; color: var(--ink-3); border-color: var(--ink-3); cursor: default; }
  .primary:disabled > i { border-right-color: var(--ink-3); }
  @media (max-width: 640px) { .step { grid-template-columns: minmax(0, 1fr); } }
</style>
