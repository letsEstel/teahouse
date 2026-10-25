<script lang="ts">
  import { write } from '../api/backend'
  import type { House, Role } from '../api/types'
  import { t } from '../i18n/index.svelte'

  // 老板邀请人入座（INVITE.md）：选角色和有效期，生成一次性的邀请，复制后用微信等任何方式发出去。
  let { house, onclose }: { house: House; onclose: () => void } = $props()
  let hasSteward = $derived(house.members.some((m) => m.role === 'steward'))
  let role = $state<Role>('guest')
  let days = $state(7)
  let name = $state('')
  // 邀请 AI 成员（法则第 17 条、docs/AI-入座.md）：邀请上带 kind = ai 和 agent。只是标记，不限制角色；默认茶客
  let ai = $state(false)
  let agent = $state('Claude Code')
  function setAi(on: boolean) {
    ai = on
    if (on) role = 'guest'
  }
  let code = $state<string | null>(null)
  let copied = $state(false)

  async function create() {
    code = await write('invite', { house: house.id, role, days, name: name.trim() || undefined, ...(ai ? { kind: 'ai' as const, agent: agent.trim() || undefined } : {}) })
    copied = false
  }
  // 发给对方的一段话：邀请本身已经带着茶馆的地址，对方整段粘贴即可
  let message = $derived(code ? t(ai ? 'invite.messageAi' : 'invite.message', { house: house.name, code }) : '')
  async function copy() {
    try {
      await navigator.clipboard.writeText(message)
      copied = true
    } catch {
      copied = false
    }
  }
</script>

<section class="invite" aria-labelledby="invite-title">
  <div class="head">
    <h2 id="invite-title">{t('invite.title')}</h2>
    <button class="close" onclick={onclose}>{t('invite.close')}</button>
  </div>

  {#if !code}
    <div class="row">
      <span class="label">{t('invite.who')}</span>
      <div class="seg" role="radiogroup" aria-label={t('invite.who')}>
        <button role="radio" aria-checked={!ai} class:on={!ai} onclick={() => setAi(false)}>{t('invite.person')}</button>
        <button role="radio" aria-checked={ai} class:on={ai} onclick={() => setAi(true)}>{t('invite.ai')}</button>
      </div>
    </div>
    {#if ai}
      <label class="row">
        <span class="label">{t('invite.agent')}</span>
        <input bind:value={agent} placeholder="Claude Code" />
      </label>
      <p class="consequence ai-note">{t('invite.aiHint')}</p>
    {/if}
    <div class="row">
      <span class="label">{t('invite.role')}</span>
      <div class="seg" role="radiogroup" aria-label={t('invite.role')}>
        <button role="radio" aria-checked={role === 'guest'} class:on={role === 'guest'} onclick={() => (role = 'guest')}>{t('role.guest')}</button>
        <button role="radio" aria-checked={role === 'steward'} class:on={role === 'steward'} disabled={hasSteward} title={hasSteward ? t('invite.stewardTaken') : ''} onclick={() => (role = 'steward')}>{t('role.steward')}</button>
      </div>
    </div>
    <div class="row">
      <span class="label">{t('invite.days')}</span>
      <div class="seg" role="radiogroup" aria-label={t('invite.days')}>
        {#each [1, 7, 30] as d}
          <button role="radio" aria-checked={days === d} class:on={days === d} onclick={() => (days = d)}>{t('invite.dayCount', { n: d })}</button>
        {/each}
      </div>
    </div>
    <label class="row">
      <span class="label">{t('invite.name')}</span>
      <input bind:value={name} placeholder={t('invite.namePh')} />
    </label>
    <div class="go">
      <p class="consequence">{t('invite.hint')}</p>
      <button class="primary" onclick={create}><i>+</i><span>{t('invite.create')}</span></button>
    </div>
  {:else}
    <p class="code num">{code}</p>
    <p class="consequence">{t('invite.ready', { n: days })}</p>
    <div class="go">
      <button class="primary" onclick={copy}><i>{#if copied}<svg width="18" height="18" viewBox="0 0 18 18" aria-hidden="true"><polyline points="3,9.5 7.5,14 15,4.5" fill="none" stroke="currentColor" stroke-width="2.2" /></svg>{:else}<svg width="18" height="18" viewBox="0 0 18 18" aria-hidden="true"><rect x="1.5" y="5.5" width="10" height="11" fill="none" stroke="currentColor" stroke-width="2" /><rect x="6.5" y="1.5" width="10" height="11" fill="currentColor" /></svg>{/if}</i><span>{copied ? t('invite.copied') : t('invite.copy')}</span></button>
      <button class="secondary" onclick={() => (code = null)}>{t('invite.another')}</button>
    </div>
  {/if}
</section>

<style>
  .ai-note { margin: -4px 0 6px; }
  .invite { margin-top: 20px; border: 2px solid var(--ink); padding: 20px 22px 22px; display: flex; flex-direction: column; gap: 16px; max-width: 40em; }
  .head { display: flex; justify-content: space-between; align-items: baseline; gap: 12px; }
  h2 { font-family: var(--display); font-weight: var(--heavy); font-size: 22px; letter-spacing: var(--track); }
  .close { font-size: 13px; font-weight: 500; border-bottom: 1px solid var(--ink); }
  .row { display: flex; flex-wrap: wrap; align-items: center; gap: 8px 16px; }
  .row .label { min-width: 6em; }
  .seg { display: flex; }
  .seg button { padding: 6px 14px; border: 1.5px solid var(--ink); margin-right: -1.5px; font-size: 14px; font-weight: 500; }
  .seg button.on { background: var(--ink); color: var(--paper); }
  .seg button:disabled { color: var(--ink-3); border-color: var(--ink-3); cursor: default; }
  input { font: inherit; color: var(--ink); background: transparent; border: 1.5px solid var(--ink); padding: 6px 10px; flex: 1 1 12em; min-width: 0; }
  ::placeholder { color: var(--ink-3); opacity: 1; }
  .go { display: flex; flex-wrap: wrap; align-items: center; gap: 12px 20px; }
  .go .consequence { flex: 1 1 16em; }
  .code { font-size: clamp(18px, 2.2vw, 26px); letter-spacing: 0.06em; padding: 14px 16px; border: 1.5px dashed var(--ink); overflow-wrap: anywhere; }
</style>
