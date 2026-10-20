<script lang="ts">
  // 正文里的一张图片（![标题](附件)）：从本地缓存取出来显示，取不到时只写标题
  import { imageUrl } from './images'

  let { house, src, alt }: { house: string; src: string; alt: string } = $props()
  let url = $state<string | null>(null)
  $effect(() => {
    const s = src
    imageUrl(house, s).then((u) => { if (s === src) url = u })
  })
</script>

<figure>
  {#if url}<img src={url} {alt} />{/if}
  {#if alt}<figcaption>{alt}</figcaption>{/if}
</figure>

<style>
  figure { margin: 12px 0 18px; }
  img { display: block; max-width: 100%; max-height: 28rem; border: 1px solid var(--rule); }
  figcaption { margin-top: 6px; font-size: 12px; color: var(--ink-3); }
</style>
