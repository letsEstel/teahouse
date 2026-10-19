<script lang="ts">
  import type { Status } from '../api/view'
  import { t } from '../i18n/index.svelte'
  // 状态只用明度表达：空心 = 待认领，半满 = 进行中，实心 = 已办成
  let { status, label = true }: { status: Status; label?: boolean } = $props()
</script>

<span class="status {status}">
  <i></i>{#if label}<span>{t(`status.${status}`)}</span>{/if}
</span>

<style>
  .status { display: inline-flex; align-items: center; gap: 10px; font-size: 13px; font-weight: 500; white-space: nowrap; }
  i { display: block; flex: none; width: 28px; height: 8px; border: 1.5px solid var(--ink); }
  .active i::before { content: ''; display: block; width: 50%; height: 100%; background: var(--ink); }
  .solved i { background: var(--ink); }
  .solved span { color: var(--ink-2); }
</style>
