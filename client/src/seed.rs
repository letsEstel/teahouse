//! `dev-seed`: turns `fixtures/houses.zh.json` (exported by the interface,
//! CLIENT-API.md §6) into real teahouses on a local relay, for development.
//!
//! Every member gets an engine of their own (`<people>/<member>/`, with a
//! device key). Each teahouse is opened at the relay with its host, and its
//! history is rebuilt in time order: people are seated on the day they
//! joined, every file is written by the person and at the time its history
//! names, each commit signed by its author's key. The host pushes; because
//! the relay only trusts keys already seated, it pushes after every seat
//! change. Open proposals are then pushed by their authors, and everyone
//! clones every teahouse they sit in.
//!
//! The fixture's `members.toml` and image references are made real: the
//! chart gets real keys, images become small generated PNGs in the
//! attachment store.

use crate::commit::{self, Author, Edit};
use crate::error::{Error, Result};
use crate::format;
use crate::Client;
use git2::{Oid, Repository};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use teahouse_core::{Member, Members, PublicKey, Role, MEMBERS_PATH};

fn epoch(rfc3339: &str) -> Result<i64> {
    // 2026-10-18T16:02:00(.000)Z
    let s = rfc3339.trim();
    let stamp = format!(
        "{}{}{}T{}{}{}Z",
        s.get(0..4).unwrap_or(""),
        s.get(5..7).unwrap_or(""),
        s.get(8..10).unwrap_or(""),
        s.get(11..13).unwrap_or(""),
        s.get(14..16).unwrap_or(""),
        s.get(17..19).unwrap_or("")
    );
    teahouse_core::rules::stamp_epoch(&stamp)
        .ok_or_else(|| Error::invalid(format!("not a time: {rfc3339}")))
}

// -- a small PNG, so images show something -----------------------------------

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in data {
        a = (a + x as u32) % 65_521;
        b = (b + a) % 65_521;
    }
    b << 16 | a
}

/// A `w`×`h` PNG in one colour (uncompressed deflate blocks).
pub fn png(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
    // Each row: filter byte 0, then the pixels.
    let mut row = vec![0u8];
    for _ in 0..w {
        row.extend_from_slice(&rgb);
    }
    let raw = row.repeat(h as usize);
    let mut z = vec![0x78, 0x01];
    let mut chunks = raw.chunks(65_535).peekable();
    while let Some(c) = chunks.next() {
        z.push(u8::from(chunks.peek().is_none()));
        z.extend_from_slice(&(c.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(c.len() as u16)).to_le_bytes());
        z.extend_from_slice(c);
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut chunk = |kind: &[u8], data: &[u8]| {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc32(&body).to_be_bytes());
    };
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(b"IHDR", &ihdr);
    chunk(b"IDAT", &z);
    chunk(b"IEND", &[]);
    out
}

// -- the seeding --------------------------------------------------------------

struct Person {
    client: Client,
    name: String,
    key: PublicKey,
}

/// One thing that happened, in a teahouse's history.
struct Event {
    when: i64,
    /// Seats first among events at the same time.
    order: u8,
    by: String,
    subject: String,
    problem: String,
    act: String,
    body: Option<String>,
    trailers: Vec<(&'static str, String)>,
    edits: Vec<Edit>,
    seat: Option<String>,
}

fn relay_open(
    relay_bin: &Path,
    relay_root: &Path,
    house: &Value,
    host: &Value,
    key: &str,
    founded: &str,
) -> Result<()> {
    let mut cmd = std::process::Command::new(relay_bin);
    cmd.arg("--root")
        .arg(relay_root)
        .args(["open", house["id"].as_str().unwrap_or("")])
        .args(["--name", house["name"].as_str().unwrap_or("")])
        .args(["--host-id", host["id"].as_str().unwrap_or("")])
        .args(["--host-name", host["name"].as_str().unwrap_or("")])
        .args(["--host-key", key])
        .env("GIT_AUTHOR_DATE", founded)
        .env("GIT_COMMITTER_DATE", founded);
    if let Some(s) = house["signboard"].as_str() {
        cmd.args(["--signboard", s]);
    }
    let out = cmd
        .output()
        .map_err(|e| Error::internal(format!("cannot run teahouse-relay: {e}")))?;
    if !out.status.success() {
        return Err(Error::internal(format!(
            "relay open {}: {}",
            house["id"],
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(())
}

fn act_of(history: &Value) -> String {
    history["act"].as_str().unwrap_or("update").to_string()
}

/// Seeds every teahouse of `fixtures`; returns one line per teahouse.
pub fn seed(
    fixtures: &Path,
    relay_root: &Path,
    people_dir: &Path,
    relay_bin: &Path,
) -> Result<Vec<String>> {
    let data: Value = serde_json::from_slice(&std::fs::read(fixtures)?)
        .map_err(|e| Error::invalid(format!("{}: {e}", fixtures.display())))?;
    let relay = format!("teahouse-local:{}", relay_root.display());
    let mut people: BTreeMap<String, Person> = BTreeMap::new();
    let mut report = Vec::new();
    for entry in data["houses"].as_array().cloned().unwrap_or_default() {
        let h = &entry["house"];
        let hid = h["id"].as_str().unwrap_or("").to_string();
        let members = h["members"].as_array().cloned().unwrap_or_default();
        for m in &members {
            let id = m["id"].as_str().unwrap_or("").to_string();
            if let std::collections::btree_map::Entry::Vacant(slot) = people.entry(id.clone()) {
                let client = Client::new(people_dir.join(&id));
                let key = client.identity()?.public();
                slot.insert(Person {
                    client,
                    name: m["name"].as_str().unwrap_or("").to_string(),
                    key,
                });
            }
        }
        let host = members
            .iter()
            .find(|m| m["role"] == "host")
            .ok_or_else(|| Error::invalid(format!("{hid}: no host")))?;
        let host_id = host["id"].as_str().unwrap_or("").to_string();
        let founded = h["founded"].as_str().unwrap_or("2026-01-01T00:00:00Z");
        relay_open(
            relay_bin,
            relay_root,
            h,
            host,
            &people[&host_id].key.openssh(),
            founded,
        )?;
        let hostp = &people[&host_id];
        hostp.client.join(&relay, &hid)?;

        // The events of this teahouse's history.
        let mut events = Vec::new();
        for m in &members {
            if m["role"] == "host" {
                continue;
            }
            let id = m["id"].as_str().unwrap_or("").to_string();
            events.push(Event {
                when: epoch(m["joined"].as_str().unwrap_or(founded))?,
                order: 0,
                by: host_id.clone(),
                subject: format!("{} 入座", m["name"].as_str().unwrap_or("")),
                problem: "meta".into(),
                act: "seat".into(),
                body: None,
                trailers: vec![],
                edits: vec![],
                seat: Some(id),
            });
        }
        let host_blobs = hostp.client.root().join("blobs");
        std::fs::create_dir_all(&host_blobs)?;
        for f in entry["files"].as_array().cloned().unwrap_or_default() {
            let path = f["path"].as_str().unwrap_or("").to_string();
            let kind = f["kind"].as_str().unwrap_or("");
            if kind == "members" {
                continue;
            }
            let history = f["history"].as_array().cloned().unwrap_or_default();
            let mut created = history
                .iter()
                .rev()
                .find(|e| !matches!(e["act"].as_str(), Some("set-status")));
            if created.is_none() {
                created = history.last();
            }
            let Some(first) = created else { continue };
            let mut text = f["text"].as_str().unwrap_or("").to_string();
            if kind == "image" || path.ends_with(".ref.md") {
                // A real attachment in place of the fixture's fake hash.
                let seed = teahouse_core::sha256::hex(path.as_bytes());
                let rgb = [
                    u8::from_str_radix(&seed[0..2], 16).unwrap_or(0) / 2 + 100,
                    u8::from_str_radix(&seed[2..4], 16).unwrap_or(0) / 2 + 100,
                    u8::from_str_radix(&seed[4..6], 16).unwrap_or(0) / 2 + 100,
                ];
                let img = png(320, 200, rgb);
                let hash = teahouse_core::sha256::hex(&img);
                std::fs::write(host_blobs.join(&hash), &img)?;
                let blob = format!("sha256:{hash}");
                let size = img.len().to_string();
                text = format::set_fields(
                    &text,
                    &[
                        ("blob", Some(&blob)),
                        ("size", Some(&size)),
                        ("type", Some("image/png")),
                    ],
                );
            }
            let problem = match f["problem"].as_str() {
                Some(p) if p.len() == 4 => p.to_string(),
                _ if path.starts_with("board/") || path.starts_with("talk/board/") => {
                    "board".into()
                }
                Some(p) => p.to_string(),
                None => "meta".into(),
            };
            let status_change = history.iter().find(|e| e["act"] == "set-status");
            let initial = match (kind, status_change) {
                ("problem", Some(_)) => {
                    format::set_fields(&text, &[("status", Some("open")), ("owner", None)])
                }
                _ => text.clone(),
            };
            let by = first["by"].as_str().unwrap_or(&host_id).to_string();
            let by = if people.contains_key(&by) {
                by
            } else {
                host_id.clone()
            };
            events.push(Event {
                when: epoch(first["when"].as_str().unwrap_or(founded))?,
                order: 1,
                by,
                subject: first["title"].as_str().unwrap_or("").to_string(),
                problem: problem.clone(),
                act: act_of(first),
                body: None,
                trailers: vec![],
                edits: vec![Edit::Put(path.clone(), initial.into_bytes())],
                seat: None,
            });
            if let (Some(e), "problem") = (status_change, kind) {
                let by = e["by"].as_str().unwrap_or(&host_id).to_string();
                events.push(Event {
                    when: epoch(e["when"].as_str().unwrap_or(founded))?,
                    order: 2,
                    by: if people.contains_key(&by) {
                        by
                    } else {
                        host_id.clone()
                    },
                    subject: e["title"].as_str().unwrap_or("").to_string(),
                    problem,
                    act: "set-status".into(),
                    body: None,
                    trailers: e["status"]
                        .as_str()
                        .map(|s| vec![("Status", s.to_string())])
                        .unwrap_or_default(),
                    edits: vec![Edit::Put(path, text.into_bytes())],
                    seat: None,
                });
            }
        }
        for d in h["decided"].as_array().cloned().unwrap_or_default() {
            let adopted = d["result"] == "adopted";
            let title = d["title"].as_str().unwrap_or("");
            let by = d["by"].as_str().unwrap_or("x");
            events.push(Event {
                when: epoch(d["when"].as_str().unwrap_or(founded))?,
                order: 3,
                by: host_id.clone(),
                subject: format!(
                    "{}{title}",
                    if adopted {
                        "采纳提案："
                    } else {
                        "不采纳提案："
                    }
                ),
                problem: "meta".into(),
                act: if adopted { "adopt" } else { "decline" }.into(),
                body: d["reason"].as_str().map(str::to_string),
                trailers: vec![(
                    "Proposal",
                    format!("{by}/{}", d["id"].as_str().unwrap_or("d")),
                )],
                edits: vec![],
                seat: None,
            });
        }
        // The fixture may have someone act before the day it says they
        // joined: seat them in time for their first act.
        let mut first_act: BTreeMap<String, i64> = BTreeMap::new();
        for e in events.iter().filter(|e| e.seat.is_none()) {
            let t = first_act.entry(e.by.clone()).or_insert(e.when);
            *t = (*t).min(e.when);
        }
        for e in events.iter_mut() {
            if let Some(id) = &e.seat {
                if let Some(t) = first_act.get(id) {
                    e.when = e.when.min(*t);
                }
            }
        }
        events.sort_by_key(|e| (e.when, e.order));

        // Replay them in the host's clone; push after each seat change.
        let repo = Repository::open(hostp.client.root().join("houses").join(&hid))?;
        let mut seated: BTreeSet<String> = [host_id.clone()].into();
        for e in &events {
            let head = repo.find_commit(repo.refname_to_id("refs/heads/main")?)?;
            let mut edits = e.edits.clone();
            if let Some(id) = &e.seat {
                let chart = repo.find_blob(head.tree()?.get_path(Path::new(MEMBERS_PATH))?.id())?;
                let mut m = Members::parse(&String::from_utf8_lossy(chart.content()))
                    .map_err(|e| Error::internal(e.to_string()))?;
                let role = members
                    .iter()
                    .find(|x| x["id"] == id.as_str())
                    .and_then(|x| Role::parse(x["role"].as_str().unwrap_or("guest")))
                    .unwrap_or(Role::Guest);
                let p = &people[id];
                m.members.push(Member::human(
                    id.clone(),
                    p.name.clone(),
                    role,
                    p.key.clone(),
                ));
                let toml = m.to_toml().map_err(|e| Error::internal(e.to_string()))?;
                edits.push(Edit::Put(MEMBERS_PATH.into(), toml.into_bytes()));
            }
            // Someone not seated yet (an inconsistent fixture) writes as the host.
            let by = if seated.contains(&e.by) {
                e.by.clone()
            } else {
                host_id.clone()
            };
            let author = &people[&by];
            let sig = Author {
                id: by.clone(),
                name: author.name.clone(),
            }
            .signature(e.when)?;
            let tree = commit::tree_with(&repo, Some(&head.tree()?), &edits)?;
            let mut trailers = vec![("Problem", e.problem.clone()), ("Act", e.act.clone())];
            trailers.extend(e.trailers.iter().cloned());
            let msg = commit::message(&e.subject, e.body.as_deref(), &trailers);
            let identity = author.client.identity()?;
            let c = commit::signed_commit(&repo, &identity, &sig, tree, &[&head], &msg)?;
            commit::move_ref(&repo, "refs/heads/main", Some(head.id()), Some(c), "seed")?;
            if let Some(id) = &e.seat {
                push(&hostp.client, &hid)?;
                seated.insert(id.clone());
            }
        }
        push(&hostp.client, &hid)?;

        // Everyone clones; open proposals come from their authors.
        for m in &members {
            let id = m["id"].as_str().unwrap_or("");
            if id != host_id && seated.contains(id) {
                people[id].client.join(&relay, &hid)?;
            }
        }
        let mut proposals = 0;
        for p in h["proposals"].as_array().cloned().unwrap_or_default() {
            let by = p["by"].as_str().unwrap_or("");
            let Some(author) = people.get(by).filter(|_| seated.contains(by)) else {
                continue;
            };
            propose(author, by, &hid, &p)?;
            proposals += 1;
        }
        // Everyone ends up with everything, proposals included.
        for m in &members {
            let id = m["id"].as_str().unwrap_or("");
            if seated.contains(id) {
                push(&people[id].client, &hid)?;
            }
        }
        report.push(format!(
            "{hid}: {} commits, {} seats, {proposals} proposals",
            events.len(),
            seated.len()
        ));
    }
    Ok(report)
}

fn push(client: &Client, house: &str) -> Result<()> {
    let s = client.sync_house(house)?;
    if s.state != crate::model::SyncKind::Synced {
        return Err(Error::internal(format!(
            "{house}: the relay did not take the seed: {}",
            s.message.unwrap_or_default()
        )));
    }
    Ok(())
}

/// The new text a fixture's diff describes (same and added lines).
fn new_text(file: &Value) -> String {
    let lines: Vec<&str> = file["lines"]
        .as_array()
        .map(|l| {
            l.iter()
                .filter(|x| x["kind"] != "del")
                .filter_map(|x| x["text"].as_str())
                .collect()
        })
        .unwrap_or_default();
    lines.join("\n") + "\n"
}

fn propose(author: &Person, by: &str, house: &str, p: &Value) -> Result<()> {
    let repo = Repository::open(author.client.root().join("houses").join(house))?;
    let head = repo.find_commit(repo.refname_to_id("refs/heads/main")?)?;
    let edits: Vec<Edit> = p["files"]
        .as_array()
        .map(|fs| {
            fs.iter()
                .filter_map(|f| {
                    f["path"]
                        .as_str()
                        .map(|path| Edit::Put(path.to_string(), new_text(f).into_bytes()))
                })
                .collect()
        })
        .unwrap_or_default();
    let when = epoch(p["when"].as_str().unwrap_or("2026-01-01T00:00:00Z"))?;
    let sig = Author {
        id: by.to_string(),
        name: author.name.clone(),
    }
    .signature(when)?;
    let tree = commit::tree_with(&repo, Some(&head.tree()?), &edits)?;
    let title = p["title"].as_str().unwrap_or("提案");
    let msg = commit::message(
        title,
        p["reason"].as_str(),
        &[
            (
                "Problem",
                p["problem"].as_str().unwrap_or("meta").to_string(),
            ),
            ("Act", "propose".into()),
        ],
    );
    let c = commit::signed_commit(
        &repo,
        &author.client.identity()?,
        &sig,
        tree,
        &[&head],
        &msg,
    )?;
    let name = format!(
        "{}-{}",
        crate::write::ascii_slug(title, 24).trim_matches('-'),
        p["id"].as_str().unwrap_or("p")
    );
    let name = name.trim_start_matches('-');
    commit::move_ref(
        &repo,
        &format!("refs/proposals/{by}/{name}"),
        None::<Oid>,
        Some(c),
        "seed",
    )?;
    push(&author.client, house)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksums() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        assert_eq!(adler32(b"Wikipedia"), 0x11e6_0398);
        let p = png(3, 2, [255, 0, 0]);
        assert!(p.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(
            epoch("2026-10-18T16:02:00.000Z").unwrap(),
            epoch("2026-10-18T16:02:00Z").unwrap()
        );
        assert!(epoch("yesterday").is_err());
    }
}
