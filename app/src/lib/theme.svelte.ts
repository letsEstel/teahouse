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

// 安卓外壳（MainActivity.kt）按页面的实际深浅给状态栏、导航栏上色。
declare global {
  interface Window { TeahouseAndroid?: { setDark(dark: boolean): void } }
}
const systemDark = matchMedia('(prefers-color-scheme: dark)')

function tellAndroid() {
  const dark = theme.mode === 'dark' || (theme.mode === 'auto' && systemDark.matches)
  try { window.TeahouseAndroid?.setDark(dark) } catch {}
}

function apply() {
  if (theme.mode === 'auto') delete document.documentElement.dataset.theme
  else document.documentElement.dataset.theme = theme.mode
  tellAndroid()
}
apply()
systemDark.addEventListener('change', tellAndroid)
// 桥接可能比页面脚本晚注入，加载完再报一次
window.addEventListener('load', tellAndroid)

export function setTheme(m: Theme) {
  theme.mode = m
  apply()
  try {
    if (m === 'auto') localStorage.removeItem('teahouse.theme')
    else localStorage.setItem('teahouse.theme', m)
  } catch {}
}
