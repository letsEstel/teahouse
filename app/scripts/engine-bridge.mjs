// 开发中间层（只在 npm run dev 时用）：引擎的 serve 还没做好之前，
// 把 POST /v1/<命令名> 转给引擎的命令行 `teahouse-client --root <目录> call <命令> <参数>`。
// 返回的信封和 HTTP 接口完全一样，等 serve 做好，去掉这一层即可，界面不用改。
//
//   TEAHOUSE_ENGINE_CLI=/path/to/teahouse-client TEAHOUSE_CLIENT_ROOT=/path/to/root npm run dev:engine
import { execFile } from 'node:child_process'
import { createReadStream, realpathSync, statSync } from 'node:fs'
import { resolve, sep } from 'node:path'

export function engineBridge() {
  const cli = process.env.TEAHOUSE_ENGINE_CLI
  const root = process.env.TEAHOUSE_CLIENT_ROOT
  return {
    name: 'teahouse-engine-bridge',
    apply: 'serve',
    configureServer(server) {
      if (!cli || !root) return
      const rootReal = realpathSync(resolve(root))
      server.middlewares.use('/v1/', (req, res) => {
        const cmd = (req.url ?? '').replace(/^\//, '').split('?')[0]
        // 附件：GET /v1/blob?house=&id=（CLIENT-API 第 4 节）。先让引擎给出本机路径，
        // 只返回引擎根目录里面的文件
        if (req.method === 'GET' && cmd === 'blob') {
          const q = new URL(req.url ?? '', 'http://x').searchParams
          const args = JSON.stringify({ house: q.get('house'), id: q.get('id') })
          return execFile(cli, ['--root', root, 'call', 'blob', args], (_e, stdout) => {
            let real = ''
            try { real = realpathSync(resolve(JSON.parse(stdout).data)) } catch {}
            if (!real.startsWith(rootReal + sep) || !statSync(real).isFile()) {
              res.statusCode = 404
              return res.end()
            }
            createReadStream(real).pipe(res)
          })
        }
        let body = ''
        req.on('data', (c) => (body += c))
        req.on('end', () => {
          execFile(cli, ['--root', root, 'call', cmd, body || '{}'], { maxBuffer: 64 << 20 }, (err, stdout, stderr) => {
            res.setHeader('content-type', 'application/json; charset=utf-8')
            // 引擎在 {ok:false} 时退出码为 1，但标准输出里仍是合法的信封
            if (stdout.trim()) return res.end(stdout)
            res.end(JSON.stringify({ ok: false, error: { code: 'internal', message: stderr || String(err) } }))
          })
        })
      })
      server.config.logger.info(`  engine bridge: /v1/* → ${cli} --root ${root}`)
    },
  }
}
