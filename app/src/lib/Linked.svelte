<script lang="ts">
  // 一段文字，里面的网址可以点。在应用里交给系统打开（浏览器或对应的 App，比如腾讯文档），
  // 不在茶馆的窗口里打开；浏览器里照常开新标签页
  import { linkify } from './links'
  import { inApp } from '../api/backend'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { report } from '../api/errors.svelte'
  let { text }: { text: string } = $props()
  function open(e: MouseEvent, url: string) {
    if (!inApp) return
    e.preventDefault()
    openUrl(url).catch((err) => report({ code: 'internal', message: `${url}: ${err}` }))
  }
</script>

{#each linkify(text) as part}{#if part.url}<a class="ext" href={part.url} target="_blank" rel="noopener noreferrer" onclick={(e) => open(e, part.url!)}>{part.text}</a>{:else}{part.text}{/if}{/each}

<style>
  .ext { color: inherit; text-decoration: underline; text-underline-offset: 3px; overflow-wrap: anywhere; }
</style>
