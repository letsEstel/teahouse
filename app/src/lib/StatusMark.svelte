<script lang="ts">
  import type { Status } from '../data/fake'
  import { statusName } from '../data/fake'
  let { status, label = true }: { status: Status; label?: boolean } = $props()
</script>

<span class="status {status}">
  <i></i>{#if label}<span>{statusName[status]}</span>{/if}
</span>

<style>
  .status { display: inline-flex; align-items: center; gap: 10px; font-size: 13px; font-weight: 500; white-space: nowrap; }
  i { display: block; width: 28px; height: 6px; }
  .open i { background: none; border: 1.5px solid var(--ink); height: 8px; }
  .active i { background: var(--yellow); }
  .solved i { background: var(--ink); }
  .solved span { color: var(--ink-2); }
</style>
