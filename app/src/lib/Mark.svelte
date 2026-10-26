<script lang="ts">
  // 茶馆的标志：墨色的圆里一芽二叶（安吉白茶）。叶子是两段同半径的圆弧，茎是一根直线。
  // 和应用图标（src-tauri/icons/source.svg）同一份几何；颜色走 --ink / --paper，夜间模式自动反过来。
  let { size = 44 }: { size?: number } = $props()
</script>

<svg width={size} height={size} viewBox="162 162 700 700" aria-hidden="true">
  <circle cx="512" cy="512" r="350" fill="var(--ink)" />
  <rect x="502.5" y="431.8" width="19.0" height="285.0" fill="var(--paper)" />
  <circle cx="512.0" cy="716.8" r="9.5" fill="var(--paper)" />
  <g transform="translate(512.0 626.5) rotate(-56)">
    <path d="M0 0 A210.0 210.0 0 0 1 0 -256.5 A210.0 210.0 0 0 1 0 0Z" fill="var(--paper)" />
  </g>
  <g transform="translate(512.0 550.5) rotate(56)">
    <path d="M0 0 A182.9 182.9 0 0 1 0 -223.2 A182.9 182.9 0 0 1 0 0Z" fill="var(--paper)" />
  </g>
  <g transform="translate(512.0 446.0) rotate(0)">
    <path d="M0 0 A211.5 211.5 0 0 1 0 -194.8 A211.5 211.5 0 0 1 0 0Z" fill="var(--paper)" />
  </g>
</svg>
