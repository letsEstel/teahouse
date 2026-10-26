<script lang="ts">
  // 机器成员名字旁边的标记：墨线方框里写"机器"。座位表只分人和机器（kind = human | machine，
  // 旧的 ai 也算机器）；agent 说明是哪种机器（Claude Code、GitHub Actions……），停留时显示。
  // 不用颜色：颜色只属于人的角色（红、黄、蓝），机器标记是"是谁"之外的另一件事。
  import { isMachine, type Member } from '../api/types'
  import { t } from '../i18n/index.svelte'

  let { m }: { m: Member } = $props()
</script>

{#if isMachine(m)}
  <abbr class="machine" title={m.agent ? t('machine.markAgent', { agent: m.agent }) : t('machine.mark')}>{t('machine.short')}</abbr>
{/if}

<style>
  .machine {
    display: inline-block;
    margin-inline-start: 0.4em;
    padding: 0 0.32em;
    border: 1px solid currentColor;
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
