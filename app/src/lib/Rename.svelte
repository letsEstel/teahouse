<script lang="ts">
  import Shape from './Shape.svelte'
  import { write } from '../api/backend'
  import { member, self, type House } from '../api/view'
  import { t } from '../i18n/index.svelte'

  // 名字归本人（法则第 21 条）：每个人都能改自己的名字，老板还能改任何人的。id 不变；
  // 新名字不能和别人重复，这里先提示一声，最后由 Relay 判断。提交由 Relay 代写，留在历史里。
  let { house, refresh, onclose }: { house: House; refresh: () => void; onclose: () => void } = $props()
  let me = $derived(self(house))
  /** 空 = 我自己；老板可以选别人 */
  let picked = $state('')
  let who = $derived(picked || house.me)
  let target = $derived(member(house, who))
  /** null = 还没动过，显示现在的名字 */
  let typed = $state<string | null>(null)
  let name = $derived(typed ?? target.name)
  let done = $state('')

  function pick(id: string) {
    picked = id
    typed = null
    done = ''
  }
  let clean = $derived(name.trim())
  let taken = $derived(house.members.some((m) => m.id !== who && m.name.toLowerCase() === clean.toLowerCase()))
  let unchanged = $derived(clean === target.name)

  async function save(e: SubmitEvent) {
    e.preventDefault()
    if (!clean || taken || unchanged) return
    const r = await write('rename', { house: house.id, name: clean, ...(who === house.me ? {} : { member: who }) })
    if (r === null) return
    done = t('rename.done', { name: clean })
    typed = null
    refresh()
  }
</script>

<section class="rename" aria-labelledby="rename-title">
  <header>
    <h2 id="rename-title">{t('rename.title')}</h2>
    <button type="button" class="close" onclick={onclose}>{t('invite.close')}</button>
  </header>
  <form onsubmit={save}>
    {#if me.role === 'host'}
      <div class="field" role="radiogroup" aria-label={t('rename.who')}>
        <span class="label">{t('rename.who')}</span>
        <div class="people">
          {#each house.members as m}
            <button type="button" role="radio" aria-checked={who === m.id} class:on={who === m.id} onclick={() => pick(m.id)}>
              <Shape role={m.role} size={11} /><span>{m.name}</span>
            </button>
          {/each}
        </div>
      </div>
    {/if}
    <label class="field">
      <span class="label">{t('rename.new')}</span>
      <input value={name} maxlength="64" autocomplete="nickname" oninput={(e) => { typed = e.currentTarget.value; done = '' }} />
    </label>
    {#if taken}<p class="taken" role="alert">{t('rename.taken')}</p>{/if}
    {#if done}<p class="done" role="status">{done}</p>{/if}
    <p class="consequence">{who === house.me ? t('rename.hintSelf') : t('rename.hintOther', { name: target.name })}</p>
    <div class="go"><button class="primary" data-write disabled={!clean || taken || unchanged}><i>→</i><span>{t('rename.submit')}</span></button></div>
  </form>
</section>

<style>
  .rename { margin-top: 18px; border: 2px solid var(--ink); padding: 16px 18px 18px; }
  header { display: flex; justify-content: space-between; align-items: baseline; gap: 12px; }
  h2 { font-family: var(--display); font-weight: var(--heavy); font-size: 20px; letter-spacing: var(--track); }
  .close { background: none; border: 0; padding: 0; font: inherit; font-size: 13px; color: var(--ink-2); cursor: pointer; text-decoration: underline; }
  form { display: flex; flex-direction: column; gap: 12px; margin-top: 12px; }
  .field { display: flex; flex-direction: column; gap: 6px; min-width: 0; }
  .people { display: flex; flex-wrap: wrap; gap: 6px; }
  .people button { display: inline-flex; align-items: center; gap: 6px; padding: 4px 9px; background: transparent; border: 1px solid var(--ink); color: var(--ink); font: inherit; font-size: 14px; cursor: pointer; }
  .people button.on { background: var(--ink); color: var(--paper); }
  input { font: inherit; color: var(--ink); background: transparent; width: 100%; padding: 8px 10px; border: 1.5px solid var(--ink); border-radius: 0; }
  .taken, .done { margin: 0; font-size: 14px; font-weight: 700; }
  .consequence { margin: 0; }
  .go { display: flex; }
</style>
