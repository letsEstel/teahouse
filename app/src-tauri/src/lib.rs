//! The Tauri shell of Teahouse: a window around the interface (app/src) and,
//! behind it, the engine (client/, `teahouse-client`).
//!
//! The interface calls `invoke('<command>', args)` with the command names of
//! docs/CLIENT-API.md §4. One raw invoke handler hands every such call to
//! `Client::call`, which answers with the §4 envelope (`{ok, data}` or
//! `{ok: false, error}`); the shell passes it through unchanged. Git work can
//! take a while, so each call runs on its own thread instead of the UI loop.

use std::path::PathBuf;
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
