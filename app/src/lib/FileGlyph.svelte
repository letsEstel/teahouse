<script lang="ts">
  import type { FileKind } from '../data/archive'
  // 档案里的几何图标，只用墨色：■ 文档 ▦ 表格 ▲ 图片 ◆ 方案 ● 问题 ▬ 公告 ▫ 留言 ⊞ 成员表 □ 目录
  let { kind, open = false }: { kind: FileKind | 'folder'; open?: boolean } = $props()
</script>

<svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
  {#if kind === 'folder'}
    <rect x="0.75" y="0.75" width="10.5" height="10.5" fill="none" stroke="var(--ink)" stroke-width="1.5" />
    {#if open}<rect x="3.5" y="3.5" width="5" height="5" fill="var(--ink)" />{/if}
  {:else if kind === 'doc'}
    <rect x="1" y="1" width="10" height="10" fill="var(--ink)" />
  {:else if kind === 'table'}
    <rect x="0.75" y="0.75" width="10.5" height="10.5" fill="none" stroke="var(--ink)" stroke-width="1.5" />
    <line x1="0.75" y1="6" x2="11.25" y2="6" stroke="var(--ink)" stroke-width="1.2" />
    <line x1="6" y1="0.75" x2="6" y2="11.25" stroke="var(--ink)" stroke-width="1.2" />
  {:else if kind === 'image'}
    <polygon points="6,1 11.5,11 0.5,11" fill="var(--ink)" />
  {:else if kind === 'plan'}
    <polygon points="6,0.5 11.5,6 6,11.5 0.5,6" fill="var(--ink)" />
  {:else if kind === 'problem'}
    <circle cx="6" cy="6" r="5" fill="var(--ink)" />
  {:else if kind === 'notice'}
    <rect x="0.5" y="3.5" width="11" height="5" fill="var(--ink)" />
  {:else if kind === 'note'}
    <rect x="2.75" y="2.75" width="6.5" height="6.5" fill="none" stroke="var(--ink)" stroke-width="1.5" />
  {:else}
    <rect x="0.75" y="0.75" width="10.5" height="10.5" fill="none" stroke="var(--ink)" stroke-width="1.5" />
    <rect x="3" y="3" width="2.5" height="2.5" fill="var(--ink)" />
    <rect x="6.5" y="3" width="2.5" height="2.5" fill="var(--ink)" />
    <rect x="3" y="6.5" width="2.5" height="2.5" fill="var(--ink)" />
  {/if}
</svg>

<style>
  svg { display: block; flex: none; }
</style>
