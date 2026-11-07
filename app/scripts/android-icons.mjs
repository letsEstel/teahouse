// 安卓启动器图标（0021：非原生的启动器上，圆形图标旁边有一圈白边）。
// 原来自适应图标的前景是整张 source.png（米白方底上的墨色圆），底色米白；启动器各自裁剪，
// 墨色圆外的米白就露成白边。现在满版：底色是墨色（android-patch.sh），前景只有米白的叶子，
// 位置和大小与 source.svg 相同；旧式图标（不认自适应图标的启动器）是撑满的墨色圆。
// 改了 source.svg 里的叶子以后运行：node scripts/android-icons.mjs
import { chromium } from 'playwright'
import { readFileSync, readdirSync, existsSync } from 'node:fs'
import { homedir } from 'node:os'
import { join } from 'node:path'

const icons = 'src-tauri/icons'
const svg = readFileSync(`${icons}/source.svg`, 'utf8')
const leaves = svg.match(/<rect x="502[\s\S]*?(?=\n<\/svg>)/)[0]   // 叶梗和三片叶子，全在 1024 的画布上
const ink = '#141414'

// 前景：108dp 的画布，坐标和 source.svg 一样（叶子在启动器的安全区里）
const foreground = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">${leaves}</svg>`
// 旧式图标：墨色圆撑满，叶子跟着圆一起放大（source.svg 里圆的半径是 350）
const s = 512 / 350
const legacy = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024"><circle cx="512" cy="512" r="512" fill="${ink}"/><g transform="translate(512 512) scale(${s}) translate(-512 -512)">${leaves}</g></svg>`

const densities = { mdpi: 1, hdpi: 1.5, xhdpi: 2, xxhdpi: 3, xxxhdpi: 4 }
const libs = join(homedir(), '.cache/teahouse/chrome-libs')
const env = existsSync(libs) ? { ...process.env, LD_LIBRARY_PATH: libs } : undefined
const cache = join(homedir(), '.cache/ms-playwright')
const dir = existsSync(cache) && readdirSync(cache).filter((d) => d.startsWith('chromium_headless_shell-')).sort().pop()
const browser = await chromium.launch(dir ? { executablePath: join(cache, dir, 'chrome-headless-shell-linux64/chrome-headless-shell'), env } : {})
const page = await browser.newPage()
async function render(src, size, path) {
  await page.setViewportSize({ width: size, height: size })
  await page.setContent(`<style>html,body{margin:0;background:transparent}svg{display:block;width:${size}px;height:${size}px}</style>${src}`)
  await page.screenshot({ path, omitBackground: true })
}
for (const [d, k] of Object.entries(densities)) {
  const out = `${icons}/android/mipmap-${d}`
  await render(foreground, 108 * k, `${out}/ic_launcher_foreground.png`)
  await render(legacy, 48 * k, `${out}/ic_launcher.png`)
  await render(legacy, 48 * k, `${out}/ic_launcher_round.png`)
}
await browser.close()
console.log('安卓图标已更新：src-tauri/icons/android/')
