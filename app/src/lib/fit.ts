// 超大字号（茶馆名、页面大标题）随容器缩放：先取 CSS 给的字号，放不下就按比例缩小，绝不换行。
// 约定：每一行是一个 display: block; width: max-content 的子元素。
export function fit(node: HTMLElement, _key?: unknown) {
  const run = () => {
    const before = node.style.fontSize
    node.style.fontSize = ''
    const max = parseFloat(getComputedStyle(node).fontSize)
    const avail = node.clientWidth - 2  // 留 2px 给字形外伸
    // 量文字本身的宽度
    const range = document.createRange()
    range.selectNodeContents(node)
    const need = range.getBoundingClientRect().width
    // 元素暂时没有排版（例如所在页面隐藏）时量出来是 0，保持原样
    if (!need || avail <= 0) { node.style.fontSize = before; return }
    const size = need > avail + 0.5 ? `${Math.floor((max * avail * 100) / need) / 100}px` : ''
    // 只在值变化时写：重设和写回在同一帧内完成，ResizeObserver 不会来回触发
    node.style.fontSize = size === before ? before : size
  }
  // 观察容器（宽度变化）和每一行（网页字体按需加载，换上真字体时行宽会变）
  const ro = new ResizeObserver(run)
  ro.observe(node.parentElement ?? node)
  for (const line of node.children) ro.observe(line)
  document.fonts.addEventListener('loadingdone', run)
  run()
  return {
    update: run,
    destroy: () => { ro.disconnect(); document.fonts.removeEventListener('loadingdone', run) },
  }
}
