// 界面和引擎之间只有这一个口子：call(命令名, 参数)。
// 命令名、参数、返回值见 docs/CLIENT-API.md 第 4 节（类型在 ./types.ts）。
//   fake  —— 现在：假数据（./fake.ts），截图和预览都用它
//   http  —— 第 2 步：开发期 POST http://127.0.0.1:7377/v1/<命令名>
//   tauri —— 第 4 步：打包后 invoke('<命令名>', 参数)
// 选择方式：环境变量 VITE_BACKEND（默认 fake）。界面任何地方都不要绕过这里去读数据。
import type { ApiError, Args, Command, Result, Returns } from './types'
import { fakeBackend } from './fake'

export interface Backend {
  kind: 'fake' | 'http' | 'tauri'
  call<C extends Command>(cmd: C, args: Args<C>): Promise<Result<Returns<C>>>
}

export class BackendError extends Error {
  constructor(public error: ApiError) {
    super(error.message)
  }
}

export const backend: Backend = fakeBackend

/** 调用命令；出错时抛出 BackendError（界面按 error.code 显示本地化的说明） */
export async function api<C extends Command>(cmd: C, args: Args<C>): Promise<Returns<C>> {
  const r = await backend.call(cmd, args)
  if (!r.ok) throw new BackendError(r.error)
  return r.data
}
