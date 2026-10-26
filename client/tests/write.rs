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
        // What the engines run: the relay itself, and once, when the relay
        // root holds a file `race` naming another engine's root, that
        // engine syncs right after this fetch: someone pushing between a
        // fetch and the push after it.
        let wrapper = dir.join("teahouse-relay-race");
        fs::write(
            &wrapper,
            format!(
                "#!/bin/sh\n\"{}\" \"$@\"\nst=$?\nrace=\"$2/race\"\ncase \"$SSH_ORIGINAL_COMMAND\" in\n  git-upload-pack*)\n    if other=$(cat \"$race\" 2>/dev/null) && rm \"$race\"; then\n      \"{}\" --root \"$other\" call sync '{{\"house\":\"wutong\"}}' >/dev/null\n    fi ;;\nesac\nexit $st\n",
                bin.display(),
                env!("CARGO_BIN_EXE_teahouse-client")
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
        }
        // SAFETY: set once, before any engine runs the relay; every test
        // sets the same value.
        unsafe { std::env::set_var("TEAHOUSE_RELAY_BIN", &wrapper) };
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
fn problems_and_notices_are_edited_directly_or_as_proposals() {
    let w = World::new("edit");
    let (wang, li, zhao) = w.wutong();
    parking(&li);
    li.ok(
        "setStatus",
        json!({ "house": "wutong", "problem": "0001", "status": "active", "owner": "li" }),
    );

    // The steward changes title and text; status, owner and file name stay.
    li.ok(
        "updateProblem",
        json!({
            "house": "wutong", "problem": "0001", "title": "停车难",
            "text": "晚上六点以后就没位置了。\n\n## 现状\n院内共 40 个车位。\n",
        }),
    );
    let p = li.ok("problem", json!({ "house": "wutong", "problem": "0001" }));
    assert_eq!(p["title"], "停车难");
    assert_eq!(p["summary"], "晚上六点以后就没位置了。");
    assert_eq!(p["body"][1]["heading"], "现状");
    assert_eq!(p["body"][1]["text"], "院内共 40 个车位。");
    assert_eq!(
        (&p["status"], &p["owner"]),
        (&json!("active"), &json!("li"))
    );
    assert_eq!(
        p["events"].as_array().unwrap().last().unwrap()["act"],
        "update-problem"
    );
    let ls = w.relay_git(&[
        "-c",
        "core.quotePath=false",
        "ls-tree",
        "--name-only",
        "main",
        "problems/",
    ]);
    assert_eq!(ls.trim().lines().count(), 1, "{ls}");
    assert_eq!(
        w.relay_git(&["log", "-1", "--format=%s", "main"]).trim(),
        "停车难"
    );

    // By path, title only.
    let first = ls.trim().to_string();
    li.ok(
        "updateProblem",
        json!({ "house": "wutong", "problem": first, "title": "停车太难" }),
    );
    let e = li.err(
        "updateProblem",
        json!({ "house": "wutong", "problem": "0001", "title": "停车太难" }),
    );
    assert_eq!(e["code"], "invalid", "{e:#}");
    let e = li.err(
        "updateProblem",
        json!({ "house": "wutong", "problem": "0001" }),
    );
    assert_eq!(e["code"], "invalid", "{e:#}");
    let e = li.err(
        "updateProblem",
        json!({ "house": "wutong", "problem": "0009", "text": "x" }),
    );
    assert_eq!(e["code"], "not_found", "{e:#}");

    // A guest's edit becomes a proposal; main is unchanged until adopted.
    zhao.ok("sync", json!({ "house": "wutong" }));
    let r = zhao.ok(
        "updateProblem",
        json!({ "house": "wutong", "problem": "0001", "text": "其实还好。\n" }),
    );
    assert_eq!(r["becameProposal"], true);
    let p = zhao.ok("problem", json!({ "house": "wutong", "problem": "0001" }));
    assert_eq!(p["summary"], "晚上六点以后就没位置了。");
    wang.ok("sync", json!({ "house": "wutong" }));
    let h = wang.house();
    assert_eq!(h["proposals"][0]["problem"], "0001");
    wang.ok(
        "adopt",
        json!({ "house": "wutong", "proposal": h["proposals"][0]["id"] }),
    );
    let p = wang.ok("problem", json!({ "house": "wutong", "problem": "0001" }));
    assert_eq!(
        (&p["title"], &p["summary"]),
        (&json!("停车太难"), &json!("其实还好。"))
    );
    assert_eq!(p["status"], "active");

    // Notices, by name or path.
    li.ok("sync", json!({ "house": "wutong" }));
    li.ok(
        "postNotice",
        json!({ "house": "wutong", "title": "Water off Saturday", "text": "周六上午停水。" }),
    );
    let r = li.ok(
        "updateNotice",
        json!({ "house": "wutong", "notice": "water-off-saturday", "text": "周六全天停水。" }),
    );
    assert_eq!(r["id"], "water-off-saturday");
    li.ok(
        "updateNotice",
        json!({ "house": "wutong", "notice": "board/water-off-saturday.md", "title": "Water off all Saturday" }),
    );
    let n = li.house()["board"][0].clone();
    assert_eq!(
        (&n["title"], &n["text"]),
        (&json!("Water off all Saturday"), &json!("周六全天停水。"))
    );
    assert_eq!(
        w.relay_git(&[
            "log",
            "-1",
            "--format=%(trailers:key=Act,valueonly)",
            "main"
        ])
        .trim(),
        "update-notice"
    );
    let e = li.err(
        "updateNotice",
        json!({ "house": "wutong", "notice": "water-off-saturday", "text": "长".repeat(12_000) }),
    );
    assert_eq!(e["code"], "too_large", "{e:#}");
    let e = li.err(
        "updateNotice",
        json!({ "house": "wutong", "notice": "nope", "text": "x" }),
    );
    assert_eq!(e["code"], "not_found", "{e:#}");

    zhao.ok("sync", json!({ "house": "wutong" }));
    let r = zhao.ok(
        "updateNotice",
        json!({ "house": "wutong", "notice": "water-off-saturday", "text": "改到周日。" }),
    );
    assert_eq!(r["becameProposal"], true);
    wang.ok("sync", json!({ "house": "wutong" }));
    let h = wang.house();
    assert_eq!(h["proposals"][0]["problem"], "board");
    wang.ok(
        "adopt",
        json!({ "house": "wutong", "proposal": h["proposals"][0]["id"] }),
    );
    assert_eq!(wang.house()["board"][0]["text"], "改到周日。");
}

#[test]
fn a_text_with_its_own_frontmatter_replaces_the_whole_file() {
    let w = World::new("fm");
    let (_wang, li, _zhao) = w.wutong();
    parking(&li);
    let r = li.ok(
        "addWork",
        json!({ "house": "wutong", "problem": "0001", "kind": "doc", "title": "访谈", "text": "第一版。" }),
    );
    let id = r["id"].as_str().unwrap().to_string();
    // An editor that loaded the whole file (`file` gives it with its
    // frontmatter) sends it back whole: the frontmatter is not doubled.
    let file = li.ok("file", json!({ "house": "wutong", "id": id }));
    let whole = file["text"]
        .as_str()
        .unwrap()
        .replace("第一版。", "第二版。");
    li.ok(
        "updateWork",
        json!({ "house": "wutong", "id": id, "text": whole }),
    );
    let file = li.ok("file", json!({ "house": "wutong", "id": id }));
    assert_eq!(file["text"], "---\ntitle: 访谈\n---\n第二版。");
    // The body alone keeps the frontmatter, as before.
    li.ok(
        "updateWork",
        json!({ "house": "wutong", "id": id, "text": "第三版。" }),
    );
    let file = li.ok("file", json!({ "house": "wutong", "id": id }));
    assert_eq!(file["text"], "---\ntitle: 访谈\n---\n第三版。");

    // A problem: the whole file plus a new title; the title wins.
    let p = w.relay_git(&[
        "-c",
        "core.quotePath=false",
        "ls-tree",
        "--name-only",
        "main",
        "problems/",
    ]);
    let p = p.trim();
    let whole = li.ok("file", json!({ "house": "wutong", "id": p }))["text"]
        .as_str()
        .unwrap()
        .replace("42", "40");
    li.ok(
        "updateProblem",
        json!({ "house": "wutong", "problem": p, "text": whole, "title": "停车难" }),
    );
    let text = li.ok("file", json!({ "house": "wutong", "id": p }))["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(text.matches("---\n").count(), 2, "{text}");
    assert!(
        text.contains("title: 停车难\n") && text.contains("院内共 40 个车位。"),
        "{text}"
    );
    assert!(text.contains("status: open\n"), "{text}");
}

#[test]
fn a_push_that_came_too_late_is_replayed_and_pushed_again() {
    let w = World::new("race");
    let (_wang, li, zhao) = w.wutong();
    parking(&li);
    zhao.ok("sync", json!({ "house": "wutong" }));
    // li writes offline; then, between zhao's fetch and zhao's push, li
    // comes online and pushes first.
    let url = li.go_offline();
    li.ok(
        "note",
        json!({ "house": "wutong", "problem": "0001", "text": "李师傅先到。" }),
    );
    li.go_online(&url);
    fs::write(
        w.relay_root.join("race"),
        li.client.root().to_str().unwrap(),
    )
    .unwrap();
    let r = zhao.ok(
        "note",
        json!({ "house": "wutong", "problem": "0001", "text": "小赵晚了两秒。" }),
    );
    assert!(!w.relay_root.join("race").exists(), "the race took place");
    assert_eq!(r["commit"], "pushed", "{r:#}");
    let log = w.relay_git(&["log", "--format=%s", "main"]);
    let mut lines = log.lines();
    assert_eq!(
        (lines.next(), lines.next()),
        (Some("小赵晚了两秒。"), Some("李师傅先到。")),
        "{log}"
    );
    assert!(zhao
        .ok("drafts", json!({ "house": "wutong" }))
        .as_array()
        .unwrap()
        .is_empty());
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
    // Killed however the test ends, so a failure cannot leave it running.
    struct KillOnDrop(std::process::Child);
    impl Drop for KillOnDrop {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut child = KillOnDrop(
        Command::new(env!("CARGO_BIN_EXE_teahouse-client"))
            .arg("--root")
            .arg(li.client.root())
            .args(["serve", "--port", "0", "--sync-every", "off"])
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let mut line = String::new();
    BufReader::new(child.0.stderr.as_mut().unwrap())
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
    // Attachments: only from this engine's cache, by attachment id.
    let photo = w.tmp.0.join("p.png");
    fs::write(&photo, b"fake png").unwrap();
    li.ok(
        "addWork",
        json!({ "house": "wutong", "problem": "0001", "kind": "image", "title": "车位", "fromPath": photo }),
    );
    let id = "works%2F0001%2F%E8%BD%A6%E4%BD%8D.png.ref.md";
    let (st, out) = request(
        &format!("GET /v1/blob?house=wutong&id={id} HTTP/1.1\r\n{host}"),
        "",
    );
    assert_eq!((st, out.as_str()), (200, "fake png"));
    let (st, _) = request(
        &format!("GET /v1/blob?house=wutong&id=.teahouse%2Fmembers.toml HTTP/1.1\r\n{host}"),
        "",
    );
    assert_eq!(st, 404, "only attachments, never other files");
    let (st, _) = request(
        &format!("GET /v1/blob?house=..%2F..&id=x HTTP/1.1\r\n{host}"),
        "",
    );
    assert_eq!(st, 404);
    let (st, out) = request(
        &format!("OPTIONS /v1/problem HTTP/1.1\r\n{host}Origin: http://localhost:5173\r\n"),
        "",
    );
    assert_eq!(st, 204, "{out}");
    drop(child);
}

#[test]
fn an_invitation_brings_a_new_device_in() {
    let w = World::new("invite");
    let (wang, li, zhao) = w.wutong();
    parking(&li);
    wang.ok("sync", json!({ "house": "wutong" }));
    // Only the host invites.
    assert_eq!(
        li.err("invite", json!({ "house": "wutong" }))["code"],
        "forbidden"
    );
    let inv = wang.ok(
        "invite",
        json!({ "house": "wutong", "name": "王阿姨的女儿", "days": 3 }),
    );
    let inv = inv.as_str().unwrap().to_string();
    let (code, relay) = inv.split_once('@').unwrap();
    assert!(code.starts_with("TH-") && code.len() == 22, "{inv}");
    assert_eq!(relay, w.relay());

    // A brand-new device: it pastes the whole invitation and a name.
    let new = w.person("daughter");
    let h = new.ok(
        "redeem",
        json!({ "code": format!("  {inv}  "), "name": "王阿姨的女儿" }),
    );
    assert_eq!(
        (&h["id"], &h["me"], &h["myRole"]),
        (&json!("wutong"), &json!("m1"), &json!("guest"))
    );
    let r = new.ok(
        "note",
        json!({ "house": "wutong", "problem": "0001", "text": "大家好。" }),
    );
    assert_eq!(r["commit"], "pushed");
    for p in [&wang, &li, &zhao] {
        p.ok("sync", json!({ "house": "wutong" }));
    }
    let members = wang.house()["members"].clone();
    assert!(members
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["id"] == "m1" && m["name"] == "王阿姨的女儿"));
    // The seat change shows as the host's act.
    assert_eq!(wang.house()["recent"][1]["act"], "seat");

    // The same invitation again, and a mistyped one, are refused.
    let other = w.person("other");
    let e = other.err("redeem", json!({ "code": inv, "name": "x" }));
    assert_eq!(e["code"], "relay_refused");
    assert!(
        e["reasons"][0].as_str().unwrap().contains("not valid"),
        "{e:#}"
    );
    let e = other.err(
        "redeem",
        json!({ "code": "TH-0000-0000-0000-0000", "name": "x" }),
    );
    assert_eq!(e["code"], "invalid", "no relay in a bare code: {e:#}");
    // The relay can be given separately.
    let e = other.err(
        "redeem",
        json!({ "relay": w.relay(), "code": "TH-0000-0000-0000-0000", "name": "x" }),
    );
    assert_eq!(e["code"], "relay_refused");
}

#[test]
fn attachments_travel_through_the_relay_and_can_be_purged() {
    let w = World::new("blobs-relay");
    let (wang, li, zhao) = w.wutong();
    parking(&li);
    let photo = w.tmp.0.join("车位.png");
    fs::write(&photo, b"\x89PNG bytes").unwrap();
    let r = li.ok("addWork", json!({ "house": "wutong", "problem": "0001", "kind": "image", "title": "车位照片", "fromPath": photo }));
    let id = r["id"].as_str().unwrap().to_string();
    let hash = teahouse_core::sha256::hex(b"\x89PNG bytes");
    assert!(
        w.relay_root.join("blobs/wutong").join(&hash).is_file(),
        "uploaded before the push"
    );

    // Another device downloads it on first use, checking the hash.
    zhao.ok("sync", json!({ "house": "wutong" }));
    let p = zhao.ok("problem", json!({ "house": "wutong", "problem": "0001" }));
    assert_eq!(p["works"][0]["blob"]["cached"], false);
    let path = zhao.ok("blob", json!({ "house": "wutong", "id": id }));
    assert_eq!(fs::read(path.as_str().unwrap()).unwrap(), b"\x89PNG bytes");

    // The host removes it for good; everyone's reference shows it as purged.
    wang.ok("sync", json!({ "house": "wutong" }));
    assert_eq!(
        li.err("purgeBlob", json!({ "house": "wutong", "id": id }))["code"],
        "forbidden"
    );
    let r = wang.ok("purgeBlob", json!({ "house": "wutong", "id": id }));
    assert_eq!(r["commit"], "pushed");
    assert!(!w.relay_root.join("blobs/wutong").join(&hash).exists());
    li.ok("sync", json!({ "house": "wutong" }));
    let p = li.ok("problem", json!({ "house": "wutong", "problem": "0001" }));
    assert_eq!(p["works"][0]["blob"]["purged"], true);
    let e = li.err("blob", json!({ "house": "wutong", "id": id }));
    assert_eq!(e["params"]["reason"], "purged");
}

#[test]
fn a_purge_made_offline_happens_after_the_next_push() {
    let w = World::new("purge-offline");
    let (wang, li, _zhao) = w.wutong();
    parking(&li);
    let f = w.tmp.0.join("合同.pdf");
    fs::write(&f, b"%PDF fake").unwrap();
    let r = li.ok("addWork", json!({ "house": "wutong", "problem": "0001", "kind": "file", "title": "合同", "fromPath": f }));
    let id = r["id"].as_str().unwrap().to_string();
    let hash = teahouse_core::sha256::hex(b"%PDF fake");
    wang.ok("sync", json!({ "house": "wutong" }));
    let url = wang.go_offline();
    let r = wang.ok("purgeBlob", json!({ "house": "wutong", "id": id }));
    assert_eq!(r["commit"], "local");
    assert!(
        w.relay_root.join("blobs/wutong").join(&hash).exists(),
        "not before the commit is pushed"
    );
    wang.go_online(&url);
    wang.ok("sync", json!({ "house": "wutong" }));
    assert!(!w.relay_root.join("blobs/wutong").join(&hash).exists());
}

#[test]
fn drafts_keep_what_the_relay_refused() {
    let w = World::new("drafts");
    let (wang, li, _zhao) = w.wutong();
    parking(&li);
    wang.ok("sync", json!({ "house": "wutong" }));
    wang.ok(
        "seat",
        json!({ "house": "wutong", "role": { "id": "li", "role": "guest" } }),
    );
    let e = li.err("openProblem", json!({ "house": "wutong", "title": "楼道灯坏了", "body": [{ "heading": "", "text": "三楼。" }] }));
    assert_eq!(e["code"], "relay_refused");
    let d = li.ok("drafts", json!({ "house": "wutong" }));
    let d = d.as_array().unwrap();
    assert_eq!(d.len(), 1);
    assert_eq!(
        (&d[0]["kind"], &d[0]["title"], &d[0]["text"]),
        (
            &json!("openProblem"),
            &json!("楼道灯坏了"),
            &json!("三楼。")
        )
    );
    assert!(
        d[0]["reason"].as_str().unwrap().contains("problems/"),
        "{d:#?}"
    );
    let id = d[0]["id"].as_str().unwrap();
    li.ok("discardDraft", json!({ "house": "wutong", "id": id }));
    assert!(li
        .ok("drafts", json!({ "house": "wutong" }))
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(
        li.err("discardDraft", json!({ "house": "wutong", "id": "../x" }))["code"],
        "not_found"
    );
}

#[test]
fn export_is_a_bundle_git_can_clone_plus_files_and_attachments() {
    let w = World::new("export");
    let (_wang, li, zhao) = w.wutong();
    parking(&li);
    let photo = w.tmp.0.join("p.png");
    fs::write(&photo, b"png!").unwrap();
    li.ok("addWork", json!({ "house": "wutong", "problem": "0001", "kind": "image", "title": "照片", "fromPath": photo }));
    zhao.ok("sync", json!({ "house": "wutong" }));
    zhao.ok("propose", json!({ "house": "wutong", "problem": "0001", "title": "建议", "changes": [{ "kind": "doc", "title": "建议", "text": "x" }] }));
    let out = w.tmp.0.join("exports");
    let dir = li.ok("export", json!({ "house": "wutong", "to": out }));
    let dir = PathBuf::from(dir.as_str().unwrap());
    li.ok("sync", json!({ "house": "wutong" }));
    let dir2 = li.ok("export", json!({ "house": "wutong", "to": out }));
    assert_ne!(
        dir2.as_str().unwrap(),
        dir.to_str().unwrap(),
        "a second export never overwrites"
    );

    let bundle = dir.join("wutong.bundle");
    // `git bundle verify` wants a repository to check against: an empty one
    // proves the bundle needs nothing else.
    let empty = w.tmp.0.join("empty");
    run(Command::new("git").args(["init", "-q"]).arg(&empty));
    run(Command::new("git")
        .arg("bundle")
        .arg("verify")
        .arg(&bundle)
        .current_dir(&empty));
    let clone = w.tmp.0.join("restored");
    run(Command::new("git")
        .arg("clone")
        .arg("-q")
        .arg(&bundle)
        .arg(&clone));
    assert!(clone.join("problems/0001-小区停车位不好找.md").is_file());
    let heads = run(Command::new("git")
        .arg("bundle")
        .arg("list-heads")
        .arg(&bundle)
        .current_dir(&w.tmp.0));
    assert!(heads.contains("refs/heads/main"), "{heads}");
    assert!(dir.join("files/works/0001/照片.png.ref.md").is_file());
    assert_eq!(
        fs::read(dir.join("files/works/0001/照片.png")).unwrap(),
        b"png!"
    );
    assert_eq!(
        fs::read(dir.join("blobs").join(teahouse_core::sha256::hex(b"png!"))).unwrap(),
        b"png!"
    );
    assert!(dir.join("README.txt").is_file());
    // The proposals li had fetched are in the second export's bundle.
    let heads2 = run(Command::new("git")
        .arg("bundle")
        .arg("list-heads")
        .arg(PathBuf::from(dir2.as_str().unwrap()).join("wutong.bundle"))
        .current_dir(&w.tmp.0));
    assert!(heads2.contains("refs/proposals/zhao/"), "{heads2}");
}

#[test]
fn a_founding_invitation_makes_a_new_device_the_host() {
    let w = World::new("found");
    let out = run(Command::new(relay_bin())
        .arg("--root")
        .arg(&w.relay_root)
        .args(["found", "lanes", "--name", "胡同读书会", "--address"])
        .arg(w.relay()));
    let (code, relay) = out.split_once('@').unwrap();
    assert!(code.starts_with("TH-"), "{out}");
    assert_eq!(relay, w.relay());

    // The first person: a fresh device, a name, the pasted invitation.
    let su = w.person("su");
    let h = su.ok("redeem", json!({ "code": out, "name": "苏琳" }));
    assert_eq!(
        (&h["id"], &h["name"], &h["myRole"], &h["seats"]),
        (
            &json!("lanes"),
            &json!("胡同读书会"),
            &json!("host"),
            &json!(1)
        )
    );
    let r = su.ok(
        "openProblem",
        json!({ "house": "lanes", "title": "第一本书读什么" }),
    );
    assert_eq!(r["commit"], "pushed");
    // …and invites the next one, who joins as a guest.
    let inv = su.ok("invite", json!({ "house": "lanes" }));
    let zhang = w.person("zhang");
    let h = zhang.ok("redeem", json!({ "code": inv, "name": "Zhang" }));
    assert_eq!((&h["me"], &h["myRole"]), (&json!("zhang"), &json!("guest")));
    // The founding code is spent.
    let e = w
        .person("late")
        .err("redeem", json!({ "code": out, "name": "x" }));
    assert_eq!(e["code"], "relay_refused", "{e:#}");
}

/// A minimal MCP client: JSON-RPC over the server's stdin/stdout.
struct Mcp {
    child: std::process::Child,
    out: std::io::BufReader<std::process::ChildStdout>,
    next: u64,
    /// Notifications the server sent (methods), in order.
    notes: Vec<String>,
}

impl Mcp {
    fn start(root: &Path) -> Mcp {
        relay_bin();
        let mut child = Command::new(env!("CARGO_BIN_EXE_teahouse-client"))
            .args(["mcp", "--root"])
            .arg(root)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let out = std::io::BufReader::new(child.stdout.take().unwrap());
        Mcp {
            child,
            out,
            next: 1,
            notes: Vec::new(),
        }
    }

    fn send(&mut self, msg: &Value) {
        use std::io::Write;
        let stdin = self.child.stdin.as_mut().unwrap();
        writeln!(stdin, "{msg}").unwrap();
        stdin.flush().unwrap();
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        use std::io::BufRead;
        let id = self.next;
        self.next += 1;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        loop {
            let mut line = String::new();
            self.out.read_line(&mut line).unwrap();
            let v: Value = serde_json::from_str(&line).unwrap_or_else(|e| panic!("{e}: {line}"));
            if v.get("id").is_none() {
                self.notes
                    .push(v["method"].as_str().unwrap_or("").to_string());
                continue;
            }
            assert_eq!(v["id"], id, "{v}");
            return v;
        }
    }

    /// Calls a tool; returns (isError, structured or text payload).
    fn tool(&mut self, name: &str, args: Value) -> (bool, Value) {
        let r = self.request("tools/call", json!({ "name": name, "arguments": args }));
        let result = &r["result"];
        let text = result["content"][0]["text"].as_str().unwrap_or("");
        let payload = serde_json::from_str(text).unwrap_or(json!(text));
        (result["isError"] == true, payload)
    }
}

impl Drop for Mcp {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn an_ai_takes_a_seat_through_mcp_and_works_as_a_guest() {
    let w = World::new("mcp");
    let (wang, li, _zhao) = w.wutong();
    parking(&li);
    wang.ok("sync", json!({ "house": "wutong" }));
    // The host invites an AI (法则第 17 条: always a guest).
    let inv = wang.ok(
        "invite",
        json!({ "house": "wutong", "kind": "ai", "name": "2 号" }),
    );

    let ai_root = w.tmp.0.join("claude-2");
    let mut m = Mcp::start(&ai_root);
    let init = m.request("initialize", json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "0" } }));
    assert_eq!(init["result"]["serverInfo"]["name"], "teahouse");
    m.send(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
    let tools: Vec<String> = m.request("tools/list", json!({}))["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();
    assert!(
        tools.contains(&"note".to_string()) && !tools.contains(&"adopt".to_string()),
        "{tools:?}"
    );

    // Seat, read, post, propose.
    let (err, h) = m.tool(
        "redeem",
        json!({ "code": inv, "name": "2 号", "agent": "Claude Code" }),
    );
    assert!(!err, "{h:#}");
    assert_eq!(
        (&h["myRole"], &h["id"]),
        (&json!("guest"), &json!("wutong"))
    );
    let me = h["me"].as_str().unwrap().to_string();
    assert_eq!(me, "ai-2");
    let (err, p) = m.tool("problem", json!({ "house": "wutong", "problem": "0001" }));
    assert!(!err);
    assert_eq!(p["title"], "小区停车位不好找");
    let (err, r) = m.tool("note", json!({ "house": "wutong", "problem": "0001", "text": "我是 2 号（AI）。建议先统计夜间空位。" }));
    assert!(!err, "{r:#}");
    assert_eq!(r["commit"], "pushed");
    let (err, r) = m.tool("addWork", json!({ "house": "wutong", "problem": "0001", "kind": "doc", "title": "夜间空位统计方法", "text": "每晚十点数一次。\n" }));
    assert!(!err, "{r:#}");
    assert_eq!(r["becameProposal"], true);
    let proposal = r["id"].as_str().unwrap().to_string();
    // People's decisions are not there, by name or otherwise.
    let (err, r) = m.tool("adopt", json!({ "house": "wutong", "proposal": proposal }));
    assert!(err, "{r:#}");
    // Resources read the same data.
    let res = m.request(
        "resources/read",
        json!({ "uri": "teahouse://wutong/problem/0001" }),
    );
    assert!(res["result"]["contents"][0]["text"]
        .as_str()
        .unwrap()
        .contains("小区停车位不好找"));
    drop(m);

    // The host sees an AI member, its post, its proposal; adopting is theirs.
    wang.ok("sync", json!({ "house": "wutong" }));
    let h = wang.house();
    let ai = h["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == me.as_str())
        .unwrap()
        .clone();
    assert_eq!(
        (&ai["kind"], &ai["agent"], &ai["role"]),
        (&json!("machine"), &json!("Claude Code"), &json!("guest"))
    );
    assert_eq!(h["proposals"][0]["by"], me.as_str());
    wang.ok("adopt", json!({ "house": "wutong", "proposal": proposal }));
    let p = wang.ok("problem", json!({ "house": "wutong", "problem": "0001" }));
    assert!(p["notes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["by"] == me.as_str()));
    assert!(p["works"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["title"] == "夜间空位统计方法"));

    // Its commits carry its own signature, which git verifies.
    let key = fs::read_to_string(ai_root.join("device.pub")).unwrap();
    let signers = w.tmp.0.join("ai-signer");
    fs::write(&signers, format!("{me}@teahouse {key}")).unwrap();
    let c = w.relay_git(&[
        "log",
        "--format=%H",
        "-1",
        "--author",
        &format!("{me}@teahouse"),
        "main",
    ]);
    let o = Command::new("git")
        .arg("-C")
        .arg(w.relay_root.join("houses/wutong.git"))
        .args(["-c", "gpg.format=ssh", "-c"])
        .arg(format!("gpg.ssh.allowedSignersFile={}", signers.display()))
        .args(["log", "-1", "--format=%G? %GS", &c])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&o.stdout).trim(),
        format!("G {me}@teahouse")
    );
}

impl Mcp {
    fn tool_names(&mut self) -> Vec<String> {
        self.request("tools/list", json!({}))["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect()
    }

    fn init(&mut self) {
        self.request("initialize", json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "0" } }));
        self.send(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
    }
}

#[test]
fn an_ai_gets_the_tools_of_its_seat() {
    let w = World::new("mcp-roles");
    let (wang, li, _zhao) = w.wutong();
    parking(&li);
    // Make room for an AI steward (法则第 17 条: a label, not a rank).
    wang.ok("sync", json!({ "house": "wutong" }));
    wang.ok(
        "seat",
        json!({ "house": "wutong", "role": { "id": "li", "role": "guest" } }),
    );
    let inv = wang.ok(
        "invite",
        json!({ "house": "wutong", "kind": "machine", "role": "steward", "name": "2 号" }),
    );

    let mut m = Mcp::start(&w.tmp.0.join("ai-steward"));
    m.init();
    assert!(
        !m.tool_names().contains(&"setStatus".to_string()),
        "no seat yet: guest tools only"
    );
    let (err, h) = m.tool(
        "redeem",
        json!({ "code": inv, "name": "2 号", "agent": "Claude Code" }),
    );
    assert!(!err, "{h:#}");
    assert_eq!(h["myRole"], "steward");
    // `kind: machine`: the id carries `bot-`, and it is shown as a machine.
    assert_eq!(h["me"], "bot-2");
    let (_, house) = m.tool("house", json!({ "house": "wutong" }));
    let me = house["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == "bot-2")
        .unwrap()
        .clone();
    assert_eq!(
        (&me["kind"], &me["agent"]),
        (&json!("machine"), &json!("Claude Code"))
    );
    // The seat changed what it may do: the client is told, and the list grows.
    let names = m.tool_names();
    assert_eq!(m.notes, ["notifications/tools/list_changed"]);
    for t in ["setStatus", "removeNote", "note", "propose"] {
        assert!(names.contains(&t.to_string()), "{t}: {names:?}");
    }
    for t in ["adopt", "seat", "invite"] {
        assert!(!names.contains(&t.to_string()), "{t}: {names:?}");
    }
    // A steward writes main directly, and manages status.
    let (err, r) = m.tool(
        "openProblem",
        json!({ "house": "wutong", "title": "夜间空位" }),
    );
    assert!(!err, "{r:#}");
    assert_eq!(
        (r["commit"].clone(), r.get("becameProposal").cloned()),
        (json!("pushed"), None)
    );
    let (err, r) = m.tool(
        "setStatus",
        json!({ "house": "wutong", "problem": "0001", "status": "active" }),
    );
    assert!(!err, "{r:#}");
    // Host-only decisions: the engine says no, not the server.
    let (err, r) = m.tool("adopt", json!({ "house": "wutong", "proposal": "x/y" }));
    assert!(err);
    assert_eq!(r["code"], "forbidden", "{r:#}");
    drop(m);

    // A host: an AI founds a teahouse, and gets every tool.
    let out = run(Command::new(relay_bin())
        .arg("--root")
        .arg(&w.relay_root)
        .args([
            "found",
            "devs",
            "--name",
            "茶馆开发组",
            "--kind",
            "ai",
            "--address",
        ])
        .arg(w.relay()));
    let mut m = Mcp::start(&w.tmp.0.join("ai-host"));
    m.init();
    let (err, h) = m.tool(
        "redeem",
        json!({ "code": out, "name": "评审方", "agent": "Claude" }),
    );
    assert!(!err, "{h:#}");
    assert_eq!((&h["me"], &h["myRole"]), (&json!("ai-1"), &json!("host")));
    let names = m.tool_names();
    for t in [
        "adopt",
        "decline",
        "seat",
        "invite",
        "purgeBlob",
        "setStatus",
        "removeNote",
    ] {
        assert!(names.contains(&t.to_string()), "{t}: {names:?}");
    }
    // It invites a person, who proposes; the AI host adopts.
    let (err, inv) = m.tool("invite", json!({ "house": "devs", "name": "用户" }));
    assert!(!err, "{inv:#}");
    let user = w.person("user");
    user.ok("redeem", json!({ "code": inv, "name": "用户" }));
    let (err, r) = m.tool(
        "openProblem",
        json!({ "house": "devs", "title": "一人多设备" }),
    );
    assert!(!err, "{r:#}");
    user.ok("sync", json!({ "house": "devs" }));
    let p = user.ok("addWork", json!({ "house": "devs", "problem": "0001", "kind": "doc", "title": "座位格式", "text": "keys = [...]\n" }));
    m.tool("sync", json!({ "house": "devs" }));
    let (err, r) = m.tool("adopt", json!({ "house": "devs", "proposal": p["id"] }));
    assert!(!err, "{r:#}");
    assert_eq!(r["commit"], "pushed");
}

#[test]
fn a_phone_pairs_with_the_laptop_and_is_removed_when_lost() {
    let w = World::new("devices");
    let (wang, li, _zhao) = w.wutong();
    parking(&li);
    // wang also hosts a second teahouse at the same relay.
    run(Command::new(relay_bin())
        .arg("--root")
        .arg(&w.relay_root)
        .args([
            "open",
            "lanes",
            "--name",
            "读书会",
            "--host-id",
            "wang",
            "--host-name",
            "王阿姨",
            "--host-key",
        ])
        .arg(wang.key()));
    wang.ok("join", json!({ "relay": w.relay(), "house": "lanes" }));

    // On the laptop: "add a device". On the phone: paste it.
    let code = wang.ok("pair", json!({ "name": "手机" }));
    let code = code.as_str().unwrap();
    assert!(code.starts_with("TH-") && code.contains('@'), "{code}");
    let phone = w.person("wang-phone");
    let h = phone.ok("redeem", json!({ "code": code, "name": "手机" }));
    assert_eq!((&h["me"], &h["myRole"]), (&json!("wang"), &json!("host")));
    let houses: Vec<String> = phone
        .ok("houses", Value::Null)
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(houses, ["lanes", "wutong"], "one code, every teahouse");

    // Both are wang; each knows which device it is.
    let d = phone.ok("devices", json!({ "house": "wutong" }));
    let d = d.as_array().unwrap();
    assert_eq!(d.len(), 2);
    assert!(
        d.iter()
            .any(|x| x["name"] == "手机" && x["thisDevice"] == true),
        "{d:?}"
    );
    assert!(
        d.iter()
            .any(|x| x["name"] == "" && x["thisDevice"] == false),
        "{d:?}"
    );
    let me = phone.house()["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == "wang")
        .unwrap()
        .clone();
    assert_eq!(me["devices"].as_array().unwrap().len(), 2);
    let r = phone.ok(
        "note",
        json!({ "house": "wutong", "problem": "0001", "text": "手机上写的。" }),
    );
    assert_eq!(r["commit"], "pushed");
    wang.ok("sync", json!({ "house": "wutong" }));
    let notes =
        wang.ok("problem", json!({ "house": "wutong", "problem": "0001" }))["notes"].clone();
    assert!(notes
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["by"] == "wang" && n["text"] == "手机上写的。"));

    // The phone writes offline, then is lost; the laptop removes it everywhere.
    let url = phone.go_offline();
    phone.ok(
        "note",
        json!({ "house": "wutong", "problem": "0001", "text": "丢之前写的。" }),
    );
    phone.go_online(&url);
    let e = wang.err(
        "unpair",
        json!({ "device": d.iter().find(|x| x["thisDevice"] == false).unwrap()["fingerprint"] }),
    );
    assert!(
        e["reasons"][0]
            .as_str()
            .unwrap()
            .contains("device you are using"),
        "the laptop is the one in use: {e:#}"
    );
    let removed = wang.ok("unpair", json!({ "device": "手机" }));
    assert_eq!(removed, json!(["lanes", "wutong"]));
    assert_eq!(
        wang.ok("devices", json!({ "house": "lanes" }))
            .as_array()
            .unwrap()
            .len(),
        1
    );
    // What it wrote and did not push never lands; it cannot get in any more.
    let s = phone.ok("sync", json!({ "house": "wutong" }));
    assert_eq!(s["state"], "error", "{s:#}");
    wang.ok("sync", json!({ "house": "wutong" }));
    let notes =
        wang.ok("problem", json!({ "house": "wutong", "problem": "0001" }))["notes"].clone();
    assert!(!notes
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["text"] == "丢之前写的。"));
    // The laptop is wang's last device in lanes now: it stays.
    let fp = wang.ok("devices", json!({ "house": "lanes" }))[0]["fingerprint"].clone();
    let e = wang.err("unpair", json!({ "house": "lanes", "device": fp }));
    assert_eq!(e["code"], "relay_refused");
}

#[test]
fn two_devices_reworking_one_proposal_keep_both() {
    let w = World::new("devices-proposal");
    let (wang, li, zhao) = w.wutong();
    parking(&li);
    zhao.ok("sync", json!({ "house": "wutong" }));
    let p = zhao.ok("propose", json!({ "house": "wutong", "problem": "0001", "title": "画线", "changes": [{ "kind": "doc", "title": "画线", "text": "v1\n" }] }));
    let pid = p["id"].as_str().unwrap().to_string();
    let code = zhao.ok("pair", json!({}));
    let tablet = w.person("zhao-tablet");
    tablet.ok("redeem", json!({ "code": code, "name": "平板" }));

    // Each device reworks the proposal while the other does not know.
    let rework = |p: &Person, text: &str| {
        let repo = git2::Repository::open(p.client.root().join("houses/wutong")).unwrap();
        let name = format!("refs/proposals/{pid}");
        let tip = repo
            .find_commit(repo.refname_to_id(&name).unwrap())
            .unwrap();
        let tree = commit::tree_with(
            &repo,
            Some(&tip.tree().unwrap()),
            &[Edit::Put(
                "works/0001/画线.md".into(),
                format!("---\ntitle: 画线\n---\n{text}").into_bytes(),
            )],
        )
        .unwrap();
        let sig = Author {
            id: "zhao".into(),
            name: "小赵".into(),
        }
        .signature(1_792_400_000)
        .unwrap();
        let msg = commit::message(
            text,
            None,
            &[("Problem", "0001".into()), ("Act", "propose".into())],
        );
        let c = commit::signed_commit(
            &repo,
            &p.client.identity().unwrap(),
            &sig,
            tree,
            &[&tip],
            &msg,
        )
        .unwrap();
        commit::move_ref(&repo, &name, Some(tip.id()), Some(c), "test").unwrap();
    };
    let url = zhao.go_offline();
    rework(&tablet, "平板改的");
    tablet.ok("sync", json!({ "house": "wutong" }));
    rework(&zhao, "电脑改的");
    zhao.go_online(&url);
    let s = zhao.ok("sync", json!({ "house": "wutong" }));
    assert_eq!(s["state"], "synced", "{s:#}");

    // Nothing was overwritten: the tablet's version stays, the laptop's
    // became a proposal of its own.
    wang.ok("sync", json!({ "house": "wutong" }));
    let props = wang.house()["proposals"].clone();
    let titles: Vec<&str> = props
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            p["files"][0]["lines"].as_array().unwrap().last().unwrap()["text"]
                .as_str()
                .unwrap()
        })
        .collect();
    assert_eq!(props.as_array().unwrap().len(), 2, "{props:#}");
    assert!(
        titles.contains(&"平板改的") && titles.contains(&"电脑改的"),
        "{titles:?}"
    );
}
