<script lang="ts">
  // 一段文字，里面的网址可以点。在应用里交给系统打开（浏览器或对应的 App，比如腾讯文档），
  // 不在茶馆的窗口里打开；浏览器里照常开新标签页
  import { linkify } from './links'
  import { splitMentions, useMentions } from './mentions'
  import { inApp } from '../api/backend'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { report } from '../api/errors.svelte'
  let { text }: { text: string } = $props()
  // 在留言里（Notes 提供了名单）：@名字 加亮，点到我的更醒目
  const mentions = useMentions()
  function open(e: MouseEvent, url: string) {
    if (!inApp) return
    e.preventDefault()
    openUrl(url).catch((err) => report({ code: 'internal', message: `${url}: ${err}` }))
  }
</script>

{#each linkify(text) as part}{#if part.url}<a class="ext" href={part.url} target="_blank" rel="noopener noreferrer" onclick={(e) => open(e, part.url!)}>{part.text}</a>{:else}{#each splitMentions(part.text, mentions?.() ?? []) as bit}{#if bit.at}<b class="at" class:me={bit.at.me}>{bit.text}</b>{:else}{bit.text}{/if}{/each}{/if}{/each}

<style>
  .at { font-weight: 700; text-decoration: underline; text-decoration-thickness: 1.5px; text-underline-offset: 3px; }
  .at.me { background: var(--ink); color: var(--paper); text-decoration: none; padding: 0 3px; }
  .ext { color: inherit; text-decoration: underline; text-underline-offset: 3px; overflow-wrap: anywhere; }
</style>
