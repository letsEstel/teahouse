<script lang="ts">
  // 一段文字，里面的网址可以点（在系统浏览器或对应的 App 里打开）
  import { linkify } from './links'
  let { text }: { text: string } = $props()
</script>

{#each linkify(text) as part}{#if part.url}<a class="ext" href={part.url} target="_blank" rel="noopener noreferrer">{part.text}</a>{:else}{part.text}{/if}{/each}

<style>
  .ext { color: inherit; text-decoration: underline; text-underline-offset: 3px; overflow-wrap: anywhere; }
</style>
