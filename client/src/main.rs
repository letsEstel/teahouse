//! `teahouse-client`: the engine from the command line.
//!
//! ```text
//! teahouse-client [--root <dir>] call <command> [<json-args>]
//! teahouse-client [--root <dir>] serve [--port 7377] [--sync-every <seconds>|off]
//! teahouse-client [--root <dir>] key
//! teahouse-client mcp [--root <dir>]
//! teahouse-client dev-seed <fixtures.json> --relay <relay-root> --people <dir>
//! ```
//!
//! `call` prints the command's JSON envelope and exits 1 when it is
//! `{ok: false}`. `serve` is the development HTTP interface on 127.0.0.1.
//! `key` prints this device's public key (making it on first use), for
//! seating a device by hand during development. `dev-seed` builds the
//! example teahouses on a local relay (`scripts/dev-stack.sh` runs it).
//! `mcp` is an MCP server on stdio for an AI member (docs/MCP.md).

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;
use teahouse_client::{serve, Client};

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

const USAGE: &str = "usage: teahouse-client [--root <dir>] call <command> [<json-args>]
       teahouse-client [--root <dir>] serve [--port 7377] [--sync-every <seconds>|off]
       teahouse-client [--root <dir>] key
       teahouse-client mcp [--root <dir>]
       teahouse-client dev-seed <fixtures.json> --relay <relay-root> --people <dir>";

/// `<app data>/teahouse`, or `$TEAHOUSE_CLIENT_ROOT`.
fn default_root() -> Option<PathBuf> {
    if let Some(r) = std::env::var_os("TEAHOUSE_CLIENT_ROOT") {
        return Some(r.into());
    }
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    };
    base.map(|b| b.join("teahouse"))
}

/// `~/x` → `$HOME/x`: MCP configurations are not read by a shell.
fn expand_home(p: &str) -> PathBuf {
    match (p.strip_prefix("~/"), std::env::var_os("HOME")) {
        (Some(rest), Some(home)) => PathBuf::from(home).join(rest),
        _ => PathBuf::from(p),
    }
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // `mcp --root <dir>`, as MCP client configurations write it.
    if args.len() == 3 && args[0] == "mcp" && args[1] == "--root" {
        let root = args.pop().expect("three args");
        args = vec!["--root".into(), root, "mcp".into()];
    }
    let mut root = None;
    if args.first().map(String::as_str) == Some("--root") {
        if args.len() < 2 {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
        root = Some(expand_home(&args.remove(1)));
        args.remove(0);
    }
    let Some(root) = root.or_else(default_root) else {
        eprintln!("cannot tell where the engine keeps its data; pass --root");
        return ExitCode::from(2);
    };
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["call", command, rest @ ..] if rest.len() <= 1 => {
            let params = match rest.first() {
                Some(s) => match serde_json::from_str(s) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("arguments are not JSON: {e}");
                        return ExitCode::from(2);
                    }
                },
                None => serde_json::Value::Null,
            };
            let out = Client::new(root).call(command, params);
            println!("{out}");
            if out["ok"] == true {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        ["dev-seed", fixtures, "--relay", relay, "--people", people] => {
            let bin =
                std::env::var_os("TEAHOUSE_RELAY_BIN").unwrap_or_else(|| "teahouse-relay".into());
            match teahouse_client::seed::seed(
                std::path::Path::new(fixtures),
                std::path::Path::new(relay),
                std::path::Path::new(people),
                std::path::Path::new(&bin),
            ) {
                Ok(lines) => {
                    for l in lines {
                        println!("{l}");
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("teahouse-client: dev-seed: {}", e.message);
                    for r in e.reasons {
                        eprintln!("  {r}");
                    }
                    ExitCode::FAILURE
                }
            }
        }
        ["mcp"] => {
            let client = Client::new(root);
            let stdin = std::io::stdin();
            match teahouse_client::mcp::serve(&client, stdin.lock(), std::io::stdout().lock()) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("teahouse-client: mcp: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        ["key"] => match Client::new(root).identity() {
            Ok(id) => {
                println!("{}", id.public_line());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("teahouse-client: {}", e.message);
                ExitCode::FAILURE
            }
        },
        ["serve", rest @ ..] => {
            let mut opts = serve::Options {
                port: 7377,
                sync_every: Some(Duration::from_secs(60)),
            };
            let mut it = rest.iter();
            while let Some(flag) = it.next() {
                let value = it.next();
                match (*flag, value) {
                    ("--port", Some(v)) => match v.parse() {
                        Ok(p) => opts.port = p,
                        Err(_) => return usage(),
                    },
                    ("--sync-every", Some(&"off")) => opts.sync_every = None,
                    ("--sync-every", Some(v)) => match v.parse::<u64>() {
                        Ok(n) if n > 0 => opts.sync_every = Some(Duration::from_secs(n)),
                        _ => return usage(),
                    },
                    _ => return usage(),
                }
            }
            match serve::serve(Client::new(root), opts) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("teahouse-client: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        _ => usage(),
    }
}
