// 开发期的后端：POST <base>/v1/<命令名>，请求体和返回体都是 JSON（CLIENT-API 第 4 节）。
// base 默认 http://127.0.0.1:7377（引擎的 serve）；用开发中间层（scripts/engine-bridge.mjs）时为空，走同源。
import type { Args, Command, Result, Returns } from './types'
import type { Backend } from './backend'

export function httpBackend(base: string): Backend {
  return {
    kind: 'http',
    async call<C extends Command>(cmd: C, args: Args<C>): Promise<Result<Returns<C>>> {
      let res: Response
      try {
        res = await fetch(`${base}/v1/${cmd}`, {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify(args ?? {}),
        })
      } catch {
        // 连不上的是本机的引擎（不是 Relay），用 params 区分
        return { ok: false, error: { code: 'offline', params: { engine: 'unreachable' }, message: `cannot reach the engine at ${base || location.origin}` } }
      }
      try {
        return (await res.json()) as Result<Returns<C>>
      } catch {
        return { ok: false, error: { code: 'internal', message: `HTTP ${res.status}` } }
      }
    },
  }
}
