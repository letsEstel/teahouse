<script lang="ts">
  // 写入的结果写在按钮上（../api/sending.svelte.ts）。这里只给读屏软件念一句，不占页面位置、不盖住内容。
  import { sending } from '../api/sending.svelte'
  import { t } from '../i18n/index.svelte'

  let text = $derived(
    sending.count > 0 ? t('send.sending')
    : sending.result === 'pushed' ? t('send.pushed')
    : sending.result === 'local' ? t('send.local')
    : sending.result === 'done' ? t('send.done')
    : ''
  )
</script>

<div class="sr" role="status" aria-live="polite">{text}</div>

<style>
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
</style>
