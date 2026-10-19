<script lang="ts">
  import { errors, dismiss, errorText } from '../api/errors.svelte'
  import { t } from '../i18n/index.svelte'
  // 写入失败时的提示：排在内容最上面，不盖住任何东西；墨色细框，不用颜色
</script>

{#if errors.list.length}
  <div class="bar" role="alert">
    {#each errors.list as e, i}
      <div class="item">
        <i aria-hidden="true"></i>
        <div class="text">
          <p class="what">{errorText(e)}</p>
          {#if e.reasons?.length}
            <ul>{#each e.reasons as r}<li>{r}</li>{/each}</ul>
          {/if}
          {#if e.message && errorText(e) !== e.message}<p class="detail">{e.message}</p>{/if}
        </div>
        <button onclick={() => dismiss(i)}>{t('err.dismiss')}</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .bar { padding: 16px var(--pad-x) 0; display: flex; flex-direction: column; gap: 10px; }
  .item { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; gap: 16px; align-items: start; border: 2px solid var(--ink); padding: 14px 18px; background: var(--paper); }
  i { width: 14px; height: 14px; margin-top: 4px; background: var(--ink); }
  .what { font-weight: 700; line-height: 1.5; }
  ul { margin: 6px 0 0 1.1em; font-size: 14px; line-height: 1.6; }
  .detail { margin-top: 4px; font-size: 12px; color: var(--ink-3); font-family: var(--latin); overflow-wrap: anywhere; }
  button { font-size: 13px; font-weight: 700; border-bottom: 1.5px solid var(--ink); padding: 2px 0; white-space: nowrap; }
</style>
