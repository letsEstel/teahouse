//! The read commands against a real local clone, built commit by commit the
//! way clients would build it: problems, works of every kind, notices,
//! replies, posts pushed out of stamp order, an adopted proposal, a declined
//! one and an open one.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use teahouse_client::Client;
use teahouse_core::PublicKey;

const BIN: &str = env!("CARGO_BIN_EXE_teahouse-client");

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Tmp(PathBuf);

impl Tmp {
    fn new(tag: &str) -> Tmp {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let p = std::env::temp_dir().join(format!("thc-{tag}-{}-{n}-{nanos}", std::process::id()));
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

fn key_for(n: u8) -> PublicKey {
    PublicKey::from_point(&[n; 32])
}

/// 2026-10-18T00:00:00Z
const T0: i64 = 1_792_281_600;
const HOUR: i64 = 3600;

fn git(dir: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

struct House {
    dir: PathBuf,
}

impl House {
    fn init(dir: PathBuf) -> House {
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q", "-b", "main"]);
        git(&dir, &["config", "commit.gpgsign", "false"]);
        House { dir }
    }

    /// Writes (`Some`) or removes (`None`) files and commits everything as
    /// `who` at `at`. `msg` is the full message, trailers included.
    fn commit(&self, who: &str, at: i64, files: &[(&str, Option<&str>)], msg: &str) -> String {
        for (p, c) in files {
            let full = self.dir.join(p);
            match c {
                Some(c) => {
                    fs::create_dir_all(full.parent().unwrap()).unwrap();
                    fs::write(&full, c).unwrap();
                }
                None => fs::remove_file(&full).unwrap(),
            }
        }
        git(&self.dir, &["add", "-A"]);
        self.run_as(who, at, &["commit", "-q", "--allow-empty", "-m", msg]);
        git(&self.dir, &["rev-parse", "HEAD"])
    }

    fn run_as(&self, who: &str, at: i64, args: &[&str]) {
        let date = format!("@{at} +0000");
        let o = Command::new("git")
            .arg("-C")
            .arg(&self.dir)
            .args(args)
            .env("GIT_AUTHOR_NAME", who.to_uppercase())
            .env("GIT_AUTHOR_EMAIL", format!("{who}@teahouse"))
            .env("GIT_COMMITTER_NAME", who.to_uppercase())
            .env("GIT_COMMITTER_EMAIL", format!("{who}@teahouse"))
            .env("GIT_AUTHOR_DATE", &date)
            .env("GIT_COMMITTER_DATE", &date)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    }

    fn checkout(&self, args: &[&str]) {
        let mut a = vec!["checkout", "-q"];
        a.extend_from_slice(args);
        git(&self.dir, &a);
    }
}

fn members(extra_guest: bool) -> String {
    let mut t = String::from(
        "version = 1\n\n[teahouse]\nname = \"梧桐里三号院\"\nsignboard = \"车位、垃圾、楼道灯，咱们自己管。\"\n",
    );
    let mut seats = vec![
        ("wang", "王阿姨", "host", 1),
        ("li", "李师傅", "steward", 2),
    ];
    if extra_guest {
        seats.push(("zhao", "小赵", "guest", 3));
    }
    for (id, name, role, n) in seats {
        t.push_str(&format!(
            "\n[[member]]\nid = \"{id}\"\nname = \"{name}\"\nrole = \"{role}\"\nkey = \"{}\"\n",
            key_for(n).openssh()
        ));
    }
    t
}

const PROBLEM: &str = "---\ntitle: 小区停车位不好找\nstatus: open            # open | active | solved\n---\n## 现状\n院内共 42 个车位，晚上七点以后基本没位置。\n\n## 目标\n年底前让每户至少有一个固定车位。\n";

/// The standard teahouse of these tests. Returns the engine root.
fn world(tmp: &Tmp) -> (PathBuf, House) {
    let root = tmp.0.join("engine");
    let h = House::init(root.join("houses").join("wutong"));
    let hash = "ab".repeat(32);
    let image_ref = format!(
        "---\ntitle: 车位照片\nblob: sha256:{hash}\nsize: 482113\ntype: image/png\n---\n东门口\n"
    );
    let pdf_ref = format!(
        "---\ntitle: 物业合同\nblob: sha256:{}\nsize: 1000\ntype: application/pdf\npurged: 2026-10-19T00:00:00Z wang\n---\n",
        "cd".repeat(32)
    );

    let c1 = h.commit(
        "wang",
        T0,
        &[
            (".teahouse/members.toml", Some(&members(false))),
            ("README.md", Some("这是一间茶馆。\n")),
        ],
        "开张\n\nProblem: meta\nAct: seat",
    );
    h.commit(
        "wang",
        T0 + HOUR,
        &[("problems/0001-parking.md", Some(PROBLEM))],
        "提出问题：小区停车位不好找\n\nProblem: 0001\nAct: open-problem",
    );
    h.commit(
        "li",
        T0 + 2 * HOUR,
        &[
            (
                "works/0001/方案.md",
                Some("---\ntitle: 错峰停车方案\nkind: plan\n---\n白天让给外来车。\n"),
            ),
            ("works/0001/车位.csv", Some("楼号,车位\n1,12\n2,30\n")),
            (
                "works/0001/车位.csv.meta.md",
                Some("---\ntitle: 各楼车位统计\nnote: 42 个车位逐一标号\n---\n"),
            ),
            ("works/0001/车位照片.png.ref.md", Some(&image_ref)),
            ("works/0001/合同.pdf.ref.md", Some(&pdf_ref)),
            ("works/0001/旧照片.jpg", Some("\u{0}\u{1}binary")),
        ],
        "添加成果\n\nProblem: 0001\nAct: add-work",
    );
    h.commit(
        "wang",
        T0 + 3 * HOUR,
        &[(".teahouse/members.toml", Some(&members(true)))],
        "小赵入座\n\nProblem: meta\nAct: seat",
    );
    h.commit(
        "li",
        T0 + 4 * HOUR,
        &[(
            "board/water.md",
            Some("---\ntitle: 周六停水\n---\n周六上午九点到十二点停水。\n"),
        )],
        "张贴公告：周六停水\n\nProblem: board\nAct: notice",
    );
    h.commit(
        "zhao",
        T0 + 5 * HOUR,
        &[(
            "talk/board/water/20261018T050000Z-zhao.md",
            Some("知道了，谢谢。"),
        )],
        "回应公告\n\nProblem: board\nAct: note",
    );
    // Pushed first, written later…
    h.commit(
        "zhao",
        T0 + 6 * HOUR,
        &[(
            "talk/0001/20261018T060000Z-zhao.md",
            Some("晚上七点以后基本没位置。"),
        )],
        "留言\n\nProblem: 0001\nAct: note",
    );
    // …pushed second, written earlier (offline). Entry order wins.
    h.commit(
        "li",
        T0 + 7 * HOUR,
        &[(
            "talk/0001/20261017T230000Z-li.md",
            Some("我昨晚数过，空位只有三个。"),
        )],
        "留言\n\nProblem: 0001\nAct: note",
    );

    // An adopted proposal: its post enters main when the merge lands.
    h.checkout(&["-b", "p-adopt"]);
    h.commit(
        "zhao",
        T0 + 8 * HOUR,
        &[
            (
                "works/0001/建议.md",
                Some("---\ntitle: 夜间临停建议\n---\n晚上允许临停两小时。\n"),
            ),
            ("talk/0001/20261018T080000Z-zhao.md", Some("我提了个建议。")),
        ],
        "夜间临停建议\n\n外来车太多了，晚上更挤。\n\nProblem: 0001\nAct: propose",
    );
    h.checkout(&["main"]);
    h.run_as(
        "wang",
        T0 + 9 * HOUR,
        &[
            "merge",
            "-q",
            "--no-ff",
            "p-adopt",
            "-m",
            "采纳提案：夜间临停建议\n\nProblem: 0001\nAct: adopt\nProposal: zhao/night-parking-x1",
        ],
    );
    git(&h.dir, &["branch", "-q", "-D", "p-adopt"]);

    // A declined one leaves only an empty commit on main.
    h.commit(
        "wang",
        T0 + 10 * HOUR,
        &[],
        "不采纳提案：拆掉花坛\n\n花坛是大家一起种的。\n\nProblem: 0001\nAct: decline\nProposal: zhao/flowerbed-y2",
    );
    let mut active = PROBLEM.replace(
        "status: open            # open | active | solved",
        "status: active",
    );
    active = active.replace("---\n## 现状", "owner: li\n---\n## 现状");
    h.commit(
        "li",
        T0 + 11 * HOUR,
        &[("problems/0001-parking.md", Some(&active))],
        "开始处理\n\nProblem: 0001\nAct: set-status\nStatus: active",
    );
    // A commit made by hand, with no Act: trailer.
    h.commit(
        "li",
        T0 + 12 * HOUR,
        &[("works/0001/车位.csv", Some("楼号,车位\n1,12\n2,31\n"))],
        "更正 2 号楼车位数\n\nProblem: 0001",
    );

    // An open proposal, two commits on top of an older main.
    h.checkout(&["-b", "open-prop", "HEAD~1"]);
    h.commit(
        "zhao",
        T0 + 13 * HOUR,
        &[("works/0001/车位.csv", Some("楼号,车位\n1,12\n2,30\n3,8\n"))],
        "补上 3 号楼\n\n3 号楼也有车位。\n\nProblem: 0001\nAct: propose",
    );
    h.commit(
        "zhao",
        T0 + 14 * HOUR,
        &[("works/0001/说明.md", Some("数据来自物业。\n"))],
        "补充来源\n\nProblem: 0001\nAct: propose",
    );
    let tip = git(&h.dir, &["rev-parse", "HEAD"]);
    git(
        &h.dir,
        &["update-ref", "refs/proposals/zhao/building-3-k7", &tip],
    );
    h.checkout(&["main"]);
    git(&h.dir, &["branch", "-q", "-D", "open-prop"]);
    // A proposal ref already contained in main is not pending.
    git(&h.dir, &["update-ref", "refs/proposals/zhao/old-z9", &c1]);

    // What the relay has seen: everything up to the declined proposal.
    let pushed = git(&h.dir, &["rev-parse", "HEAD~2"]);
    git(&h.dir, &["update-ref", "refs/remotes/origin/main", &pushed]);

    // This device is 小赵's.
    fs::write(
        root.join("device.pub"),
        format!("{} zhao@phone\n", key_for(3).openssh()),
    )
    .unwrap();
    fs::create_dir_all(root.join("blobs")).unwrap();
    fs::write(root.join("blobs").join(&hash), b"PNG").unwrap();
    (root, h)
}

fn ok(c: &Client, cmd: &str, args: Value) -> Value {
    let v = c.call(cmd, args.clone());
    assert_eq!(v["ok"], true, "{cmd} {args}: {v:#}");
    v["data"].clone()
}

fn err(c: &Client, cmd: &str, args: Value) -> Value {
    let v = c.call(cmd, args.clone());
    assert_eq!(v["ok"], false, "{cmd} {args}: {v:#}");
    v["error"].clone()
}

fn when(t: i64) -> String {
    teahouse_core::rules::format_rfc3339(t)
}

#[test]
fn houses_lists_every_clone_with_my_seat_and_sync_state() {
    let tmp = Tmp::new("houses");
    let (root, _h) = world(&tmp);
    let c = Client::new(&root);
    let v = ok(&c, "houses", Value::Null);
    assert_eq!(
        v,
        json!([{
            "id": "wutong",
            "name": "梧桐里三号院",
            "me": "zhao",
            "myRole": "guest",
            "seats": 3,
            "pendingProposals": 1,
            "sync": { "state": "ahead", "unpushed": 2 },
        }])
    );
    // No engine data at all: no teahouses, not an error.
    assert_eq!(
        ok(&Client::new(tmp.0.join("nothing")), "houses", Value::Null),
        json!([])
    );
}

#[test]
fn house_has_members_problems_board_proposals_and_decisions() {
    let tmp = Tmp::new("house");
    let (root, _h) = world(&tmp);
    let c = Client::new(&root);
    let v = ok(&c, "house", json!({ "house": "wutong" }));

    assert_eq!(v["name"], "梧桐里三号院");
    assert_eq!(v["signboard"], "车位、垃圾、楼道灯，咱们自己管。");
    assert_eq!(v["founded"], when(T0));
    assert_eq!(v["capacity"], 15);
    assert_eq!(v["me"], "zhao");
    assert_eq!(
        v["members"],
        json!([
            { "id": "wang", "name": "王阿姨", "role": "host", "kind": "human", "joined": when(T0) },
            { "id": "li", "name": "李师傅", "role": "steward", "kind": "human", "joined": when(T0) },
            { "id": "zhao", "name": "小赵", "role": "guest", "kind": "human", "joined": when(T0 + 3 * HOUR) },
        ])
    );

    assert_eq!(
        v["problems"],
        json!([{
            "id": "0001",
            "title": "小区停车位不好找",
            "summary": "院内共 42 个车位，晚上七点以后基本没位置。",
            "status": "active",
            "by": "wang",
            "owner": "li",
            "opened": when(T0 + HOUR),
            "updated": when(T0 + 12 * HOUR),
            // plan, table, image, pdf, old binary, adopted doc; not the table's meta
            "workCount": 6,
            "pendingProposals": 1,
        }])
    );

    let board = &v["board"];
    assert_eq!(board.as_array().unwrap().len(), 1);
    assert_eq!(board[0]["id"], "water");
    assert_eq!(board[0]["title"], "周六停水");
    assert_eq!(board[0]["text"], "周六上午九点到十二点停水。");
    assert_eq!(board[0]["by"], "li");
    assert_eq!(
        board[0]["replies"],
        json!([{
            "id": "talk/board/water/20261018T050000Z-zhao.md",
            "by": "zhao",
            "text": "知道了，谢谢。",
            "when": when(T0 + 5 * HOUR),
            "writtenAt": "2026-10-18T05:00:00Z",
        }])
    );

    // Newest first; the merge is one event, the inferred act is "update-work".
    let recent = v["recent"].as_array().unwrap();
    assert_eq!(recent.len(), 12);
    assert_eq!(
        recent[0],
        json!({ "by": "li", "act": "update-work", "title": "更正 2 号楼车位数", "when": when(T0 + 12 * HOUR), "problem": "0001" })
    );
    assert_eq!(recent[1]["act"], "set-status");
    assert_eq!(recent[1]["status"], "active");
    assert_eq!(recent[3]["act"], "adopt");
    assert_eq!(recent[11]["act"], "seat");
    assert!(recent[11].get("problem").is_none());

    let proposals = v["proposals"].as_array().unwrap();
    assert_eq!(proposals.len(), 1, "{proposals:#?}");
    let p = &proposals[0];
    assert_eq!(p["id"], "zhao/building-3-k7");
    assert_eq!(p["title"], "补上 3 号楼");
    assert_eq!(p["reason"], "3 号楼也有车位。");
    assert_eq!(p["by"], "zhao");
    assert_eq!(p["problem"], "0001");
    assert_eq!(p["when"], when(T0 + 14 * HOUR));
    let files = p["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0]["path"], "works/0001/说明.md");
    assert_eq!(files[0]["change"], "add");
    assert_eq!(files[0]["kind"], "doc");
    assert_eq!(
        files[0]["lines"],
        json!([{ "kind": "add", "text": "数据来自物业。" }])
    );
    assert_eq!(files[1]["path"], "works/0001/车位.csv");
    assert_eq!(files[1]["change"], "modify");
    assert_eq!(files[1]["kind"], "table");
    assert_eq!(
        files[1]["lines"],
        json!([
            { "kind": "same", "text": "楼号,车位" },
            { "kind": "same", "text": "1,12" },
            { "kind": "same", "text": "2,30" },
            { "kind": "add", "text": "3,8" },
        ])
    );

    assert_eq!(
        v["decided"],
        json!([
            {
                "id": "zhao/flowerbed-y2", "title": "拆掉花坛", "by": "zhao",
                "result": "declined", "when": when(T0 + 10 * HOUR),
                "reason": "花坛是大家一起种的。",
            },
            {
                "id": "zhao/night-parking-x1", "title": "夜间临停建议", "by": "zhao",
                "result": "adopted", "when": when(T0 + 9 * HOUR),
                "reason": "外来车太多了，晚上更挤。",
            },
        ])
    );
    assert_eq!(v["sync"], json!({ "state": "ahead", "unpushed": 2 }));
}

#[test]
fn problem_has_body_works_events_and_notes_in_entry_order() {
    let tmp = Tmp::new("problem");
    let (root, _h) = world(&tmp);
    let c = Client::new(&root);
    let v = ok(
        &c,
        "problem",
        json!({ "house": "wutong", "problem": "0001" }),
    );

    assert_eq!(v["id"], "0001");
    assert_eq!(v["status"], "active");
    assert_eq!(
        v["body"],
        json!([
            { "heading": "现状", "text": "院内共 42 个车位，晚上七点以后基本没位置。" },
            { "heading": "目标", "text": "年底前让每户至少有一个固定车位。" },
        ])
    );

    let works: Vec<&Value> = v["works"].as_array().unwrap().iter().collect();
    let by_id = |id: &str| -> &Value {
        works
            .iter()
            .find(|w| w["id"] == format!("works/0001/{id}"))
            .unwrap_or_else(|| panic!("no work {id}: {works:#?}"))
    };
    assert_eq!(works.len(), 6);
    let plan = by_id("方案.md");
    assert_eq!(plan["kind"], "plan");
    assert_eq!(plan["title"], "错峰停车方案");
    assert_eq!(plan["text"], "白天让给外来车。\n");
    assert_eq!(plan["by"], "li");
    let table = by_id("车位.csv");
    assert_eq!(table["kind"], "table");
    assert_eq!(table["title"], "各楼车位统计");
    assert_eq!(table["note"], "42 个车位逐一标号");
    assert!(plan.get("note").is_none());
    assert_eq!(table["text"], "楼号,车位\n1,12\n2,31\n");
    assert_eq!(table["when"], when(T0 + 12 * HOUR));
    let img = by_id("车位照片.png.ref.md");
    assert_eq!(img["kind"], "image");
    assert_eq!(img["title"], "车位照片");
    assert_eq!(img["size"], 482113);
    assert!(img.get("text").is_none());
    assert_eq!(
        img["blob"],
        json!({ "hash": format!("sha256:{}", "ab".repeat(32)), "type": "image/png", "cached": true })
    );
    let pdf = by_id("合同.pdf.ref.md");
    assert_eq!(pdf["kind"], "file");
    assert_eq!(pdf["blob"]["purged"], true);
    assert_eq!(pdf["blob"]["cached"], false);
    let old = by_id("旧照片.jpg");
    assert_eq!(old["kind"], "file");
    assert_eq!(old["size"], 8);
    assert!(old.get("blob").is_none());
    let adopted = by_id("建议.md");
    assert_eq!(
        adopted["by"], "wang",
        "a work arrives with the merge that lands it"
    );
    assert_eq!(adopted["when"], when(T0 + 9 * HOUR));

    // Entry order, not stamp order: zhao's (stamped 06:00) entered before
    // li's (stamped the day before); the adopted post entered with the merge.
    let notes = v["notes"].as_array().unwrap();
    let order: Vec<(&str, &str)> = notes
        .iter()
        .map(|n| (n["by"].as_str().unwrap(), n["writtenAt"].as_str().unwrap()))
        .collect();
    assert_eq!(
        order,
        [
            ("zhao", "2026-10-18T06:00:00Z"),
            ("li", "2026-10-17T23:00:00Z"),
            ("zhao", "2026-10-18T08:00:00Z"),
        ]
    );
    assert_eq!(notes[2]["when"], when(T0 + 9 * HOUR));

    let acts: Vec<&str> = v["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["act"].as_str().unwrap())
        .collect();
    assert_eq!(
        acts,
        [
            "open-problem",
            "add-work",
            "note",
            "note",
            "adopt",
            "decline",
            "set-status",
            "update-work"
        ]
    );

    let e = err(
        &c,
        "problem",
        json!({ "house": "wutong", "problem": "0002" }),
    );
    assert_eq!(e["code"], "not_found");
    assert_eq!(e["params"], json!({ "problem": "0002" }));
}

#[test]
fn archive_in_both_views() {
    let tmp = Tmp::new("archive");
    let (root, _h) = world(&tmp);
    let c = Client::new(&root);
    // Directory kinds are codes for the interface to translate; labels are
    // only what people wrote.
    let shape = |n: &Value| (n["id"].clone(), n["dir"].clone(), n["label"].clone());

    let dirs = ok(&c, "archive", json!({ "house": "wutong", "view": "dirs" }));
    assert_eq!(dirs["kind"], "dir");
    assert_eq!(dirs["label"], "梧桐里三号院");
    let top: Vec<_> = dirs["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(shape)
        .collect();
    let n = Value::Null;
    assert_eq!(
        top,
        [
            (json!(".teahouse"), json!("meta"), n.clone()),
            (json!("board"), json!("board"), n.clone()),
            (json!("problems"), json!("problems"), n.clone()),
            (json!("talk"), json!("talk"), n.clone()),
            (json!("works"), json!("works"), n.clone()),
        ]
    );
    // The teahouse's own files, README.md included, are under meta.
    let meta_dir: Vec<(&str, &str)> = dirs["children"][0]["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["id"].as_str().unwrap(), n["kind"].as_str().unwrap()))
        .collect();
    assert_eq!(
        meta_dir,
        [(".teahouse/members.toml", "members"), ("README.md", "doc")]
    );
    let members = &dirs["children"][0]["children"][0];
    assert!(members.get("label").is_none());
    let works0001 = &dirs["children"][4]["children"][0];
    assert_eq!(
        shape(works0001),
        (json!("works/0001"), n.clone(), json!("小区停车位不好找"))
    );
    assert_eq!(works0001["problem"], "0001");
    // A table's title file is not listed on its own (§1.2).
    assert!(works0001["children"]
        .as_array()
        .unwrap()
        .iter()
        .all(|n| n["id"] != "works/0001/车位.csv.meta.md"));
    let talk = &dirs["children"][3]["children"];
    assert_eq!(
        shape(&talk[1]),
        (json!("talk/board"), json!("board"), n.clone())
    );
    assert_eq!(talk[1]["children"][0]["label"], "周六停水");

    let probs = ok(
        &c,
        "archive",
        json!({ "house": "wutong", "view": "problems" }),
    );
    let p = &probs["children"][0];
    assert_eq!(p["kind"], "problem");
    assert_eq!(p["label"], "小区停车位不好找");
    assert_eq!(p["problem"], "0001");
    let kids: Vec<(&str, &str)> = p["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["kind"].as_str().unwrap(), n["label"].as_str().unwrap()))
        .collect();
    assert!(kids.contains(&("plan", "错峰停车方案")), "{kids:?}");
    assert!(kids.contains(&("image", "车位照片")), "{kids:?}");
    assert!(kids.contains(&("table", "各楼车位统计")), "{kids:?}");
    assert_eq!(
        kids.len(),
        6 + 3,
        "six works, three notes, no title file: {kids:?}"
    );
    // Notes in entry order; the label is the member's name, the id has the stamp.
    let notes: Vec<(&str, &str)> = p["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["kind"] == "note")
        .map(|n| (n["label"].as_str().unwrap(), n["id"].as_str().unwrap()))
        .collect();
    assert_eq!(
        notes,
        [
            ("小赵", "talk/0001/20261018T060000Z-zhao.md"),
            ("李师傅", "talk/0001/20261017T230000Z-li.md"),
            ("小赵", "talk/0001/20261018T080000Z-zhao.md"),
        ]
    );
    let board = &probs["children"][1];
    assert_eq!(shape(board), (json!("board"), json!("board"), n.clone()));
    assert_eq!(board["children"][0]["label"], "周六停水");
    assert_eq!(board["children"][0]["children"][0]["kind"], "note");
    let meta = &probs["children"][2];
    assert_eq!(shape(meta), (json!(".teahouse"), json!("meta"), n.clone()));
    let rest: Vec<&str> = meta["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["kind"].as_str().unwrap())
        .collect();
    assert_eq!(rest, ["members", "doc"]);
    assert_eq!(probs["children"].as_array().unwrap().len(), 3);

    let e = err(&c, "archive", json!({ "house": "wutong", "view": "tree" }));
    assert_eq!(e["code"], "invalid");
}

#[test]
fn file_has_text_and_history_with_diffs() {
    let tmp = Tmp::new("file");
    let (root, _h) = world(&tmp);
    let c = Client::new(&root);
    let v = ok(
        &c,
        "file",
        json!({ "house": "wutong", "id": "works/0001/车位.csv" }),
    );
    assert_eq!(v["node"]["kind"], "table");
    assert_eq!(v["node"]["label"], "各楼车位统计");
    assert!(v["node"].get("dir").is_none());
    assert_eq!(v["text"], "楼号,车位\n1,12\n2,31\n");
    let h = v["history"].as_array().unwrap();
    assert_eq!(h.len(), 2);
    assert_eq!(h[0]["act"], "add-work");
    assert_eq!(h[0]["by"], "li");
    assert_eq!(h[1]["title"], "更正 2 号楼车位数");
    assert!(h[1].get("status").is_none());
    assert_eq!(h[1]["problem"], "0001");
    assert_eq!(
        h[1]["diff"],
        json!([
            { "kind": "same", "text": "楼号,车位" },
            { "kind": "same", "text": "1,12" },
            { "kind": "del", "text": "2,30" },
            { "kind": "add", "text": "2,31" },
        ])
    );

    let p = ok(
        &c,
        "file",
        json!({ "house": "wutong", "id": "problems/0001-parking.md" }),
    );
    let last = p["history"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(
        (&last["act"], &last["status"]),
        (&json!("set-status"), &json!("active"))
    );

    let bin = ok(
        &c,
        "file",
        json!({ "house": "wutong", "id": "works/0001/旧照片.jpg" }),
    );
    assert!(bin.get("text").is_none());
    assert!(bin["history"][0].get("diff").is_none());

    let e = err(
        &c,
        "file",
        json!({ "house": "wutong", "id": "works/0001/nope.md" }),
    );
    assert_eq!(e["code"], "not_found");
}

#[test]
fn blobs_come_from_the_local_cache() {
    let tmp = Tmp::new("blob");
    let (root, _h) = world(&tmp);
    let c = Client::new(&root);
    let path = ok(
        &c,
        "blob",
        json!({ "house": "wutong", "id": "works/0001/车位照片.png.ref.md" }),
    );
    assert_eq!(fs::read(path.as_str().unwrap()).unwrap(), b"PNG");
    let purged = err(
        &c,
        "blob",
        json!({ "house": "wutong", "id": "works/0001/合同.pdf.ref.md" }),
    );
    assert_eq!(purged["code"], "not_found");
    fs::remove_file(root.join("blobs").join("ab".repeat(32))).unwrap();
    let missing = err(
        &c,
        "blob",
        json!({ "house": "wutong", "id": "works/0001/车位照片.png.ref.md" }),
    );
    assert_eq!(missing["code"], "offline");
}

#[test]
fn bad_requests_are_invalid_or_not_found() {
    let tmp = Tmp::new("bad");
    let (root, _h) = world(&tmp);
    let c = Client::new(&root);
    for house in ["nope", "../wutong", ".git", ""] {
        let e = err(&c, "house", json!({ "house": house }));
        assert_eq!(e["code"], "not_found", "{house}");
    }
    assert_eq!(err(&c, "house", json!({}))["code"], "invalid");
    assert_eq!(err(&c, "frobnicate", Value::Null)["code"], "invalid");
    assert_eq!(
        err(&c, "note", json!({ "house": "wutong" }))["code"],
        "invalid"
    );
}

#[test]
fn a_device_without_a_seat_reads_as_nobody() {
    let tmp = Tmp::new("unseated");
    let (root, _h) = world(&tmp);
    fs::write(
        root.join("device.pub"),
        format!("{}\n", key_for(9).openssh()),
    )
    .unwrap();
    let c = Client::new(&root);
    let v = ok(&c, "houses", Value::Null);
    assert_eq!(v[0]["me"], "");
    assert_eq!(v[0]["myRole"], "guest");
}

#[test]
fn the_command_line_prints_the_envelope() {
    let tmp = Tmp::new("cli");
    let (root, _h) = world(&tmp);
    let out = Command::new(BIN)
        .arg("--root")
        .arg(&root)
        .args(["call", "problem", r#"{"house":"wutong","problem":"0001"}"#])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["data"]["title"], "小区停车位不好找");

    let out = Command::new(BIN)
        .arg("--root")
        .arg(&root)
        .args(["call", "house", r#"{"house":"nope"}"#])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["error"]["code"], "not_found");
}

/// The relay judges `Problem:` trailers and changed paths with the git
/// program; the engine reads them with libgit2. They must agree, including
/// on messages that only look like they have trailers.
#[test]
fn libgit2_agrees_with_git_on_trailers_and_first_parent_changes() {
    let tmp = Tmp::new("agree");
    let h = House::init(tmp.0.join("h"));
    let messages = [
        "plain\n\nProblem: 0001",
        "two trailers\n\nProblem: 0001\nProblem: 0002",
        "lowercase key\n\nproblem: 0003",
        "spaces\n\nProblem :   0004  ",
        "folded\n\nProblem: 00\n 05",
        "mixed paragraph\n\nProblem: 0006 is mentioned here\nbut this is prose",
        "trailer then prose\n\nProblem: 0007\n\nAnd a closing remark.",
        "only subject Problem: 0008",
        "with sign-off\n\nSome text.\n\nProblem: 0009\nSigned-off-by: A <a@b>\nCo-authored-by: C <c@d>",
        "comment-ish\n\n# Problem: 0010\nProblem: 0011",
        "unicode\n\n理由：没有\n\nProblem: 0012\nAct: note",
        "Subject\non two lines\n\nProblem: 0013",
    ];
    for (i, m) in messages.iter().enumerate() {
        h.commit(
            "wang",
            T0 + i as i64,
            &[(&format!("f{i}.txt"), Some("x"))],
            m,
        );
    }
    // A merge whose second parent brings in a file, and a removal.
    h.checkout(&["-b", "side"]);
    h.commit(
        "zhao",
        T0 + 100,
        &[("side/a.md", Some("a"))],
        "side\n\nProblem: 0001",
    );
    h.checkout(&["main"]);
    h.commit(
        "wang",
        T0 + 101,
        &[("f0.txt", None)],
        "remove\n\nProblem: 0001",
    );
    h.run_as(
        "wang",
        T0 + 102,
        &[
            "merge",
            "-q",
            "--no-ff",
            "side",
            "-m",
            "merge\n\nProblem: 0001",
        ],
    );

    let ours = teahouse_client::git::Git::open(&h.dir)
        .unwrap()
        .first_parent_steps(&git(&h.dir, &["rev-parse", "main"]))
        .unwrap();
    let hashes = git(&h.dir, &["rev-list", "--first-parent", "--reverse", "main"]);
    let hashes: Vec<&str> = hashes.lines().collect();
    assert_eq!(ours.len(), hashes.len());
    for (step, hash) in ours.iter().zip(hashes) {
        assert_eq!(step.hash, hash);
        let theirs = git(
            &h.dir,
            &[
                "log",
                "-1",
                "--format=%(trailers:key=Problem,valueonly,unfold,separator=%x1f)",
                hash,
            ],
        );
        let theirs: Vec<String> = theirs
            .split('\u{1f}')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect();
        assert_eq!(step.problem, theirs, "Problem trailers of {hash}");
        assert_eq!(
            step.subject,
            git(&h.dir, &["log", "-1", "--format=%s", hash]),
            "{hash}"
        );
        // RELAY.md "显示顺序": compare with the first parent explicitly.
        let first = format!("{hash}^");
        let mut args = vec![
            "-c",
            "core.quotePath=false",
            "diff-tree",
            "-r",
            "--no-renames",
        ];
        args.extend(["--no-commit-id", "--name-status"]);
        if step.parents.is_empty() {
            args.extend(["--root", hash]);
        } else {
            args.extend([first.as_str(), hash]);
        }
        let paths = git(&h.dir, &args);
        let mut theirs: Vec<(char, String)> = paths
            .lines()
            .filter_map(|l| l.split_once('\t'))
            .map(|(s, p)| (s.chars().next().unwrap(), p.to_string()))
            .collect();
        let mut mine = step.changes.clone();
        theirs.sort();
        mine.sort();
        assert_eq!(mine, theirs, "changes of {hash}");
    }
}
