// 打包后的后端：invoke('<命令名>', 参数)（CLIENT-API 第 4 节）。
// src-tauri 里只有一个原始 invoke 处理器，把每个命令交给引擎的 Client::call，原样返回信封。
import { invoke } from '@tauri-apps/api/core'
import type { Args, Command, Result, Returns } from './types'
import type { Backend } from './backend'

export const tauriBackend: Backend = {
  kind: 'tauri',
  async call<C extends Command>(cmd: C, args: Args<C>): Promise<Result<Returns<C>>> {
    try {
      return await invoke<Result<Returns<C>>>(cmd, (args ?? {}) as Record<string, unknown>)
    } catch (e) {
      // 只有外壳本身出错时才会走到这里（引擎的错误都在信封里）
      return { ok: false, error: { code: 'internal', message: String(e) } }
    }
  },
}
