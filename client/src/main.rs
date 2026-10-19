//! `teahouse-client`: the engine from the command line.
//!
//! ```text
//! teahouse-client [--root <dir>] call <command> [<json-args>]
//! ```
//!
//! Prints the command's JSON envelope; exits 1 when it is `{ok: false}`.
//! The HTTP interface (`serve`) comes with step 2 of CLIENT-API.md §7.

use std::path::PathBuf;
use std::process::ExitCode;
use teahouse_client::Client;

const USAGE: &str = "usage: teahouse-client [--root <dir>] call <command> [<json-args>]";

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

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut root = None;
    if args.first().map(String::as_str) == Some("--root") {
        if args.len() < 2 {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
        root = Some(PathBuf::from(args.remove(1)));
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
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
