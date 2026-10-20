//! `teahouse-client mcp`: an MCP server on stdio (docs/MCP.md §3), so an AI
//! such as Claude Code can sit in a teahouse as a member.
//!
//! It is one more interface to the same engine: the same local clones, the
//! same device key, the same commands and JSON as CLIENT-API.md §4.
//! `kind = "ai"` is a label, not a rank (法第 17 条): the tools offered follow
//! the seats this AI holds. Every member gets the guest's tools; a steward
//! also `setStatus` and `removeNote`; a host also `adopt`, `decline`,
//! `seat`, `invite` and `purgeBlob`. The server adds no limits of its own:
//! the engine and the relay's laws decide, per teahouse, as for anyone.
//!
//! Transport: JSON-RPC 2.0, one message per line on stdin/stdout; nothing
//! else is ever written to stdout.

use crate::Client;
use serde_json::{json, Value};
use std::io::{BufRead, Write};

/// Protocol versions this server speaks, newest first.
const VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "\
你坐在一间茶馆里（Teahouse）：不超过 15 人的小团体在这里把事情办成。你是 AI 成员，座位表上标着 kind = ai。
You sit in a teahouse: a small group (at most 15) getting things done. You are an AI member, marked kind = ai on the seating chart.

规矩 · The rules:
1. 每个提交都面向一个问题。先用 house 看有哪些问题，用 problem 读全文，再动手。
   Everything you write faces a problem. Read `house`, then `problem`, before writing.
2. 留言写下后不能修改，也不能删除；想改口就再写一条。Posts are never edited; write a new one.
3. 你能做什么，看你的座位（houses 里的 myRole），和人一样：茶客写的成果、新问题、公告会变成提案，由老板决定是否采纳（返回 becameProposal: true）；伙计直接写，也管状态和删留言；老板还管采纳、座位和邀请。
   What you may do follows your seat (myRole in `houses`), as for people: a guest's works, problems and notices become proposals; a steward writes directly and moderates; a host also adopts, seats and invites.
4. 不要假装是人。你的名字旁边会显示 AI，你的每个提交都用你自己的钥匙签名。
   Never pretend to be a person: you are shown as AI and sign everything with your own key.
5. 先 sync 再读，写完的结果里会告诉你是否已经推送（commit: pushed / local）。
   `sync` before reading; write results say whether they were pushed.
6. 有权限的事，也要像人一样谨慎：采纳、拒绝、改座位、删留言都会留在历史里。拿不准就先留言问。
   Use the powers your seat gives you as a careful person would; they stay in history. When unsure, ask in a post first.";

fn obj(props: Value, required: &[&str]) -> Value {
    json!({ "type": "object", "properties": props, "required": required, "additionalProperties": false })
}

fn s(desc: &str) -> Value {
    json!({ "type": "string", "description": desc })
}

const HOUSE: &str = "teahouse id (from `houses`)";

/// The tools: name, description, input schema.
fn tools() -> Vec<(&'static str, &'static str, Value)> {
    let house = || s(HOUSE);
    let section = json!({
        "type": "object",
        "properties": { "heading": s("section heading; empty for the summary before the first heading"), "text": s("section text") },
        "required": ["text"]
    });
    let kind = json!({ "type": "string", "enum": ["doc", "plan", "table", "image", "file"], "description": "doc/plan: markdown text; table: CSV text; image/file: fromPath" });
    vec![
        ("houses", "The teahouses this device sits in, with your member id, role and sync state. Start here.", obj(json!({}), &[])),
        ("house", "One teahouse: members (kind ai marks AI members), problems, recent events, open proposals, decisions, notices.", obj(json!({ "house": house() }), &["house"])),
        ("problem", "One problem in full: summary, sections, works (results), events, and the discussion in the order it happened. Read this before writing about a problem.", obj(json!({ "house": house(), "problem": s("4-digit problem id, e.g. 0001") }), &["house", "problem"])),
        ("archive", "The teahouse's files as a tree, by directory or by problem.", obj(json!({ "house": house(), "view": { "type": "string", "enum": ["dirs", "problems"] } }), &["house", "view"])),
        ("file", "One file by its path (the `id` from archive or works): its text and its history with diffs.", obj(json!({ "house": house(), "id": s("repository path") }), &["house", "id"])),
        ("note", "Post a message under a problem (or reply to a notice). Posts are permanent: they are never edited or removed by you. Give exactly one of problem or notice.", obj(json!({ "house": house(), "problem": s("4-digit problem id"), "notice": s("notice id, to reply to a notice"), "text": s("plain text, at most 32 KiB") }), &["house", "text"])),
        ("propose", "Propose changes to a problem's works for the host to adopt or decline. Changes: new files (kind, title, text) or edits/deletions of existing ones (id + text, or id + delete). The title says what the proposal does; the reason says why.", obj(json!({
            "house": house(),
            "problem": s("4-digit problem id, or \"board\""),
            "title": s("what this proposal does, e.g. 修改《术语对照表》"),
            "reason": s("why"),
            "changes": { "type": "array", "items": { "type": "object", "properties": {
                "id": s("path of an existing file to change or delete"),
                "kind": kind.clone(),
                "title": s("title of a new file"),
                "text": s("the new content"),
                "fromPath": s("a local file to attach (image/file)"),
                "delete": { "type": "boolean" }
            } } }
        }), &["house", "problem", "title", "changes"])),
        ("withdraw", "Withdraw one of your own open proposals.", obj(json!({ "house": house(), "proposal": s("proposal id, <your-id>/<name>") }), &["house", "proposal"])),
        ("openProblem", "Raise a new problem. As a guest this becomes a proposal for the host (becameProposal: true).", obj(json!({ "house": house(), "title": s("the problem in one line"), "body": { "type": "array", "items": section } }), &["house", "title"])),
        ("addWork", "Add a result (work) to a problem: a document, plan, table or attachment. As a guest this becomes a proposal.", obj(json!({ "house": house(), "problem": s("4-digit problem id"), "kind": kind, "title": s("title"), "text": s("content for doc/plan/table"), "fromPath": s("local file for image/file"), "note": s("one line shown under the title") }), &["house", "problem", "kind", "title"])),
        ("updateWork", "Change an existing work's content (by its id). As a guest this becomes a proposal.", obj(json!({ "house": house(), "id": s("the work's path"), "text": s("the new content"), "fromPath": s("a local file, for attachments") }), &["house", "id"])),
        ("updateProblem", "Change a problem's title and/or its text (the Markdown after the frontmatter: summary and ## sections); status and owner stay. As a guest this becomes a proposal.", obj(json!({ "house": house(), "problem": s("4-digit problem id"), "title": s("the new title"), "text": s("the new text") }), &["house", "problem"])),
        ("sync", "Fetch what others wrote and push what you wrote. Without house: every teahouse.", obj(json!({ "house": house() }), &[])),
        ("pair", "Make a pairing code (TH-…@relay, 10 minutes, once) so another device of yours (or another machine you run on) joins as the same member, in every teahouse you sit in there.", obj(json!({ "house": s("any teahouse you sit in at that relay (default: the first)"), "name": s("a name for the new device, e.g. 手机") }), &[])),
        ("devices", "Your devices in a teahouse: name, SHA256 fingerprint, when added, and which one this is.", obj(json!({ "house": house() }), &["house"])),
        ("unpair", "Remove another of your own devices (e.g. a lost one), by name or SHA256 fingerprint; without house, everywhere you have it. Not the device in use, never the last one.", obj(json!({ "house": house(), "device": s("device name or SHA256:… fingerprint") }), &["device"])),
        // A steward's and a host's, offered when this device holds such a seat.
        ("setStatus", "Change a problem's status (open, active, solved) and its owner. Steward and host.", obj(json!({ "house": house(), "problem": s("4-digit problem id"), "status": { "type": "string", "enum": ["open", "active", "solved"] }, "owner": s("member id who takes it on; empty to clear") }), &["house", "problem", "status"])),
        ("removeNote", "Remove a post (moderation): it leaves main, but stays in history. Steward and host.", obj(json!({ "house": house(), "id": s("the post's path (Note.id)") }), &["house", "id"])),
        ("adopt", "Adopt a proposal: merge it into main and close it. Host.", obj(json!({ "house": house(), "proposal": s("proposal id, <author>/<name>") }), &["house", "proposal"])),
        ("decline", "Decline a proposal with a reason; it is closed and the decision stays in history. Host.", obj(json!({ "house": house(), "proposal": s("proposal id"), "reason": s("why, for its author") }), &["house", "proposal"])),
        ("seat", "Change the seating chart: remove a member, or change a member's role. New members come only by invitation. Host.", obj(json!({ "house": house(), "remove": s("member id to remove"), "role": { "type": "object", "properties": { "id": s("member id"), "role": { "type": "string", "enum": ["guest", "steward", "host"] } }, "required": ["id", "role"] } }), &["house"])),
        ("invite", "Create a one-time invitation (TH-…@relay) for a person or an AI. Host.", obj(json!({ "house": house(), "role": { "type": "string", "enum": ["guest", "steward"] }, "name": s("whom it is for"), "days": { "type": "integer", "minimum": 1, "maximum": 30 }, "kind": { "type": "string", "enum": ["human", "ai"] }, "agent": s("for an AI: which one") }), &["house"])),
        ("purgeBlob", "Remove an attachment's content for good (the reference stays, marked purged). Host.", obj(json!({ "house": house(), "id": s("the attachment reference's path") }), &["house", "id"])),
        ("redeem", "Take a seat with an invitation (TH-XXXX-XXXX-XXXX-XXXX@relay) from the host, then clone the teahouse. Give your name as people should see it; agent names what you are (e.g. Claude Code).", obj(json!({ "code": s("the whole invitation as pasted"), "name": s("your name in this teahouse"), "agent": s("what kind of AI you are"), "relay": s("relay address, only if the invitation has none") }), &["code", "name"])),
    ]
}

/// The seat a tool needs somewhere for it to be offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Need {
    Anyone,
    Steward,
    Host,
}

fn need(tool: &str) -> Need {
    match tool {
        "setStatus" | "removeNote" => Need::Steward,
        "adopt" | "decline" | "seat" | "invite" | "purgeBlob" => Need::Host,
        _ => Need::Anyone,
    }
}

/// The highest seat this device holds in any teahouse.
fn held(client: &Client) -> Need {
    let mut best = Need::Anyone;
    for h in client.houses().unwrap_or_default() {
        if h.me.is_empty() {
            continue;
        }
        let n = match h.my_role {
            "host" => Need::Host,
            "steward" => Need::Steward,
            _ => Need::Anyone,
        };
        best = best.max(n);
    }
    best
}

/// The tools this device's seats call for.
fn offered(client: &Client) -> Vec<(&'static str, &'static str, Value)> {
    let held = held(client);
    tools()
        .into_iter()
        .filter(|(name, _, _)| need(name) <= held)
        .collect()
}

/// What a tool call answers: the engine's envelope as text, and as
/// structured content when it is an object. No limits of its own: the
/// engine and the relay decide, for this teahouse and this seat.
fn call_tool(client: &Client, name: &str, args: Value) -> Value {
    if !tools().iter().any(|(n, _, _)| *n == name) {
        return json!({
            "content": [{ "type": "text", "text": format!("unknown tool {name}") }],
            "isError": true
        });
    }
    let args = if args.is_null() { json!({}) } else { args };
    let out = client.call(name, args);
    let ok = out["ok"] == true;
    let payload = if ok {
        out["data"].clone()
    } else {
        out["error"].clone()
    };
    let text = serde_json::to_string_pretty(&payload).unwrap_or_default();
    let mut result = json!({ "content": [{ "type": "text", "text": text }], "isError": !ok });
    if ok && payload.is_object() {
        result["structuredContent"] = payload;
    }
    result
}

fn resources(client: &Client) -> Value {
    let mut list = Vec::new();
    for id in client.house_ids().unwrap_or_default() {
        let Ok(snap) = client.snapshot(&id) else {
            continue;
        };
        let Ok(h) = snap.house() else { continue };
        list.push(json!({
            "uri": format!("teahouse://{id}"),
            "name": h.name,
            "description": "the teahouse: members, problems, proposals, notices",
            "mimeType": "application/json"
        }));
        for p in h.problems {
            list.push(json!({
                "uri": format!("teahouse://{id}/problem/{}", p.id),
                "name": format!("{} · {}", p.id, p.title),
                "description": p.summary,
                "mimeType": "application/json"
            }));
        }
    }
    json!({ "resources": list })
}

fn read_resource(client: &Client, uri: &str) -> Result<Value, String> {
    let rest = uri
        .strip_prefix("teahouse://")
        .ok_or("not a teahouse:// URI")?;
    let out = match rest.split('/').collect::<Vec<_>>().as_slice() {
        [house] => client.call("house", json!({ "house": house })),
        [house, "problem", id] => client.call("problem", json!({ "house": house, "problem": id })),
        _ => return Err(format!("unknown resource {uri}")),
    };
    if out["ok"] != true {
        return Err(out["error"]["message"]
            .as_str()
            .unwrap_or("error")
            .to_string());
    }
    let text = serde_json::to_string_pretty(&out["data"]).unwrap_or_default();
    Ok(json!({ "contents": [{ "uri": uri, "mimeType": "application/json", "text": text }] }))
}

/// Answers one request; `None` for notifications.
pub fn handle(client: &Client, msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned();
    let method = msg["method"].as_str().unwrap_or("");
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    let id = id?; // notifications (no id) get no answer
    let result: Result<Value, (i64, String)> = match method {
        "initialize" => {
            let asked = params["protocolVersion"].as_str().unwrap_or("");
            let version = VERSIONS
                .iter()
                .find(|v| **v == asked)
                .copied()
                .unwrap_or(VERSIONS[0]);
            Ok(json!({
                "protocolVersion": version,
                "capabilities": { "tools": { "listChanged": true }, "resources": { "listChanged": false } },
                "serverInfo": { "name": "teahouse", "title": "茶馆 · Teahouse", "version": env!("CARGO_PKG_VERSION") },
                "instructions": INSTRUCTIONS
            }))
        }
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({
            "tools": offered(client).into_iter().map(|(name, description, schema)| json!({
                "name": name, "description": description, "inputSchema": schema
            })).collect::<Vec<_>>()
        })),
        "tools/call" => match params["name"].as_str() {
            Some(name) => Ok(call_tool(client, name, params["arguments"].clone())),
            None => Err((-32602, "tools/call needs a name".into())),
        },
        "resources/list" => Ok(resources(client)),
        "resources/templates/list" => Ok(json!({ "resourceTemplates": [
            { "uriTemplate": "teahouse://{house}", "name": "a teahouse", "mimeType": "application/json" },
            { "uriTemplate": "teahouse://{house}/problem/{id}", "name": "a problem in full", "mimeType": "application/json" }
        ] })),
        "resources/read" => match params["uri"].as_str() {
            Some(uri) => read_resource(client, uri).map_err(|e| (-32002, e)),
            None => Err((-32602, "resources/read needs a uri".into())),
        },
        other => Err((-32601, format!("method not found: {other}"))),
    };
    Some(match result {
        Ok(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
        Err((code, message)) => {
            json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
        }
    })
}

/// Serves MCP until the input ends. When a call changes the seats this
/// device holds (`redeem`, a `sync` that brings a new role), the client is
/// told to list the tools again.
pub fn serve(client: &Client, input: impl BufRead, mut output: impl Write) -> std::io::Result<()> {
    let mut seats = held(client);
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(Value::Array(batch)) => {
                let answers: Vec<Value> = batch.iter().filter_map(|m| handle(client, m)).collect();
                (!answers.is_empty()).then_some(Value::Array(answers))
            }
            Ok(msg) => handle(client, &msg),
            Err(e) => Some(
                json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": format!("parse error: {e}") } }),
            ),
        };
        if let Some(r) = reply {
            writeln!(output, "{r}")?;
            output.flush()?;
        }
        if line.contains("\"tools/call\"") {
            let now = held(client);
            if now != seats {
                seats = now;
                let note =
                    json!({ "jsonrpc": "2.0", "method": "notifications/tools/list_changed" });
                writeln!(output, "{note}")?;
                output.flush()?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol() {
        let c = Client::new(std::env::temp_dir().join("teahouse-mcp-unit-nonexistent"));
        let init = handle(&c, &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-03-26" } })).unwrap();
        assert_eq!(init["result"]["protocolVersion"], "2025-03-26");
        assert!(init["result"]["instructions"]
            .as_str()
            .unwrap()
            .contains("kind = ai"));
        let init = handle(&c, &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "1999-01-01" } })).unwrap();
        assert_eq!(init["result"]["protocolVersion"], VERSIONS[0]);
        assert!(handle(
            &c,
            &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })
        )
        .is_none());
        let names: Vec<String> = handle(
            &c,
            &json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
        )
        .unwrap()["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();
        for absent in [
            "adopt",
            "decline",
            "seat",
            "invite",
            "purgeBlob",
            "setStatus",
            "removeNote",
        ] {
            assert!(!names.iter().any(|n| n == absent), "{absent}");
        }
        // No seat anywhere: the guest's tools only (of 24 in all).
        assert_eq!(names.len(), 17);
        assert_eq!(tools().len(), 24);
        let r = handle(&c, &json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": { "name": "adopt", "arguments": {} } })).unwrap();
        assert_eq!(r["result"]["isError"], true);
        let r = handle(&c, &json!({ "jsonrpc": "2.0", "id": 4, "method": "nope" })).unwrap();
        assert_eq!(r["error"]["code"], -32601);
        let r = handle(&c, &json!({ "jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": { "name": "houses" } })).unwrap();
        assert_eq!(
            (
                r["result"]["isError"].clone(),
                r["result"]["content"][0]["text"].as_str().unwrap()
            ),
            (json!(false), "[]")
        );
    }
}
