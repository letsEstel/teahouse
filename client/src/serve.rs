//! `serve`: the development HTTP interface (CLIENT-API.md §4): the same
//! commands as Tauri's `invoke`, as `POST http://127.0.0.1:<port>/v1/<command>`
//! with JSON in and the `{ok, data|error}` envelope out. It also syncs every
//! teahouse in the background.
//!
//! Any web page can send requests to 127.0.0.1, so a request is refused
//! unless its `Host` is this server (no DNS rebinding) and its `Origin`,
//! when there is one, is a page served from this machine (Vite's dev server).
//! Commands need `Content-Type: application/json`, which a foreign page can
//! only send after a CORS preflight that we refuse.

use crate::Client;
use serde_json::{json, Value};
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tiny_http::{Header, Method, Request, Response, Server};

/// Request bodies above this are refused (a table's CSV is the largest
/// thing sent inline; attachments go by path).
const MAX_BODY: u64 = 16 * 1024 * 1024;
/// Background sync backs off up to this after failures.
const MAX_BACKOFF: Duration = Duration::from_secs(10 * 60);

pub struct Options {
    pub port: u16,
    /// `None`: no background sync.
    pub sync_every: Option<Duration>,
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid header")
}

fn get<'r>(req: &'r Request, name: &str) -> Option<&'r str> {
    req.headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str())
}

/// `http://localhost:5173`, `http://127.0.0.1:7377`, …
fn local_origin(origin: &str) -> bool {
    let Some(rest) = origin.strip_prefix("http://") else {
        return false;
    };
    let host = rest.rsplit_once(':').map_or(rest, |(h, p)| {
        if p.bytes().all(|b| b.is_ascii_digit()) {
            h
        } else {
            rest
        }
    });
    matches!(host, "localhost" | "127.0.0.1" | "[::1]")
}

fn host_ok(host: Option<&str>, port: u16) -> bool {
    matches!(host, Some(h) if h == format!("127.0.0.1:{port}") || h == format!("localhost:{port}"))
}

fn json_response(
    v: &Value,
    status: u16,
    origin: Option<&str>,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut r = Response::from_data(serde_json::to_vec(v).expect("json"))
        .with_status_code(status)
        .with_header(header("Content-Type", "application/json; charset=utf-8"))
        .with_header(header("Cache-Control", "no-store"));
    if let Some(o) = origin {
        r = r
            .with_header(header("Access-Control-Allow-Origin", o))
            .with_header(header("Vary", "Origin"));
    }
    r
}

fn refuse(req: Request, status: u16, why: &str) {
    let v = json!({ "ok": false, "error": { "code": "forbidden", "message": why } });
    let _ = req.respond(json_response(&v, status, None));
}

/// Handles one request; `call` runs a command.
/// `a%20b` → `a b`; `+` is a space. `None` for malformed escapes or
/// non-UTF-8 results.
fn percent_decode(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' => {
                let hex = std::str::from_utf8(b.get(i + 1..i + 3)?).ok()?;
                out.push(u8::from_str_radix(hex, 16).ok()?);
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

/// The value of `key` in a query string.
fn query_param(query: &str, key: &str) -> Option<String> {
    query
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .find(|(k, _)| *k == key)
        .and_then(|(_, v)| percent_decode(v))
}

/// Looks up an attachment's cached file and media type.
type BlobLookup<'a> = &'a dyn Fn(&str, &str) -> crate::error::Result<(std::path::PathBuf, String)>;

fn handle(mut req: Request, port: u16, call: &dyn Fn(&str, Value) -> Value, blob: BlobLookup<'_>) {
    if !host_ok(get(&req, "Host"), port) {
        return refuse(req, 403, "wrong Host");
    }
    let origin = get(&req, "Origin").map(str::to_string);
    if let Some(o) = &origin {
        if !local_origin(o) {
            return refuse(req, 403, "requests from other sites are refused");
        }
    }
    let (url, query) = match req.url().split_once('?') {
        Some((u, q)) => (u.to_string(), q.to_string()),
        None => (req.url().to_string(), String::new()),
    };
    match (req.method().clone(), url.as_str()) {
        (Method::Options, _) => {
            let mut r = Response::empty(204)
                .with_header(header("Access-Control-Allow-Methods", "POST, GET"))
                .with_header(header("Access-Control-Allow-Headers", "Content-Type"))
                .with_header(header("Access-Control-Max-Age", "600"));
            if let Some(o) = &origin {
                r = r.with_header(header("Access-Control-Allow-Origin", o));
            }
            let _ = req.respond(r);
        }
        (Method::Get, "/v1/health") => {
            let _ = req.respond(json_response(
                &json!({ "ok": true }),
                200,
                origin.as_deref(),
            ));
        }
        // Read-only: a file from this engine's attachment cache, for <img>
        // in the development interface (Tauri uses convertFileSrc).
        (Method::Get, "/v1/blob") => {
            let (Some(house), Some(id)) = (query_param(&query, "house"), query_param(&query, "id"))
            else {
                let v = json!({ "ok": false, "error": { "code": "invalid", "message": "need house and id" } });
                let _ = req.respond(json_response(&v, 400, origin.as_deref()));
                return;
            };
            match blob(&house, &id).and_then(|(path, mime)| Ok((std::fs::File::open(path)?, mime)))
            {
                Ok((file, mime)) => {
                    let mut r = Response::from_file(file)
                        .with_header(header("Content-Type", &mime))
                        .with_header(header("X-Content-Type-Options", "nosniff"))
                        // An SVG or HTML attachment must not run as a page.
                        .with_header(header(
                            "Content-Security-Policy",
                            "default-src 'none'; style-src 'unsafe-inline'; sandbox",
                        ))
                        .with_header(header("Cache-Control", "private, max-age=3600"));
                    if let Some(o) = &origin {
                        r = r.with_header(header("Access-Control-Allow-Origin", o));
                    }
                    let _ = req.respond(r);
                }
                Err(e) => {
                    let status = if e.code == crate::error::Code::NotFound {
                        404
                    } else {
                        409
                    };
                    let v = json!({ "ok": false, "error": e });
                    let _ = req.respond(json_response(&v, status, origin.as_deref()));
                }
            }
        }
        (Method::Post, path) if path.starts_with("/v1/") => {
            let command = path["/v1/".len()..].to_string();
            let is_json = get(&req, "Content-Type")
                .is_some_and(|c| c.trim_start().starts_with("application/json"));
            if !is_json {
                return refuse(req, 415, "Content-Type must be application/json");
            }
            let mut body = Vec::new();
            if req
                .as_reader()
                .take(MAX_BODY + 1)
                .read_to_end(&mut body)
                .is_err()
                || body.len() as u64 > MAX_BODY
            {
                return refuse(req, 413, "request too large");
            }
            let args = if body.iter().all(u8::is_ascii_whitespace) {
                Value::Null
            } else {
                match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(e) => {
                        let v = json!({ "ok": false, "error": { "code": "invalid", "message": format!("body is not JSON: {e}") } });
                        let _ = req.respond(json_response(&v, 400, origin.as_deref()));
                        return;
                    }
                }
            };
            let out = call(&command, args);
            let _ = req.respond(json_response(&out, 200, origin.as_deref()));
        }
        _ => {
            let v = json!({ "ok": false, "error": { "code": "not_found", "message": "no such endpoint" } });
            let _ = req.respond(json_response(&v, 404, origin.as_deref()));
        }
    }
}

/// Serves until the process ends. Commands and background syncs take turns
/// (one at a time), so a sync never runs under a write.
pub fn serve(client: Client, opts: Options) -> std::io::Result<()> {
    let server = Server::http(("127.0.0.1", opts.port)).map_err(std::io::Error::other)?;
    let port = server.server_addr().to_ip().map_or(opts.port, |a| a.port());
    eprintln!("teahouse-client: serving on http://127.0.0.1:{port}/v1/");
    let client = Arc::new(Mutex::new(client));

    if let Some(every) = opts.sync_every {
        let c = client.clone();
        std::thread::spawn(move || {
            let mut wait = every;
            loop {
                std::thread::sleep(wait);
                let ok = {
                    let c = c.lock().unwrap_or_else(|p| p.into_inner());
                    let ids = c.house_ids().unwrap_or_default();
                    ids.iter().all(|h| {
                        matches!(c.sync_house(h), Ok(s) if !matches!(s.state, crate::model::SyncKind::Offline | crate::model::SyncKind::Error))
                    })
                };
                wait = if ok {
                    every
                } else {
                    (wait * 2).min(MAX_BACKOFF).max(every)
                };
            }
        });
    }

    for req in server.incoming_requests() {
        let call = |cmd: &str, args: Value| {
            let c = client.lock().unwrap_or_else(|p| p.into_inner());
            c.call(cmd, args)
        };
        let blob = |house: &str, id: &str| {
            let c = client.lock().unwrap_or_else(|p| p.into_inner());
            c.snapshot(house)?.blob_file(id)
        };
        handle(req, port, &call, &blob);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries() {
        let q = "house=wutong&id=works%2F0001%2F%E8%BD%A6%E4%BD%8D.png.ref.md&x=a+b";
        assert_eq!(query_param(q, "house").as_deref(), Some("wutong"));
        assert_eq!(
            query_param(q, "id").as_deref(),
            Some("works/0001/车位.png.ref.md")
        );
        assert_eq!(query_param(q, "x").as_deref(), Some("a b"));
        assert_eq!(query_param(q, "y"), None);
        assert_eq!(percent_decode("%E8%BD"), None);
        assert_eq!(percent_decode("%zz"), None);
    }

    #[test]
    fn origins_and_hosts() {
        assert!(local_origin("http://localhost:5173"));
        assert!(local_origin("http://127.0.0.1:7377"));
        assert!(local_origin("http://localhost"));
        assert!(!local_origin("https://localhost:5173"));
        assert!(!local_origin("http://localhost.evil.example:80"));
        assert!(!local_origin("http://evil.example"));
        assert!(!local_origin("null"));
        assert!(host_ok(Some("127.0.0.1:7377"), 7377));
        assert!(host_ok(Some("localhost:7377"), 7377));
        assert!(!host_ok(Some("evil.example:7377"), 7377));
        assert!(!host_ok(None, 7377));
    }
}
