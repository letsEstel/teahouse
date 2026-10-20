//! The Tauri shell of Teahouse: a window around the interface (app/src) and,
//! behind it, the engine (client/, `teahouse-client`).
//!
//! The interface calls `invoke('<command>', args)` with the command names of
//! docs/CLIENT-API.md §4. One raw invoke handler hands every such call to
//! `Client::call`, which answers with the §4 envelope (`{ok, data}` or
//! `{ok: false, error}`); the shell passes it through unchanged. Git work can
//! take a while, so each call runs on its own thread instead of the UI loop.
//!
//! One command belongs to the shell itself: `stageFile`. A file picked on a
//! phone arrives as bytes (raw invoke body, name in `x-file-name`), but the
//! engine attaches files by path (`fromPath`); the shell saves the bytes under
//! the app's cache directory and answers `{ok, data: {path}}`.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::ipc::InvokeBody;
use tauri::Manager;
use teahouse_client::Client;

/// `<app data>/teahouse` (CLIENT-API.md §2): houses/, blobs/, drafts/, device.
struct Root(PathBuf);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let root = app.path().app_data_dir()?.join("teahouse");
            std::fs::create_dir_all(&root)?;
            app.manage(Root(root));
            Ok(())
        })
        .invoke_handler(|invoke| {
            if invoke.message.command() == "stageFile" {
                let staged = invoke.message.webview().path().app_cache_dir().map_err(|e| e.to_string()).and_then(|dir| {
                    let name = invoke
                        .message
                        .headers()
                        .get("x-file-name")
                        .and_then(|v| v.to_str().ok())
                        .map(percent_decode)
                        .unwrap_or_default();
                    match invoke.message.payload() {
                        InvokeBody::Raw(bytes) => stage(&dir.join("staged"), &name, bytes),
                        InvokeBody::Json(_) => Err("stageFile expects the file's bytes".into()),
                    }
                });
                invoke.resolver.resolve(match staged {
                    Ok(path) => serde_json::json!({ "ok": true, "data": { "path": path } }),
                    Err(message) => serde_json::json!({ "ok": false, "error": { "code": "internal", "message": message } }),
                });
                return true;
            }
            let root = invoke.message.webview().state::<Root>().0.clone();
            let command = invoke.message.command().to_string();
            let args = match invoke.message.payload() {
                InvokeBody::Json(v) => v.clone(),
                InvokeBody::Raw(_) => serde_json::Value::Null,
            };
            let resolver = invoke.resolver;
            std::thread::spawn(move || {
                resolver.resolve(Client::new(root).call(&command, args));
            });
            true
        })
        .run(tauri::generate_context!())
        .expect("error while running the Teahouse app");
}

/// Save a picked file as `<dir>/<time>-<name>` and return its path. Only the
/// last path component of `name` is kept, so a name cannot leave `dir`.
fn stage(dir: &Path, name: &str, bytes: &[u8]) -> Result<String, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let base = Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty() && *n != "." && *n != "..")
        .unwrap_or("file");
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    let path = dir.join(format!("{stamp}-{base}"));
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    path.into_os_string().into_string().map_err(|_| "path is not UTF-8".to_string())
}

/// `encodeURIComponent` undone (the header carries the file name).
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Some(v) = std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
