/// <reference types="svelte" />
/// <reference types="vite/client" />

/** package.json 的版本号和代号（vite.config.ts 的 define） */
declare const __RELEASE__: { version: string; codename: { zh: string; latin: string } }
