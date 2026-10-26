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
//!
//! `export` without a `to` ("take the whole teahouse away") goes to the
//! Downloads folder, under `Teahouse/`: the interface cannot pick a folder on
//! a phone, and Downloads is where people look. On Android that is the shared
//! `Download/` (an app may create its own files there without a permission).
//!
//! Background sync (CLIENT-API.md §4): while the app is in the foreground a
//! thread syncs every teahouse every 60 s, backing off to at most 10 minutes
//! after failures (as `teahouse-client serve` does). The interface reports
//! foreground and background with the shell command `syncLoop {active}` (from
//! `visibilitychange`); turning active also syncs at once (start-up, back
//! from the background). When a sync brings anything new, the shell emits
//! `teahouse://changed` with the teahouse ids and the interface reloads.
//! Commands and syncs take turns behind one lock, so a sync never runs under
//! a write.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::ipc::InvokeBody;
use tauri::{AppHandle, Emitter, Manager};
use teahouse_client::Client;

/// `<app data>/teahouse` (CLIENT-API.md §2): houses/, blobs/, drafts/, device.
struct Root(PathBuf);

/// One engine command or background sync at a time (git work on one copy).
struct EngineLock(Arc<Mutex<()>>);

const SYNC_EVERY: Duration = Duration::from_secs(60);
const MAX_BACKOFF: Duration = Duration::from_secs(10 * 60);

/// The background sync: `active` while the app is in the foreground; `kick`
/// asks for a sync now (start-up, back from the background).
#[derive(Default)]
struct SyncState {
    active: bool,
    kick: bool,
}
struct Syncer(Arc<(Mutex<SyncState>, Condvar)>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let root = app.path().app_data_dir()?.join("teahouse");
            std::fs::create_dir_all(&root)?;
            let lock = Arc::new(Mutex::new(()));
            let syncer = Arc::new((Mutex::new(SyncState::default()), Condvar::new()));
            spawn_sync_loop(app.handle().clone(), root.clone(), lock.clone(), syncer.clone());
            app.manage(Root(root));
            app.manage(EngineLock(lock));
            app.manage(Syncer(syncer));
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
            if invoke.message.command() == "syncLoop" {
                let active = matches!(invoke.message.payload(), InvokeBody::Json(v) if v.get("active").and_then(|a| a.as_bool()) == Some(true));
                let syncer = invoke.message.webview().state::<Syncer>().0.clone();
                let (state, wake) = &*syncer;
                let mut s = state.lock().unwrap_or_else(|p| p.into_inner());
                if active && !s.active {
                    s.kick = true;
                }
                s.active = active;
                wake.notify_all();
                invoke.resolver.resolve(serde_json::json!({ "ok": true, "data": null }));
                return true;
            }
            let root = invoke.message.webview().state::<Root>().0.clone();
            let lock = invoke.message.webview().state::<EngineLock>().0.clone();
            let command = invoke.message.command().to_string();
            let mut args = match invoke.message.payload() {
                InvokeBody::Json(v) => v.clone(),
                InvokeBody::Raw(_) => serde_json::Value::Null,
            };
            if command == "export" && args.get("to").and_then(|t| t.as_str()).is_none_or(|t| t.trim().is_empty()) {
                if let (Some(dir), Some(obj)) = (export_dir(invoke.message.webview().app_handle()), args.as_object_mut()) {
                    obj.insert("to".into(), dir.to_string_lossy().into_owned().into());
                }
            }
            let resolver = invoke.resolver;
            std::thread::spawn(move || {
                let _turn = lock.lock().unwrap_or_else(|p| p.into_inner());
                resolver.resolve(Client::new(root).call(&command, args));
            });
            true
        })
        .run(tauri::generate_context!())
        .expect("error while running the Teahouse app");
}

/// Where `export` puts a teahouse when the interface gives no folder.
fn export_dir(app: &AppHandle) -> Option<PathBuf> {
    #[cfg(target_os = "android")]
    let downloads = Some(PathBuf::from("/storage/emulated/0/Download"));
    #[cfg(not(target_os = "android"))]
    let downloads = app.path().download_dir().ok().or_else(|| app.path().document_dir().ok());
    #[cfg(target_os = "android")]
    let _ = app;
    downloads.map(|d| d.join("Teahouse"))
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

/// Every 60 s while active (or at once when kicked): sync each teahouse and
/// tell the interface which ones changed. Failures back off up to 10 minutes.
fn spawn_sync_loop(app: AppHandle, root: PathBuf, lock: Arc<Mutex<()>>, syncer: Arc<(Mutex<SyncState>, Condvar)>) {
    std::thread::spawn(move || {
        let mut wait = SYNC_EVERY;
        loop {
            {
                let (state, wake) = &*syncer;
                let mut s = state.lock().unwrap_or_else(|p| p.into_inner());
                // In the background: sleep until the app comes back.
                while !s.active {
                    s = wake.wait(s).unwrap_or_else(|p| p.into_inner());
                }
                if !s.kick {
                    let (next, timeout) = wake.wait_timeout(s, wait).unwrap_or_else(|p| p.into_inner());
                    s = next;
                    // Woken early by anything but a kick (e.g. going to the background): look again.
                    if !timeout.timed_out() && !s.kick {
                        continue;
                    }
                    if !s.active {
                        continue;
                    }
                }
                if s.kick {
                    wait = SYNC_EVERY;
                }
                s.kick = false;
            }
            let (ok, changed) = {
                let _turn = lock.lock().unwrap_or_else(|p| p.into_inner());
                sync_all(&Client::new(root.clone()))
            };
            if !changed.is_empty() {
                let _ = app.emit("teahouse://changed", &changed);
            }
            wait = if ok { SYNC_EVERY } else { (wait * 2).clamp(SYNC_EVERY, MAX_BACKOFF) };
        }
    });
}

/// Syncs every teahouse; returns whether all went through and which changed.
/// "Changed" compares what the interface reads (the teahouse and the list of
/// teahouses) before and after, so a new seat, note or proposal all count.
fn sync_all(c: &Client) -> (bool, Vec<String>) {
    let ids = c.house_ids().unwrap_or_default();
    let mut ok = true;
    let mut changed = Vec::new();
    for h in &ids {
        let before = fingerprint(c, h);
        let s = c.call("sync", serde_json::json!({ "house": h }));
        let state = s.pointer("/data/state").and_then(|v| v.as_str()).unwrap_or("error");
        if !s.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) || state == "offline" || state == "error" {
            ok = false;
        }
        if fingerprint(c, h) != before {
            changed.push(h.clone());
        }
    }
    (ok, changed)
}

fn fingerprint(c: &Client, house: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    // The sync state (last-sync time) changes on every sync; leave it out.
    let mut h = c.call("house", serde_json::json!({ "house": house }));
    if let Some(d) = h.get_mut("data").and_then(|d| d.as_object_mut()) {
        d.remove("sync");
    }
    h.to_string().hash(&mut hasher);
    let mut list = c.call("houses", serde_json::json!({}));
    if let Some(a) = list.get_mut("data").and_then(|d| d.as_array_mut()) {
        for x in a.iter_mut().filter_map(|x| x.as_object_mut()) {
            x.remove("sync");
        }
    }
    list.to_string().hash(&mut hasher);
    hasher.finish()
}
