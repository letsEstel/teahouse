// 夜间模式：默认跟随系统（prefers-color-scheme），侧栏可以手动改成浅色或深色。
// ?theme=light|dark 强制指定（截图用）。手动选择记在本机。
export type Theme = 'auto' | 'light' | 'dark'
export const themes: Theme[] = ['auto', 'light', 'dark']

function initial(): Theme {
  const q = new URLSearchParams(location.search).get('theme')
  if (q === 'light' || q === 'dark' || q === 'auto') return q
  try {
    const s = localStorage.getItem('teahouse.theme')
    if (s === 'light' || s === 'dark') return s
  } catch {}
  return 'auto'
}

export const theme = $state({ mode: initial() })

function apply() {
  if (theme.mode === 'auto') delete document.documentElement.dataset.theme
  else document.documentElement.dataset.theme = theme.mode
}
apply()

export function setTheme(m: Theme) {
  theme.mode = m
  apply()
  try {
    if (m === 'auto') localStorage.removeItem('teahouse.theme')
    else localStorage.setItem('teahouse.theme', m)
  } catch {}
}
