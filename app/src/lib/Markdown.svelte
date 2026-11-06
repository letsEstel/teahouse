<script lang="ts">
  import Linked from './Linked.svelte'
  import MdImage from './MdImage.svelte'
  import { parse, type Block, type Inline } from './markdown'
  import { inApp } from '../api/backend'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { report } from '../api/errors.svelte'

  // 显示 Markdown 正文（语法、安全见 markdown.ts）。lead：'first' = 第一段是摘要，字大一号；'all' = 每段都是
  let { text, house, lead }: { text: string; house: string; lead?: 'first' | 'all' } = $props()
  let tree = $derived(parse(text))
  function open(e: MouseEvent, url: string) {
    if (!inApp) return
    e.preventDefault()
    openUrl(url).catch((err) => report({ code: 'internal', message: `${url}: ${err}` }))
  }
</script>

{#snippet inline(list: Inline[])}
  {#each list as x}
    {#if x.t === 'text'}<Linked text={x.text} />
    {:else if x.t === 'code'}<code>{x.text}</code>
    {:else if x.t === 'strong'}<strong>{@render inline(x.c)}</strong>
    {:else if x.t === 'em'}<em>{@render inline(x.c)}</em>
    {:else if x.t === 'del'}<del>{@render inline(x.c)}</del>
    {:else if x.t === 'br'}<br />
    {:else if x.t === 'link'}<a href={x.url} target="_blank" rel="noopener noreferrer" onclick={(e) => open(e, x.url)}>{@render plain(x.c)}</a>
    {/if}
  {/each}
{/snippet}
{#snippet plain(list: Inline[])}
  {#each list as x}
    {#if x.t === 'text'}{x.text}{:else if x.t === 'code'}<code>{x.text}</code>{:else if x.t === 'strong' || x.t === 'em' || x.t === 'del' || x.t === 'link'}{@render plain(x.c)}{/if}
  {/each}
{/snippet}

{#snippet blocks(list: Block[], top: boolean)}
  {#each list as b, i}
    {#if b.t === 'p'}<p class:lead={lead === 'all' ? top : lead === 'first' && top && i === 0}>{@render inline(b.c)}</p>
    {:else if b.t === 'h'}
      <svelte:element this={`h${Math.min(b.level + 2, 6)}`} class="h{Math.min(b.level, 4)}">{@render inline(b.c)}</svelte:element>
    {:else if b.t === 'img'}<MdImage {house} src={b.src} alt={b.alt} />
    {:else if b.t === 'ul'}<ul>{#each b.items as it}<li class:task={it.task !== undefined}>{#if it.task !== undefined}<i class="box" class:on={it.task} aria-hidden="true"></i>{/if}<div>{@render blocks(it.blocks, false)}</div></li>{/each}</ul>
    {:else if b.t === 'ol'}<ol start={b.start}>{#each b.items as it}<li><div>{@render blocks(it.blocks, false)}</div></li>{/each}</ol>
    {:else if b.t === 'quote'}<blockquote>{@render blocks(b.blocks, false)}</blockquote>
    {:else if b.t === 'code'}<pre><code>{b.text}</code></pre>
    {:else if b.t === 'hr'}<hr />
    {:else if b.t === 'table'}
      <div class="table"><table>
        <thead><tr>{#each b.head as c}<th>{@render inline(c)}</th>{/each}</tr></thead>
        <tbody>{#each b.rows as r}<tr>{#each r as c}<td>{@render inline(c)}</td>{/each}</tr>{/each}</tbody>
      </table></div>
    {/if}
  {/each}
{/snippet}

<div class="md">{@render blocks(tree, true)}</div>

<style>
  .md { font-size: 15px; line-height: 1.85; overflow-wrap: anywhere; min-width: 0; }
  .md p { margin: 0 0 12px; white-space: pre-line; }
  .md .lead { font-size: 17px; line-height: 1.8; margin-bottom: 22px; }
  .md .h1, .md .h2, .md .h3, .md .h4 { font-family: var(--display); letter-spacing: var(--track); line-height: 1.35; }
  .md .h1 { font-size: 24px; font-weight: var(--heavy); margin: 26px 0 10px; }
  .md .h2 { font-size: 19px; font-weight: var(--heavy); margin: 24px 0 8px; padding-top: 12px; border-top: 2px solid var(--ink); }
  .md .h3 { font-size: 16px; font-weight: 700; margin: 18px 0 6px; }
  .md .h4 { font-size: 14px; font-weight: 700; margin: 14px 0 4px; }
  .md > :global(:first-child) { margin-top: 0; }
  .md ul, .md ol { margin: 0 0 12px; padding-left: 1.5em; }
  .md li > div > :global(p:last-child) { margin-bottom: 4px; }
  .md li.task { list-style: none; display: flex; gap: 8px; margin-left: -1.5em; align-items: baseline; }
  .box { flex: none; width: 11px; height: 11px; border: 1.5px solid var(--ink); }
  .box.on { background: var(--ink); }
  .md blockquote { margin: 0 0 14px; padding: 4px 0 4px 14px; border-left: 5px solid var(--ink); color: var(--ink-2); }
  .md code { font-family: ui-monospace, Menlo, Consolas, var(--sans), monospace; font-size: 0.88em; padding: 1px 4px; background: var(--paper-deep); }
  .md pre { margin: 0 0 14px; padding: 10px 12px; background: var(--paper-deep); overflow-x: auto; }
  .md pre code { padding: 0; background: none; white-space: pre; overflow-wrap: normal; }
  .md hr { border: 0; border-top: 1px solid var(--ink); margin: 18px 0; }
  .md del { color: var(--ink-3); }
  .md a { color: inherit; text-decoration: underline; text-underline-offset: 3px; }
  .table { overflow-x: auto; margin: 0 0 14px; }
  .md table { border-collapse: collapse; width: 100%; font-size: 14px; }
  .md th, .md td { text-align: left; padding: 5px 14px 5px 0; border-bottom: 1px solid var(--rule); }
  .md th { border-bottom: 2px solid var(--ink); }
</style>
