<script lang="ts">
  // 编辑器：CodeMirror 6（Obsidian 用的也是它）+ Markdown 实时预览。
  // 实时预览：光标不在的那几行，把 # ** > - 这些记号藏起来，直接显示成标题、粗体、引用、列表；
  // 光标移到哪一行，那一行就露出原文，可以照常改。文件里存的始终是纯 Markdown。
  // 表格（CSV）用同一个编辑器，不做预览（markdown={false}）。
  import { onMount } from 'svelte'
  import { EditorState, RangeSetBuilder, type Range } from '@codemirror/state'
  import { EditorView, Decoration, ViewPlugin, WidgetType, keymap, placeholder as ph, type DecorationSet, type ViewUpdate } from '@codemirror/view'
  import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands'
  import { syntaxTree } from '@codemirror/language'
  import { markdown as md } from '@codemirror/lang-markdown'

  let { value, onchange, markdown = true, placeholder = '' }: {
    value: string
    onchange?: (text: string) => void
    markdown?: boolean
    placeholder?: string
  } = $props()

  let host: HTMLDivElement
  let view: EditorView | undefined

  /** 列表的记号换成一个小方块 */
  class Bullet extends WidgetType {
    toDOM() {
      const s = document.createElement('span')
      s.className = 'cm-bullet'
      return s
    }
  }
  const hide = Decoration.replace({})
  const bullet = Decoration.replace({ widget: new Bullet() })
  const mark = (cls: string) => Decoration.mark({ class: cls })
  const line = (cls: string) => Decoration.line({ class: cls })

  function preview(v: EditorView): DecorationSet {
    const doc = v.state.doc
    // 光标（或选区）所在的行：这些行露出原文
    const active = new Set<number>()
    for (const r of v.state.selection.ranges) {
      for (let n = doc.lineAt(r.from).number; n <= doc.lineAt(r.to).number; n++) active.add(n)
    }
    const touches = (from: number, to: number) => {
      for (let n = doc.lineAt(from).number; n <= doc.lineAt(to).number; n++) if (active.has(n)) return true
      return false
    }
    const out: Range<Decoration>[] = []

    // 开头的 frontmatter（--- … ---）：标题、说明这些，整块淡一些
    let bodyFrom = 0
    const fm = doc.sliceString(0, Math.min(doc.length, 2000)).match(/^---\n[\s\S]*?\n---(\n|$)/)
    if (fm) {
      bodyFrom = fm[0].length
      for (let n = 1; n <= doc.lineAt(Math.max(0, bodyFrom - 1)).number; n++) out.push(line('cm-front').range(doc.line(n).from))
    }

    for (const { from, to } of v.visibleRanges) {
      syntaxTree(v.state).iterate({
        from, to,
        enter: (node) => {
          if (node.from < bodyFrom) return
          const name = node.name
          const h = /^ATXHeading(\d)$/.exec(name)
          if (h) {
            out.push(line(`cm-h cm-h${h[1]}`).range(doc.lineAt(node.from).from))
            return
          }
          if (name === 'HeaderMark') {
            // "## " 连同后面的空格一起藏
            if (!touches(node.from, node.to)) out.push(hide.range(node.from, Math.min(node.to + 1, doc.lineAt(node.from).to)))
            return
          }
          if (name === 'StrongEmphasis') out.push(mark('cm-strong').range(node.from, node.to))
          else if (name === 'Emphasis') out.push(mark('cm-em').range(node.from, node.to))
          else if (name === 'InlineCode') out.push(mark('cm-code').range(node.from, node.to))
          else if (name === 'Link') out.push(mark('cm-link').range(node.from, node.to))
          else if (name === 'Blockquote') {
            for (let n = doc.lineAt(node.from).number; n <= doc.lineAt(node.to).number; n++) out.push(line('cm-quote').range(doc.line(n).from))
          } else if (name === 'EmphasisMark' || name === 'CodeMark' || name === 'LinkMark' || name === 'URL') {
            const parent = node.node.parent
            if (parent && !touches(parent.from, parent.to) && node.to > node.from) out.push(hide.range(node.from, node.to))
          } else if (name === 'QuoteMark') {
            if (!touches(node.from, node.to)) out.push(hide.range(node.from, Math.min(node.to + 1, doc.lineAt(node.from).to)))
          } else if (name === 'ListMark') {
            const text = doc.sliceString(node.from, node.to)
            if (!touches(node.from, node.to) && /^[-*+]$/.test(text)) out.push(bullet.range(node.from, node.to))
          }
        },
      })
    }
    // RangeSetBuilder 要求按位置排好；行装饰排在同一位置的其他装饰前面
    out.sort((a, b) => a.from - b.from || a.value.startSide - b.value.startSide)
    const b = new RangeSetBuilder<Decoration>()
    for (const r of out) b.add(r.from, r.to, r.value)
    return b.finish()
  }

  const livePreview = ViewPlugin.fromClass(
    class {
      decorations: DecorationSet
      constructor(v: EditorView) { this.decorations = preview(v) }
      update(u: ViewUpdate) {
        if (u.docChanged || u.selectionSet || u.viewportChanged || syntaxTree(u.startState) !== syntaxTree(u.state)) this.decorations = preview(u.view)
      }
    },
    { decorations: (p) => p.decorations },
  )

  onMount(() => {
    view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: value,
        extensions: [
          history(),
          keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
          EditorView.lineWrapping,
          ph(placeholder),
          ...(markdown ? [md(), livePreview] : []),
          EditorView.updateListener.of((u) => { if (u.docChanged) onchange?.(u.state.doc.toString()) }),
          EditorView.contentAttributes.of({ autocapitalize: 'sentences', spellcheck: 'true' }),
        ],
      }),
    })
    return () => view?.destroy()
  })

  /** 在光标处插入一段文字（例如附件的引用） */
  export function insert(text: string) {
    if (!view) return
    const { from, to } = view.state.selection.main
    view.dispatch({ changes: { from, to, insert: text }, selection: { anchor: from + text.length } })
    view.focus()
  }
</script>

<div class="editor" class:plain={!markdown} bind:this={host}></div>

<style>
  .editor { border: 1.5px solid var(--ink); background: var(--paper); min-height: 16rem; }
  .editor :global(.cm-editor) { min-height: 16rem; font-size: 16px; }
  .editor :global(.cm-editor.cm-focused) { outline: 2px solid var(--ink); outline-offset: 2px; }
  .editor :global(.cm-scroller) { font-family: var(--sans); line-height: 1.75; }
  .editor :global(.cm-content) { padding: 18px 0; caret-color: var(--ink); }
  .editor :global(.cm-line) { padding: 0 20px; }
  .editor :global(.cm-cursor) { border-left: 2px solid var(--ink); }
  .editor :global(.cm-selectionBackground),
  .editor :global(.cm-focused .cm-selectionBackground) { background: color-mix(in srgb, var(--ink) 16%, transparent) !important; }
  .editor :global(.cm-placeholder) { color: var(--ink-3); }
  .plain :global(.cm-scroller) { font-family: ui-monospace, 'SFMono-Regular', Menlo, monospace; font-size: 14px; }

  /* 实时预览 */
  .editor :global(.cm-front) { color: var(--ink-3); font-size: 13px; }
  .editor :global(.cm-h) { font-weight: 700; line-height: 1.4; }
  .editor :global(.cm-h1) { font-size: 1.75em; padding-top: 0.4em; }
  .editor :global(.cm-h2) { font-size: 1.35em; padding-top: 0.5em; }
  .editor :global(.cm-h3) { font-size: 1.15em; padding-top: 0.4em; }
  .editor :global(.cm-strong) { font-weight: 700; }
  .editor :global(.cm-em) { font-style: italic; }
  .editor :global(.cm-code) { font-family: ui-monospace, Menlo, monospace; font-size: 0.9em; background: var(--paper-deep); }
  .editor :global(.cm-link) { text-decoration: underline; text-underline-offset: 3px; }
  .editor :global(.cm-quote) { border-left: 4px solid var(--ink); padding-left: 16px; margin-left: 20px; }
  .editor :global(.cm-bullet) { display: inline-block; width: 6px; height: 6px; background: var(--ink); vertical-align: 0.2em; margin-right: 2px; }
</style>
