<script lang="ts">
  // AI 成员名字旁边的标记（docs/MCP.md 第 1 节第 2 条）：墨线方框里写"AI"。
  // 不用颜色：颜色只属于人的角色（红、黄、蓝），AI 标记是"是谁"之外的另一件事。
  import type { Member } from '../api/types'
  import { t } from '../i18n/index.svelte'

  let { m }: { m: Member } = $props()
</script>

{#if m.kind === 'ai'}
  <abbr class="ai" title={m.agent ? t('ai.markAgent', { agent: m.agent }) : t('ai.mark')}>AI</abbr>
{/if}

<style>
  .ai {
    display: inline-block;
    margin-inline-start: 0.4em;
    padding: 0 0.32em;
    border: 1px solid currentColor;
    font-family: var(--latin);
    font-size: max(0.68em, 9px);
    font-weight: 600;
    line-height: 1.35;
    letter-spacing: 0.06em;
    text-decoration: none;
    vertical-align: 0.12em;
    white-space: nowrap;
    cursor: help;
  }
</style>
