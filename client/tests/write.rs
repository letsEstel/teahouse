//! The write side end to end: each person is an engine of their own, every
//! push goes to a real relay (`teahouse-relay shell`, the forced command
//! sshd runs, through the engine's development transport), and the relay
//! judges with the git program. So these tests are also the cross-check of
//! CLIENT-API.md §7: what libgit2 writes and signs, git accepts.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Once;
use teahouse_client::commit::{self, Author, Edit};
use teahouse_client::Client;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Tmp(PathBuf);

impl Tmp {
    fn new(tag: &str) -> Tmp {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let p = std::env::temp_dir().join(format!("thw-{tag}-{}-{n}-{nanos}", std::process::id()));
        fs::create_dir_all(&p).unwrap();
        Tmp(p)
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        if std::env::var_os("TEAHOUSE_KEEP_TMP").is_none() {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

/// The relay binary of this workspace, built if needed.
fn relay_bin() -> PathBuf {
    static BUILD: Once = Once::new();
    let dir = Path::new(env!("CARGO_BIN_EXE_teahouse-client"))
        .parent()
        .unwrap()
        .to_path_buf();
    let bin = dir.join("teahouse-relay");
    BUILD.call_once(|| {
        let st = Command::new(env!("CARGO"))
            .args([
                "build",
                "-q",
                "-p",
                "teahouse-relay",
                "--bin",
                "teahouse-relay",
            ])
            .status()
            .unwrap();
        assert!(st.success(), "cannot build teahouse-relay");
        // SAFETY: set once, before any engine runs the relay; every test
        // sets the same value.
        unsafe { std::env::set_var("TEAHOUSE_RELAY_BIN", &bin) };
    });
    bin
}

fn run(cmd: &mut Command) -> String {
    let o = cmd.output().unwrap();
    assert!(
        o.status.success(),
        "{cmd:?}: {}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

struct World {
    tmp: Tmp,
    relay_root: PathBuf,
}

struct Person {
    id: &'static str,
    client: Client,
}

impl Person {
    fn call(&self, cmd: &str, args: Value) -> Value {
        self.client.call(cmd, args)
    }

    fn ok(&self, cmd: &str, args: Value) -> Value {
        let v = self.call(cmd, args.clone());
        assert_eq!(v["ok"], true, "{} {cmd} {args}: {v:#}", self.id);
        v["data"].clone()
    }

    fn err(&self, cmd: &str, args: Value) -> Value {
        let v = self.call(cmd, args.clone());
        assert_eq!(v["ok"], false, "{} {cmd} {args}: {v:#}", self.id);
        v["error"].clone()
    }

    fn house(&self) -> Value {
        self.ok("house", json!({ "house": "wutong" }))
    }

    fn key(&self) -> String {
        self.client.identity().unwrap().public_line()
    }

    fn repo(&self) -> git2::Repository {
        git2::Repository::open(self.client.root().join("houses/wutong")).unwrap()
    }

    /// Cuts this person off the network: the relay's address becomes an ssh
    /// port nobody listens on, so the real ssh transport fails to connect.
    /// Returns the address to restore.
    fn go_offline(&self) -> String {
        let repo = self.repo();
        let url = repo
            .find_remote("origin")
            .unwrap()
            .url()
            .unwrap()
            .to_string();
        repo.remote_set_url("origin", "ssh://teahouse@127.0.0.1:1/wutong")
            .unwrap();
        url
    }

    fn go_online(&self, url: &str) {
        self.repo().remote_set_url("origin", url).unwrap();
    }
}

impl World {
    fn new(tag: &str) -> World {
        let bin = relay_bin();
        let tmp = Tmp::new(tag);
        let relay_root = tmp.0.join("relay");
        run(Command::new(&bin)
            .arg("--root")
            .arg(&relay_root)
            .args(["init", "--binary"])
            .arg(&bin));
        World { tmp, relay_root }
    }

    fn relay(&self) -> String {
        format!("teahouse-local:{}", self.relay_root.display())
    }

    fn person(&self, id: &'static str) -> Person {
        let p = Person {
            id,
            client: Client::new(self.tmp.0.join(id)),
        };
        p.client.identity().unwrap();
        p
    }

    /// 梧桐里: host wang, steward li, guest zhao, all joined.
    fn wutong(&self) -> (Person, Person, Person) {
        let (wang, li, zhao) = (self.person("wang"), self.person("li"), self.person("zhao"));
        run(Command::new(relay_bin())
            .arg("--root")
            .arg(&self.relay_root)
            .args([
                "open",
                "wutong",
                "--name",
                "梧桐里三号院",
                "--host-id",
                "wang",
            ])
            .args(["--host-name", "王阿姨", "--host-key"])
            .arg(wang.key()));
        wang.ok("join", json!({ "relay": self.relay(), "house": "wutong" }));
        self.seat(
            &wang,
            &[
                ("li", "李师傅", "steward", &li),
                ("zhao", "小赵", "guest", &zhao),
            ],
        );
        for p in [&li, &zhao] {
            p.ok("join", json!({ "relay": self.relay(), "house": "wutong" }));
        }
        (wang, li, zhao)
    }

    /// Seats people by editing the chart as the host (inviting comes with
    /// step 3), committed and signed through the engine's own commit code.
    fn seat(&self, host: &Person, people: &[(&str, &str, &str, &Person)]) {
        let file = host.ok(
            "file",
            json!({ "house": "wutong", "id": ".teahouse/members.toml" }),
        );
        let mut chart = file["text"].as_str().unwrap().to_string();
        for (id, name, role, p) in people {
            chart.push_str(&format!(
                "\n[[member]]\nid = \"{id}\"\nname = \"{name}\"\nrole = \"{role}\"\nkey = \"{}\"\n",
                p.key().rsplit_once(' ').unwrap().0
            ));
        }
        let repo = host.repo();
        let head = repo
            .find_commit(repo.refname_to_id("refs/heads/main").unwrap())
            .unwrap();
        let tree = commit::tree_with(
            &repo,
            Some(&head.tree().unwrap()),
            &[Edit::Put(
                ".teahouse/members.toml".into(),
                chart.into_bytes(),
            )],
        )
        .unwrap();
        let author = Author {
            id: "wang".into(),
            name: "王阿姨".into(),
        }
        .signature(1_792_281_600)
        .unwrap();
        let msg = commit::message(
            "入座",
            None,
            &[("Problem", "meta".into()), ("Act", "seat".into())],
        );
        let id = host.client.identity().unwrap();
        let c = commit::signed_commit(&repo, &id, &author, tree, &[&head], &msg).unwrap();
        commit::move_ref(&repo, "refs/heads/main", Some(head.id()), Some(c), "test").unwrap();
        let s = host.ok("sync", json!({ "house": "wutong" }));
        assert_eq!(s["state"], "synced", "{s:#}");
    }

    /// The relay's own copy, read with the git program.
    fn relay_git(&self, args: &[&str]) -> String {
        run(Command::new("git")
            .arg("-C")
            .arg(self.relay_root.join("houses/wutong.git"))
            .args(args))
    }
}

fn parking(p: &Person) -> Value {
    p.ok(
        "openProblem",
        json!({
            "house": "wutong",
            "title": "小区停车位不好找",
            "body": [
                { "heading": "", "text": "晚上七点以后基本没位置。" },
                { "heading": "现状", "text": "院内共 42 个车位。" },
            ],
        }),
    )
}

#[test]
fn the_whole_flow_reaches_the_relay_signed() {
    let w = World::new("flow");
    let (wang, li, zhao) = w.wutong();

    // The steward opens a problem and adds a plan: straight onto main.
    let r = parking(&li);
    assert_eq!((&r["commit"], &r["id"]), (&json!("pushed"), &json!("0001")));
    assert_eq!(r["sync"]["state"], "synced");
    let r = li.ok(
        "addWork",
        json!({ "house": "wutong", "problem": "0001", "kind": "plan", "title": "错峰停车方案", "text": "白天让给外来车。\n", "note": "试行一个月" }),
    );
    assert_eq!(r["id"], "works/0001/错峰停车方案.md");

    // A guest's addWork becomes a proposal.
    zhao.ok("sync", json!({ "house": "wutong" }));
    let r = zhao.ok(
        "addWork",
        json!({ "house": "wutong", "problem": "0001", "kind": "table", "title": "各楼车位", "text": "楼号,车位\n1,12\n" }),
    );
    assert_eq!(r["becameProposal"], true);
    let pid = r["id"].as_str().unwrap().to_string();
    assert!(pid.starts_with("zhao/"), "{pid}");
    assert_eq!(r["commit"], "pushed");

    // The host sees it, adopts it; the proposal ref goes in the same push.
    wang.ok("sync", json!({ "house": "wutong" }));
    let h = wang.house();
    assert_eq!(h["proposals"][0]["id"], pid);
    assert_eq!(h["proposals"][0]["by"], "zhao");
    let r = wang.ok("adopt", json!({ "house": "wutong", "proposal": pid }));
    assert_eq!(r["commit"], "pushed");
    assert!(w.relay_git(&["for-each-ref", "refs/proposals/"]).is_empty());

    // Guests post directly.
    zhao.ok("sync", json!({ "house": "wutong" }));
    let r = zhao.ok(
        "note",
        json!({ "house": "wutong", "problem": "0001", "text": "我数过，空位只有三个。" }),
    );
    assert_eq!(r["commit"], "pushed");

    // Everyone converges on the same history.
    for p in [&wang, &li, &zhao] {
        p.ok("sync", json!({ "house": "wutong" }));
    }
    let prob = li.ok("problem", json!({ "house": "wutong", "problem": "0001" }));
    assert_eq!(prob["summary"], "晚上七点以后基本没位置。");
    let works: Vec<(&str, &str)> = prob["works"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| (w["title"].as_str().unwrap(), w["kind"].as_str().unwrap()))
        .collect();
    assert_eq!(works, [("各楼车位", "table"), ("错峰停车方案", "plan")]);
    assert_eq!(prob["works"][1]["note"], "试行一个月");
    assert_eq!(prob["notes"][0]["by"], "zhao");
    let acts: Vec<&str> = prob["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["act"].as_str().unwrap())
        .collect();
    assert_eq!(acts, ["open-problem", "add-work", "adopt", "note"]);
    let h = li.house();
    assert_eq!(h["decided"][0]["result"], "adopted");
    assert_eq!(h["decided"][0]["by"], "zhao");
    assert_eq!(h["sync"]["state"], "synced");
    let authors: Vec<&str> = h["recent"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["by"].as_str().unwrap())
        .collect();
    assert_eq!(&authors[..4], ["zhao", "wang", "li", "li"]);

    // Every commit the engines wrote verifies with git's own ssh signature
    // check, against the keys on the chart.
    let signers = w.tmp.0.join("allowed_signers");
    let lines: String = [(&wang, "wang"), (&li, "li"), (&zhao, "zhao")]
        .iter()
        .map(|(p, id)| format!("{id}@teahouse {}\n", p.key()))
        .collect();
    fs::write(&signers, lines).unwrap();
    let ours = w.relay_git(&["rev-list", "main", "--not", "main~5"]);
    for c in ours.lines() {
        let o = Command::new("git")
            .arg("-C")
            .arg(w.relay_root.join("houses/wutong.git"))
            .args(["-c", "gpg.format=ssh", "-c"])
            .arg(format!("gpg.ssh.allowedSignersFile={}", signers.display()))
            .args(["verify-commit", "--raw", c])
            .output()
            .unwrap();
        assert!(
            o.status.success(),
            "{c}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        assert!(
            String::from_utf8_lossy(&o.stderr).contains("Good \"git\" signature"),
            "{c}"
        );
    }
}

#[test]
fn preflight_refuses_before_writing() {
    let w = World::new("preflight");
    let (wang, li, zhao) = w.wutong();
    parking(&li);
    zhao.ok("sync", json!({ "house": "wutong" }));
    let before = zhao.repo().refname_to_id("refs/heads/main").unwrap();
    for (cmd, args) in [
        (
            "setStatus",
            json!({ "house": "wutong", "problem": "0001", "status": "active" }),
        ),
        ("adopt", json!({ "house": "wutong", "proposal": "zhao/x" })),
        ("seat", json!({ "house": "wutong", "remove": "li" })),
        (
            "removeNote",
            json!({ "house": "wutong", "id": "talk/0001/x.md" }),
        ),
    ] {
        assert_eq!(zhao.err(cmd, args)["code"], "forbidden", "{cmd}");
    }
    let e = li.err("seat", json!({ "house": "wutong", "remove": "zhao" }));
    assert_eq!(e["code"], "forbidden");
    let big = "字".repeat(12_000);
    let e = zhao.err(
        "note",
        json!({ "house": "wutong", "problem": "0001", "text": big }),
    );
    assert_eq!(e["code"], "too_large");
    let e = zhao.err(
        "note",
        json!({ "house": "wutong", "problem": "0009", "text": "?" }),
    );
    assert_eq!(
        (&e["code"], &e["params"]["problem"]),
        (&json!("not_found"), &json!("0009"))
    );
    assert_eq!(
        zhao.repo().refname_to_id("refs/heads/main").unwrap(),
        before,
        "nothing was written"
    );
    // A device with no seat cannot write at all.
    let stranger = w.person("stranger");
    let e = stranger.err("join", json!({ "relay": w.relay(), "house": "wutong" }));
    assert_eq!(e["code"], "not_a_member", "{e:#}");
    assert!(!stranger.client.root().join("houses/wutong").exists());
    let _ = wang;
}

#[test]
fn a_refused_push_is_taken_back_and_kept_as_a_draft() {
    let w = World::new("refused");
    let (wang, li, _zhao) = w.wutong();
    parking(&li);
    // The host makes li a guest; li's engine has not synced and still
    // thinks it may open problems directly. The relay says no.
    wang.ok("sync", json!({ "house": "wutong" }));
    wang.ok(
        "seat",
        json!({ "house": "wutong", "role": { "id": "li", "role": "guest" } }),
    );
    let before = li.repo().refname_to_id("refs/heads/main").unwrap();
    let e = li.err(
        "openProblem",
        json!({ "house": "wutong", "title": "楼道灯坏了" }),
    );
    assert_eq!(e["code"], "relay_refused", "{e:#}");
    let reasons: Vec<&str> = e["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap())
        .collect();
    assert!(
        reasons.iter().any(|r| r.contains("problems/")),
        "{reasons:#?}"
    );
    assert!(
        reasons.iter().all(|r| !r.starts_with("remote:")),
        "{reasons:#?}"
    );
    // Back to where it was (plus the host's change, fetched meanwhile),
    // with what li wrote kept.
    let main = li.repo().refname_to_id("refs/heads/main").unwrap();
    let tracking = li.repo().refname_to_id("refs/remotes/origin/main").unwrap();
    assert!(
        main == before || main == tracking,
        "the refused commit is gone"
    );
    assert_eq!(li.house()["problems"].as_array().unwrap().len(), 1);
    let draft = e["params"]["draft"].as_str().unwrap();
    let d: Value =
        serde_json::from_slice(&fs::read(li.client.root().join(draft)).unwrap()).unwrap();
    assert_eq!(d["command"], "openProblem");
    assert_eq!(d["args"]["title"], "楼道灯坏了");
    // After syncing, li knows the new seat and the same command becomes a
    // proposal.
    li.ok("sync", json!({ "house": "wutong" }));
    let r = li.ok(
        "openProblem",
        json!({ "house": "wutong", "title": "楼道灯坏了" }),
    );
    assert_eq!(r["becameProposal"], true);
}

#[test]
fn offline_writes_wait_for_the_next_sync() {
    let w = World::new("offline");
    let (_wang, li, _zhao) = w.wutong();
    parking(&li);
    let url = li.go_offline();
    let r = li.ok(
        "note",
        json!({ "house": "wutong", "problem": "0001", "text": "断网时写的。" }),
    );
    assert_eq!(r["commit"], "local");
    assert_eq!(r["sync"]["state"], "offline");
    assert_eq!(r["sync"]["unpushed"], 1);
    let s = li.ok("houses", Value::Null);
    assert_eq!(s[0]["sync"]["state"], "offline");
    li.go_online(&url);
    let s = li.ok("sync", json!({ "house": "wutong" }));
    assert_eq!((&s["state"], &s["unpushed"]), (&json!("synced"), &json!(0)));
    assert!(s["lastSync"].is_string());
    assert!(w
        .relay_git(&["log", "-1", "--format=%s", "main"])
        .contains("断网时写的"));
}

#[test]
fn concurrent_new_problems_are_renumbered_on_replay() {
    let w = World::new("renumber");
    let (wang, li, _zhao) = w.wutong();
    parking(&li);
    wang.ok("sync", json!({ "house": "wutong" }));
    // li works offline: problem 0002 and a post under it.
    let url = li.go_offline();
    let r = li.ok(
        "openProblem",
        json!({ "house": "wutong", "title": "楼道灯坏了" }),
    );
    assert_eq!((&r["commit"], &r["id"]), (&json!("local"), &json!("0002")));
    li.ok(
        "note",
        json!({ "house": "wutong", "problem": "0002", "text": "三楼的灯。" }),
    );
    li.go_online(&url);
    // Meanwhile the host takes 0002.
    let r = wang.ok(
        "openProblem",
        json!({ "house": "wutong", "title": "垃圾分类" }),
    );
    assert_eq!((&r["commit"], &r["id"]), (&json!("pushed"), &json!("0002")));
    // li's sync replays both commits on top, as 0003.
    let s = li.ok("sync", json!({ "house": "wutong" }));
    assert_eq!(s["state"], "synced", "{s:#}");
    let h = li.house();
    let titles: Vec<(&str, &str)> = h["problems"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["id"].as_str().unwrap(), p["title"].as_str().unwrap()))
        .collect();
    assert_eq!(
        titles,
        [
            ("0001", "小区停车位不好找"),
            ("0002", "垃圾分类"),
            ("0003", "楼道灯坏了")
        ]
    );
    let p3 = li.ok("problem", json!({ "house": "wutong", "problem": "0003" }));
    assert_eq!(p3["notes"][0]["text"], "三楼的灯。");
    let msg = w.relay_git(&["log", "--format=%B", "-1", "main~1"]);
    assert!(
        msg.contains("Problem: 0003") && msg.contains("Renumbered: 0002 -> 0003"),
        "{msg}"
    );
}

#[test]
fn adopting_a_new_problem_whose_number_was_taken_renumbers_it() {
    let w = World::new("adopt-renumber");
    let (wang, li, zhao) = w.wutong();
    parking(&li);
    zhao.ok("sync", json!({ "house": "wutong" }));
    let r = zhao.ok("openProblem", json!({ "house": "wutong", "title": "健身器材", "body": [{ "heading": "", "text": "单杠松了。" }] }));
    assert_eq!(r["becameProposal"], true);
    let pid = r["id"].as_str().unwrap().to_string();
    li.ok(
        "openProblem",
        json!({ "house": "wutong", "title": "楼道灯坏了" }),
    );
    wang.ok("sync", json!({ "house": "wutong" }));
    let r = wang.ok("adopt", json!({ "house": "wutong", "proposal": pid }));
    assert_eq!(r["commit"], "pushed");
    let h = wang.house();
    let titles: Vec<(&str, &str)> = h["problems"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["id"].as_str().unwrap(), p["title"].as_str().unwrap()))
        .collect();
    assert_eq!(
        titles,
        [
            ("0001", "小区停车位不好找"),
            ("0002", "楼道灯坏了"),
            ("0003", "健身器材")
        ]
    );
    let msg = w.relay_git(&["log", "--format=%B", "-1", "main"]);
    assert!(
        msg.contains("Renumbered: 0002 -> 0003") && msg.contains("Problem: 0003"),
        "{msg}"
    );
}

#[test]
fn decline_withdraw_status_seat_and_moderation() {
    let w = World::new("manage");
    let (wang, li, zhao) = w.wutong();
    parking(&li);
    zhao.ok("sync", json!({ "house": "wutong" }));
    let a = zhao.ok(
        "propose",
        json!({
            "house": "wutong", "problem": "0001", "title": "拆掉花坛", "reason": "腾出车位。",
            "changes": [{ "kind": "doc", "title": "花坛", "text": "拆。" }],
        }),
    );
    let b = zhao.ok("propose", json!({
        "house": "wutong", "problem": "0001", "title": "画线", "changes": [{ "kind": "doc", "title": "画线", "text": "画。" }],
    }));
    let (a, b) = (a["id"].as_str().unwrap(), b["id"].as_str().unwrap());
    wang.ok("sync", json!({ "house": "wutong" }));
    wang.ok(
        "decline",
        json!({ "house": "wutong", "proposal": a, "reason": "花坛是大家一起种的。" }),
    );
    zhao.ok("withdraw", json!({ "house": "wutong", "proposal": b }));
    assert!(w.relay_git(&["for-each-ref", "refs/proposals/"]).is_empty());
    wang.ok("sync", json!({ "house": "wutong" }));
    let h = wang.house();
    assert!(h["proposals"].as_array().unwrap().is_empty());
    assert_eq!(h["decided"][0]["result"], "declined");
    assert_eq!(h["decided"][0]["reason"], "花坛是大家一起种的。");

    li.ok("sync", json!({ "house": "wutong" }));
    li.ok(
        "setStatus",
        json!({ "house": "wutong", "problem": "0001", "status": "active", "owner": "li" }),
    );
    let p = li.ok("problem", json!({ "house": "wutong", "problem": "0001" }));
    assert_eq!(
        (&p["status"], &p["owner"]),
        (&json!("active"), &json!("li"))
    );

    zhao.ok("sync", json!({ "house": "wutong" }));
    let n = zhao.ok(
        "note",
        json!({ "house": "wutong", "problem": "0001", "text": "广告：收旧家电" }),
    );
    li.ok("sync", json!({ "house": "wutong" }));
    li.ok("removeNote", json!({ "house": "wutong", "id": n["id"] }));
    let p = li.ok("problem", json!({ "house": "wutong", "problem": "0001" }));
    assert!(p["notes"].as_array().unwrap().is_empty());
    assert_eq!(
        p["events"].as_array().unwrap().last().unwrap()["act"],
        "remove-note"
    );

    wang.ok("sync", json!({ "house": "wutong" }));
    wang.ok("seat", json!({ "house": "wutong", "remove": "zhao" }));
    let h = wang.house();
    assert_eq!(h["members"].as_array().unwrap().len(), 2);
    // zhao's next sync: no seat any more, the relay does not let him in.
    let s = zhao.ok("sync", json!({ "house": "wutong" }));
    assert_eq!(s["state"], "error", "{s:#}");
}

#[test]
fn notices_and_replies() {
    let w = World::new("notices");
    let (wang, li, zhao) = w.wutong();
    let r = li.ok(
        "postNotice",
        json!({ "house": "wutong", "title": "Water off Saturday", "text": "周六上午停水。" }),
    );
    assert_eq!(r["id"], "water-off-saturday");
    let r = li.ok(
        "postNotice",
        json!({ "house": "wutong", "title": "周日开会", "text": "晚上七点。" }),
    );
    let second = r["id"].as_str().unwrap().to_string();
    assert!(second.starts_with('n'), "{second}");
    zhao.ok("sync", json!({ "house": "wutong" }));
    zhao.ok(
        "note",
        json!({ "house": "wutong", "notice": "water-off-saturday", "text": "知道了。" }),
    );
    let r = zhao.ok(
        "postNotice",
        json!({ "house": "wutong", "title": "失物招领", "text": "捡到钥匙一串。" }),
    );
    assert_eq!(r["becameProposal"], true);
    wang.ok("sync", json!({ "house": "wutong" }));
    let h = wang.house();
    let board: Vec<&str> = h["board"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["title"].as_str().unwrap())
        .collect();
    assert_eq!(board.len(), 2);
    assert!(board.contains(&"周日开会"));
    let water = h["board"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == "water-off-saturday")
        .unwrap();
    assert_eq!(water["replies"][0]["text"], "知道了。");
    assert_eq!(h["proposals"][0]["problem"], "board");
    wang.ok(
        "adopt",
        json!({ "house": "wutong", "proposal": h["proposals"][0]["id"] }),
    );
    assert_eq!(wang.house()["board"].as_array().unwrap().len(), 3);
}

#[test]
fn attachments_go_to_the_cache_and_into_git_as_references() {
    let w = World::new("blobs");
    let (_wang, li, _zhao) = w.wutong();
    parking(&li);
    let photo = w.tmp.0.join("车位.png");
    fs::write(&photo, b"\x89PNG fake").unwrap();
    let r = li.ok(
        "addWork",
        json!({ "house": "wutong", "problem": "0001", "kind": "image", "title": "车位照片", "fromPath": photo }),
    );
    assert_eq!(r["id"], "works/0001/车位照片.png.ref.md");
    let p = li.ok("problem", json!({ "house": "wutong", "problem": "0001" }));
    let img = &p["works"][0];
    assert_eq!((&img["kind"], &img["size"]), (&json!("image"), &json!(9)));
    assert_eq!(img["blob"]["cached"], true);
    let path = li.ok("blob", json!({ "house": "wutong", "id": r["id"] }));
    assert_eq!(fs::read(path.as_str().unwrap()).unwrap(), b"\x89PNG fake");
    // Only the small reference went into git.
    let text = w.relay_git(&["show", "main:works/0001/车位照片.png.ref.md"]);
    // Values with `:` are quoted (CLIENT-API.md §1.1).
    assert!(
        text.contains("blob: \"sha256:") && text.contains("type: image/png"),
        "{text}"
    );
}

#[test]
fn serve_answers_local_pages_only() {
    use std::io::{BufRead, BufReader, Read, Write};
    let w = World::new("serve");
    let (_wang, li, _zhao) = w.wutong();
    parking(&li);
    let mut child = Command::new(env!("CARGO_BIN_EXE_teahouse-client"))
        .arg("--root")
        .arg(li.client.root())
        .args(["serve", "--port", "0", "--sync-every", "off"])
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut line = String::new();
    BufReader::new(child.stderr.as_mut().unwrap())
        .read_line(&mut line)
        .unwrap();
    let port: u16 = line
        .trim()
        .rsplit(':')
        .next()
        .unwrap()
        .trim_end_matches("/v1/")
        .parse()
        .unwrap();
    let request = |head: &str, body: &str| -> (u16, String) {
        let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(
            s,
            "{head}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        let status = out[9..12].parse().unwrap();
        (
            status,
            out.split("\r\n\r\n").nth(1).unwrap_or("").to_string(),
        )
    };
    let host = format!("Host: 127.0.0.1:{port}\r\n");
    let json_ct = "Content-Type: application/json\r\n";
    let body = r#"{"house":"wutong","problem":"0001"}"#;

    let (st, out) = request(
        &format!("POST /v1/problem HTTP/1.1\r\n{host}{json_ct}Origin: http://localhost:5173\r\n"),
        body,
    );
    assert_eq!(st, 200);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["data"]["title"], "小区停车位不好找");

    let (st, out) = request(
        &format!("POST /v1/note HTTP/1.1\r\n{host}{json_ct}"),
        r#"{"house":"wutong","problem":"0001","text":"从 HTTP 来的。"}"#,
    );
    assert_eq!(st, 200, "{out}");
    assert_eq!(
        serde_json::from_str::<Value>(&out).unwrap()["data"]["commit"],
        "pushed"
    );

    // Another site, a rebinding host name, or a form post: refused.
    let (st, _) = request(
        &format!("POST /v1/problem HTTP/1.1\r\n{host}{json_ct}Origin: https://evil.example\r\n"),
        body,
    );
    assert_eq!(st, 403);
    let (st, _) = request(
        &format!("POST /v1/problem HTTP/1.1\r\nHost: evil.example:{port}\r\n{json_ct}"),
        body,
    );
    assert_eq!(st, 403);
    let (st, _) = request(
        &format!("POST /v1/problem HTTP/1.1\r\n{host}Content-Type: text/plain\r\n"),
        body,
    );
    assert_eq!(st, 415);
    let (st, out) = request(
        &format!("OPTIONS /v1/problem HTTP/1.1\r\n{host}Origin: http://localhost:5173\r\n"),
        "",
    );
    assert_eq!(st, 204, "{out}");
    child.kill().unwrap();
    let _ = child.wait();
}
