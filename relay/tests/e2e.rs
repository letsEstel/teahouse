//! End-to-end tests: real `git` clients talk to a real relay through the same
//! forced-command path OpenSSH uses (`local-ssh` stands in for sshd only).
//! Every law of the teahouse is exercised, including deliberate attacks.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use teahouse_core::PublicKey;

const BIN: &str = env!("CARGO_BIN_EXE_teahouse-relay");

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Tmp(PathBuf);

impl Tmp {
    fn new(tag: &str) -> Tmp {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let p = std::env::temp_dir().join(format!("th-{tag}-{}-{n}-{nanos}", std::process::id()));
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

struct World {
    tmp: Tmp,
    root: PathBuf,
    keys: PathBuf,
}

fn s(o: &Output) -> String {
    format!(
        "status {}\nstdout:\n{}\nstderr:\n{}",
        o.status,
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn key_for(n: u8) -> PublicKey {
    PublicKey::from_point(&[n; 32])
}

impl World {
    fn new() -> World {
        let tmp = Tmp::new("e2e");
        let root = tmp.0.join("relay");
        let keys = tmp.0.join("keys");
        fs::create_dir_all(&keys).unwrap();
        let w = World { tmp, root, keys };
        w.relay_ok(&["init", "--binary", BIN]);
        w
    }

    fn relay(&self, args: &[&str]) -> Output {
        Command::new(BIN)
            .arg("--root")
            .arg(&self.root)
            .args(args)
            .env("TEAHOUSE_LOCAL_KEYS", &self.keys)
            .env_remove("TEAHOUSE_ROOT")
            .output()
            .unwrap()
    }

    fn relay_ok(&self, args: &[&str]) -> String {
        let o = self.relay(args);
        assert!(o.status.success(), "relay {args:?} failed:\n{}", s(&o));
        String::from_utf8_lossy(&o.stdout).into_owned()
    }

    /// Creates `<name>.pub` and returns the key.
    fn person(&self, name: &str, n: u8) -> PublicKey {
        let k = key_for(n);
        fs::write(
            self.keys.join(format!("{name}.pub")),
            format!("{} {name}@test\n", k.openssh()),
        )
        .unwrap();
        k
    }

    fn key_file(&self, name: &str) -> String {
        format!("@{}", self.keys.join(format!("{name}.pub")).display())
    }

    fn git(&self, dir: &Path, args: &[&str]) -> Output {
        Command::new("git")
            .current_dir(dir)
            .args(args)
            .env(
                "GIT_SSH_COMMAND",
                format!("{BIN} --root {} local-ssh", self.root.display()),
            )
            .env("GIT_SSH_VARIANT", "simple")
            .env("TEAHOUSE_LOCAL_KEYS", &self.keys)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_AUTHOR_DATE", "2026-10-18T12:00:00Z")
            .env("GIT_COMMITTER_DATE", "2026-10-18T12:00:00Z")
            .env_remove("GIT_DIR")
            .env_remove("TEAHOUSE_KEY")
            .env_remove("TEAHOUSE_HOUSE")
            .output()
            .unwrap()
    }

    fn git_ok(&self, dir: &Path, args: &[&str]) -> String {
        let o = self.git(dir, args);
        assert!(
            o.status.success(),
            "git {args:?} in {} failed:\n{}",
            dir.display(),
            s(&o)
        );
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }

    fn clone(&self, who: &str, house: &str) -> PathBuf {
        let dir = self.tmp.0.join(format!(
            "{who}-{house}-{}",
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let url = format!("ssh://{who}@local/{house}");
        let o = self.git(
            &self.tmp.0,
            &["clone", "--quiet", &url, dir.to_str().unwrap()],
        );
        assert!(
            o.status.success(),
            "{who} could not clone {house}:\n{}",
            s(&o)
        );
        self.git_ok(&dir, &["config", "user.name", who]);
        self.git_ok(&dir, &["config", "user.email", &format!("{who}@test")]);
        dir
    }

    fn try_clone(&self, who: &str, house: &str) -> Output {
        let dir = self.tmp.0.join(format!(
            "try-{who}-{}",
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let url = format!("ssh://{who}@local/{house}");
        self.git(
            &self.tmp.0,
            &["clone", "--quiet", &url, dir.to_str().unwrap()],
        )
    }

    fn write(&self, dir: &Path, path: &str, content: &str) {
        let p = dir.join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }

    fn commit(&self, dir: &Path, msg: &str) -> String {
        self.git_ok(dir, &["add", "-A"]);
        self.git_ok(dir, &["commit", "--quiet", "--allow-empty", "-m", msg]);
        self.git_ok(dir, &["rev-parse", "HEAD"])
    }

    fn push(&self, dir: &Path, args: &[&str]) -> Output {
        let mut full = vec!["push", "--quiet", "origin"];
        full.extend_from_slice(args);
        self.git(dir, &full)
    }

    fn push_ok(&self, dir: &Path, args: &[&str]) {
        let o = self.push(dir, args);
        assert!(
            o.status.success(),
            "push {args:?} should succeed:\n{}",
            s(&o)
        );
    }

    /// Asserts the push is refused and the reason mentions `needle`.
    fn push_denied(&self, dir: &Path, args: &[&str], needle: &str) {
        let o = self.push(dir, args);
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(
            !o.status.success(),
            "push {args:?} should be refused:\n{}",
            s(&o)
        );
        assert!(
            err.contains(needle),
            "refusal should mention {needle:?}:\n{err}"
        );
    }

    fn main_oid(&self, house: &str) -> String {
        let out = Command::new("git")
            .arg("--git-dir")
            .arg(self.root.join("houses").join(format!("{house}.git")))
            .args(["rev-parse", "refs/heads/main"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn members_toml(&self, entries: &[(&str, &str, u8)]) -> String {
        let mut t = String::from("version = 1\n\n[teahouse]\nname = \"梧桐里三号院\"\nsignboard = \"车位、垃圾、楼道灯，咱们自己管。\"\n");
        for (id, role, n) in entries {
            t.push_str(&format!(
                "\n[[member]]\nid = \"{id}\"\nname = \"{}\"\nrole = \"{role}\"\nkey = \"{}\"\n",
                id.to_uppercase(),
                key_for(*n).openssh()
            ));
        }
        t
    }

    /// The standard teahouse: host chenli, steward zhou, guests wang and li,
    /// problem 0001. Returns the host's clone.
    fn wutong(&self) -> PathBuf {
        self.person("chenli", 1);
        self.person("zhou", 2);
        self.person("wang", 3);
        self.person("li", 4);
        self.person("outsider", 99);
        self.relay_ok(&[
            "open",
            "wutong",
            "--name",
            "梧桐里三号院",
            "--signboard",
            "车位、垃圾、楼道灯，咱们自己管。",
            "--host-id",
            "chenli",
            "--host-name",
            "陈立",
            "--host-key",
            &self.key_file("chenli"),
        ]);
        let host = self.clone("chenli", "wutong");
        self.write(
            &host,
            ".teahouse/members.toml",
            &self.members_toml(&[
                ("chenli", "host", 1),
                ("zhou", "steward", 2),
                ("wang", "guest", 3),
                ("li", "guest", 4),
            ]),
        );
        self.write(&host, "problems/0001-停车位.md", "# 小区停车位不好找\n");
        self.commit(&host, "入座：周、王、李；提出问题 0001\n\nProblem: meta");
        self.push_ok(&host, &["main"]);
        host
    }

    fn audit(&self) -> String {
        fs::read_to_string(self.root.join("log/audit.log")).unwrap_or_default()
    }
}

// ───────────────────────────── the happy path ─────────────────────────────

#[test]
fn a_teahouse_opens_seats_people_and_gets_work_done() {
    let w = World::new();
    let host = w.wutong();
    assert!(w.relay_ok(&["members", "wutong"]).contains("steward  zhou"));

    // The steward lands a result directly.
    let zhou = w.clone("zhou", "wutong");
    w.write(&zhou, "works/0001/车位分布图.md", "42 个车位\n");
    w.commit(&zhou, "添加成果：车位分布图\n\nProblem: 0001");
    w.push_ok(&zhou, &["main"]);

    // The steward may open a new problem in the same commit that names it.
    w.write(&zhou, "problems/0002-垃圾点.md", "# 垃圾分类点的气味\n");
    w.commit(&zhou, "提出问题：垃圾点\n\nProblem: 0002");
    w.write(&zhou, "board/2026-10-18.md", "周六上午楼下集合\n");
    w.commit(&zhou, "贴公告\n\nProblem: 0002");
    w.push_ok(&zhou, &["main"]);

    // A guest proposes; the host adopts the proposal into main.
    let wang = w.clone("wang", "wutong");
    w.write(&wang, "works/0001/错峰共享方案.md", "07:00–19:00 …\n");
    w.commit(&wang, "提案：错峰共享\n\nProblem: 0001");
    w.push_ok(&wang, &["HEAD:refs/proposals/wang/night-share"]);

    w.git_ok(
        &host,
        &[
            "fetch",
            "--quiet",
            "origin",
            "main",
            "refs/proposals/wang/night-share:refs/remotes/origin/p",
        ],
    );
    w.git_ok(&host, &["reset", "--quiet", "--hard", "origin/main"]);
    w.git_ok(
        &host,
        &[
            "merge",
            "--quiet",
            "--no-ff",
            "origin/p",
            "-m",
            "采纳提案：错峰共享\n\nProblem: 0001",
        ],
    );
    w.push_ok(&host, &["main", ":refs/proposals/wang/night-share"]);

    // Everyone sees it.
    let li = w.clone("li", "wutong");
    assert!(li.join("works/0001/错峰共享方案.md").exists());
    assert!(li.join("board/2026-10-18.md").exists());

    w.relay_ok(&["check"]);
    let log = w.audit();
    assert!(log.contains("\"event\":\"house.opened\""));
    assert!(log.contains("\"event\":\"push.allowed\""));
}

// ─────────────────────────── every commit faces a problem ───────────────────────────

#[test]
fn commits_must_face_an_existing_problem() {
    let w = World::new();
    let _ = w.wutong();
    let zhou = w.clone("zhou", "wutong");

    w.write(&zhou, "works/0001/a.md", "a");
    w.commit(&zhou, "no trailer here");
    w.push_denied(&zhou, &["main"], "no `Problem:` trailer");

    w.git_ok(
        &zhou,
        &["commit", "--quiet", "--amend", "-m", "x\n\nProblem: 0042"],
    );
    w.push_denied(&zhou, &["main"], "problem 0042 does not exist");

    w.git_ok(
        &zhou,
        &["commit", "--quiet", "--amend", "-m", "x\n\nProblem: 1"],
    );
    w.push_denied(&zhou, &["main"], "not a 4-digit problem id");

    w.git_ok(
        &zhou,
        &[
            "commit",
            "--quiet",
            "--amend",
            "-m",
            "x\n\nProblem: 0001\nProblem: 0001",
        ],
    );
    w.push_denied(&zhou, &["main"], "exactly one is required");

    w.git_ok(
        &zhou,
        &["commit", "--quiet", "--amend", "-m", "x\n\nProblem: 0001"],
    );
    w.push_ok(&zhou, &["main"]);
}

// ───────────────────────────── the steward's limits ─────────────────────────────

#[test]
fn steward_writes_only_problems_works_and_board() {
    let w = World::new();
    let _ = w.wutong();
    let before = w.main_oid("wutong");
    let zhou = w.clone("zhou", "wutong");

    w.write(&zhou, "README.md", "taken over\n");
    w.commit(&zhou, "x\n\nProblem: 0001");
    w.push_denied(&zhou, &["main"], "the steward may only change");

    w.git_ok(&zhou, &["reset", "--quiet", "--hard", "origin/main"]);
    w.write(&zhou, "works/0001/ok.md", "fine");
    w.commit(&zhou, "meta from steward\n\nProblem: meta");
    w.push_denied(&zhou, &["main"], "only the host may land it");

    assert_eq!(w.main_oid("wutong"), before, "nothing landed");
}

#[test]
fn steward_cannot_promote_themselves() {
    let w = World::new();
    let _ = w.wutong();
    let zhou = w.clone("zhou", "wutong");
    w.write(
        &zhou,
        ".teahouse/members.toml",
        &w.members_toml(&[
            ("chenli", "guest", 1),
            ("zhou", "host", 2),
            ("wang", "guest", 3),
            ("li", "guest", 4),
        ]),
    );
    w.commit(&zhou, "coup\n\nProblem: 0001");
    w.push_denied(&zhou, &["main"], "not .teahouse/members.toml");
}

#[test]
fn steward_cannot_sneak_a_change_in_and_out_again() {
    // Touch members.toml in one commit, revert it in the next: the net diff is
    // empty, but history would carry a forged seating chart. Refused.
    let w = World::new();
    let _ = w.wutong();
    let zhou = w.clone("zhou", "wutong");
    let original = fs::read_to_string(zhou.join(".teahouse/members.toml")).unwrap();
    w.write(
        &zhou,
        ".teahouse/members.toml",
        &original.replace("role = \"guest\"", "role = \"guest\" # ok"),
    );
    w.commit(&zhou, "a\n\nProblem: 0001");
    w.write(&zhou, ".teahouse/members.toml", &original);
    w.commit(&zhou, "b\n\nProblem: 0001");
    w.push_denied(&zhou, &["main"], ".teahouse/members.toml");
}

#[test]
fn steward_evil_merge_is_caught() {
    // A merge whose *resolution* adds a change neither parent had.
    let w = World::new();
    let _ = w.wutong();
    let zhou = w.clone("zhou", "wutong");
    w.git_ok(&zhou, &["checkout", "--quiet", "-b", "side"]);
    w.write(&zhou, "works/0001/side.md", "side");
    w.commit(&zhou, "side\n\nProblem: 0001");
    w.git_ok(&zhou, &["checkout", "--quiet", "main"]);
    w.write(&zhou, "works/0001/main.md", "main");
    w.commit(&zhou, "main\n\nProblem: 0001");
    w.git_ok(
        &zhou,
        &["merge", "--quiet", "--no-commit", "--no-ff", "side"],
    );
    w.write(&zhou, "README.md", "smuggled in the merge\n");
    w.git_ok(&zhou, &["add", "-A"]);
    w.git_ok(
        &zhou,
        &["commit", "--quiet", "-m", "merge\n\nProblem: 0001"],
    );
    w.push_denied(&zhou, &["main"], "README.md");

    // The same merge without the smuggled change is fine.
    w.git_ok(&zhou, &["reset", "--quiet", "--hard", "HEAD~1"]);
    w.git_ok(
        &zhou,
        &[
            "merge",
            "--quiet",
            "--no-ff",
            "side",
            "-m",
            "merge\n\nProblem: 0001",
        ],
    );
    w.push_ok(&zhou, &["main"]);
}

#[test]
fn steward_cannot_land_a_guest_proposal_that_touches_seats() {
    let w = World::new();
    let _ = w.wutong();
    let wang = w.clone("wang", "wutong");
    w.write(
        &wang,
        ".teahouse/members.toml",
        &w.members_toml(&[
            ("chenli", "host", 1),
            ("zhou", "steward", 2),
            ("wang", "guest", 3),
            ("li", "guest", 4),
            ("newbie", "guest", 5),
        ]),
    );
    w.commit(&wang, "请邀请 newbie 入座\n\nProblem: meta");
    w.push_ok(&wang, &["HEAD:refs/proposals/wang/invite"]); // proposing is fine

    let zhou = w.clone("zhou", "wutong");
    w.git_ok(
        &zhou,
        &[
            "fetch",
            "--quiet",
            "origin",
            "refs/proposals/wang/invite:refs/remotes/origin/invite",
        ],
    );
    w.git_ok(&zhou, &["merge", "--quiet", "--ff-only", "origin/invite"]);
    w.push_denied(&zhou, &["main"], "only the host may land it");

    // The host decides, and may.
    let host = w.clone("chenli", "wutong");
    w.git_ok(
        &host,
        &[
            "fetch",
            "--quiet",
            "origin",
            "refs/proposals/wang/invite:refs/remotes/origin/invite",
        ],
    );
    w.git_ok(&host, &["merge", "--quiet", "--ff-only", "origin/invite"]);
    w.push_ok(&host, &["main"]);
    assert!(w.relay_ok(&["members", "wutong"]).contains("newbie"));
}

// ───────────────────────────── guests and proposals ─────────────────────────────

#[test]
fn guests_write_only_their_own_proposals() {
    let w = World::new();
    let _ = w.wutong();
    let wang = w.clone("wang", "wutong");
    w.write(&wang, "works/0001/idea.md", "idea");
    w.commit(&wang, "idea\n\nProblem: 0001");

    w.push_denied(&wang, &["main"], "guests cannot write main");
    w.push_denied(&wang, &["HEAD:refs/proposals/li/idea"], "belongs to li");
    w.push_denied(
        &wang,
        &["HEAD:refs/heads/wang-branch"],
        "only refs/heads/main and refs/proposals",
    );
    w.push_ok(&wang, &["HEAD:refs/proposals/wang/idea"]);

    // Proposals still face problems.
    w.git_ok(&wang, &["commit", "--quiet", "--amend", "-m", "no trailer"]);
    w.push_denied(
        &wang,
        &["-f", "HEAD:refs/proposals/wang/idea2"],
        "no `Problem:` trailer",
    );

    // A guest may rework (force) and withdraw their own proposal.
    w.git_ok(
        &wang,
        &[
            "commit",
            "--quiet",
            "--amend",
            "-m",
            "idea v2\n\nProblem: 0001",
        ],
    );
    w.push_ok(&wang, &["-f", "HEAD:refs/proposals/wang/idea"]);
    w.push_ok(&wang, &[":refs/proposals/wang/idea"]);

    // Others cannot delete someone's proposal — except the host.
    w.push_ok(&wang, &["HEAD:refs/proposals/wang/again"]);
    let li = w.clone("li", "wutong");
    w.push_denied(&li, &[":refs/proposals/wang/again"], "belongs to wang");
    let zhou = w.clone("zhou", "wutong");
    w.push_denied(&zhou, &[":refs/proposals/wang/again"], "belongs to wang");
    let host = w.clone("chenli", "wutong");
    w.push_ok(&host, &[":refs/proposals/wang/again"]);
}

// ───────────────────────────── history is append-only ─────────────────────────────

#[test]
fn nobody_rewrites_or_deletes_history_not_even_the_host() {
    let w = World::new();
    let host = w.wutong();
    let before = w.main_oid("wutong");
    w.git_ok(&host, &["reset", "--quiet", "--hard", "HEAD~1"]);
    w.write(&host, "README.md", "rewritten\n");
    w.commit(&host, "rewrite\n\nProblem: meta");
    w.push_denied(&host, &["-f", "main"], "");
    w.push_denied(&host, &[":main"], "main can never be deleted");
    w.push_denied(&host, &["HEAD:refs/tags/v1"], "only refs/heads/main");
    assert_eq!(w.main_oid("wutong"), before);
}

#[test]
fn a_push_is_all_or_nothing() {
    let w = World::new();
    let _ = w.wutong();
    let before = w.main_oid("wutong");
    let zhou = w.clone("zhou", "wutong");
    w.write(&zhou, "works/0001/good.md", "good");
    w.commit(&zhou, "good\n\nProblem: 0001");
    // main alone would be accepted, but it travels with a forbidden ref.
    w.push_denied(
        &zhou,
        &["main", "HEAD:refs/heads/sneaky"],
        "only refs/heads/main",
    );
    assert_eq!(w.main_oid("wutong"), before);
}

// ───────────────────────────── seats ─────────────────────────────

#[test]
fn the_seating_chart_must_stay_valid() {
    let w = World::new();
    let host = w.wutong();

    // Sixteen people.
    let mut many: Vec<(String, &str, u8)> = vec![("chenli".into(), "host", 1)];
    for i in 0..15u8 {
        many.push((format!("g{i}"), "guest", 10 + i));
    }
    let entries: Vec<(&str, &str, u8)> =
        many.iter().map(|(a, b, c)| (a.as_str(), *b, *c)).collect();
    w.write(&host, ".teahouse/members.toml", &w.members_toml(&entries));
    w.commit(&host, "sixteen\n\nProblem: meta");
    w.push_denied(&host, &["main"], "at most 15 seats");

    // Two hosts.
    w.git_ok(&host, &["reset", "--quiet", "--hard", "origin/main"]);
    w.write(
        &host,
        ".teahouse/members.toml",
        &w.members_toml(&[("chenli", "host", 1), ("zhou", "host", 2)]),
    );
    w.commit(&host, "two hosts\n\nProblem: meta");
    w.push_denied(&host, &["main"], "exactly one host");

    // Deleting the chart.
    w.git_ok(&host, &["reset", "--quiet", "--hard", "origin/main"]);
    fs::remove_file(host.join(".teahouse/members.toml")).unwrap();
    w.commit(&host, "gone\n\nProblem: meta");
    w.push_denied(&host, &["main"], "cannot exist without it");

    // Nothing else may live in .teahouse/, not even from the host.
    w.git_ok(&host, &["reset", "--quiet", "--hard", "origin/main"]);
    w.write(&host, ".teahouse/hooks.sh", "#!/bin/sh\n");
    w.commit(&host, "x\n\nProblem: meta");
    w.push_denied(&host, &["main"], ".teahouse/ holds only members.toml");

    // Look-alikes that would overwrite the chart on a Windows/macOS checkout.
    for evil in [".TeaHouse/members.toml", ".teahouse./members.toml"] {
        w.git_ok(&host, &["reset", "--quiet", "--hard", "origin/main"]);
        w.write(&host, evil, "version = 1\n");
        w.commit(&host, "x\n\nProblem: meta");
        w.push_denied(&host, &["main"], "imitates .teahouse/");
    }

    // Fifteen is fine.
    w.git_ok(&host, &["reset", "--quiet", "--hard", "origin/main"]);
    w.write(
        &host,
        ".teahouse/members.toml",
        &w.members_toml(&entries[..15]),
    );
    w.commit(&host, "fifteen\n\nProblem: meta");
    w.push_ok(&host, &["main"]);
}

#[test]
fn only_seated_keys_get_in_and_removal_takes_effect_immediately() {
    let w = World::new();
    let host = w.wutong();

    let o = w.try_clone("outsider", "wutong");
    assert!(!o.status.success());
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("no seat"),
        "{}",
        s(&o)
    );

    // Unknown teahouses look exactly like ones you have no seat in.
    let o = w.try_clone("chenli", "nonexistent");
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("no seat"),
        "{}",
        s(&o)
    );

    // A member of another teahouse is an outsider here.
    w.relay_ok(&[
        "open",
        "kant",
        "--name",
        "康德读书会",
        "--host-id",
        "su",
        "--host-name",
        "苏",
        "--host-key",
        &w.key_file("outsider"),
    ]);
    assert!(w.try_clone("outsider", "kant").status.success());
    assert!(!w.try_clone("outsider", "wutong").status.success());

    // The host removes li; li is out on the next request.
    let li = w.clone("li", "wutong");
    w.write(
        &host,
        ".teahouse/members.toml",
        &w.members_toml(&[
            ("chenli", "host", 1),
            ("zhou", "steward", 2),
            ("wang", "guest", 3),
        ]),
    );
    w.commit(&host, "李搬走了\n\nProblem: meta");
    w.push_ok(&host, &["main"]);
    let o = w.git(&li, &["fetch", "origin"]);
    assert!(
        !o.status.success(),
        "removed member must not fetch:\n{}",
        s(&o)
    );
}

// ───────────────────────────── the gate itself ─────────────────────────────

#[test]
fn the_shell_runs_nothing_but_git() {
    let w = World::new();
    let _ = w.wutong();
    let key = key_for(1);
    for cmd in [
        "bash",
        "git-upload-archive '/wutong'",
        "git-upload-pack '/../../etc'",
        "scp -t /tmp",
        "git-receive-pack 'wutong'; id",
    ] {
        let o = Command::new(BIN)
            .args(["--root", w.root.to_str().unwrap(), "shell", key.blob_b64()])
            .env("SSH_ORIGINAL_COMMAND", cmd)
            .output()
            .unwrap();
        assert!(!o.status.success(), "{cmd:?} must be refused:\n{}", s(&o));
    }
    // An interactive login just says where you sit.
    let o = Command::new(BIN)
        .args(["--root", w.root.to_str().unwrap(), "shell", key.blob_b64()])
        .env_remove("SSH_ORIGINAL_COMMAND")
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("wutong"),
        "{}",
        s(&o)
    );
    // Refused commands and logins are audited too.
    let log = w.audit();
    assert_eq!(
        log.matches("\"event\":\"shell.refused\"").count(),
        5,
        "{log}"
    );
    let info: Vec<&str> = log
        .lines()
        .filter(|l| l.contains("\"event\":\"shell.info\""))
        .collect();
    assert_eq!(info.len(), 1, "{log}");
    assert!(info[0].contains("\"houses\":\"wutong\""), "{}", info[0]);

    // A key with no seat anywhere gets an empty list, still audited.
    let o = Command::new(BIN)
        .args([
            "--root",
            w.root.to_str().unwrap(),
            "shell",
            key_for(99).blob_b64(),
        ])
        .env("SSH_ORIGINAL_COMMAND", "info")
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("no seat"),
        "{}",
        s(&o)
    );
    let log = w.audit();
    let info: Vec<&str> = log
        .lines()
        .filter(|l| l.contains("\"event\":\"shell.info\""))
        .collect();
    assert_eq!(info.len(), 2, "{log}");
    assert!(info[1].contains("\"houses\":\"\""), "{}", info[1]);
}

#[test]
fn pushes_that_bypass_the_relay_shell_are_refused() {
    // Fail closed: if anything ever reaches receive-pack without the shell
    // (a misconfigured sshd, a local file:// push), the hook refuses it.
    let w = World::new();
    let host = w.wutong();
    let bare = w.root.join("houses/wutong.git");
    w.git_ok(&host, &["remote", "add", "direct", bare.to_str().unwrap()]);
    w.write(&host, "README.md", "direct\n");
    w.commit(&host, "x\n\nProblem: meta");
    let o = w.git(&host, &["push", "direct", "main"]);
    assert!(!o.status.success());
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("must arrive through the relay"),
        "{}",
        s(&o)
    );
}

#[test]
fn every_decision_is_audited() {
    let w = World::new();
    let _ = w.wutong();
    let wang = w.clone("wang", "wutong");
    w.write(&wang, "works/0001/x.md", "x");
    w.commit(&wang, "x\n\nProblem: 0001");
    let _ = w.push(&wang, &["main"]);
    let log = w.audit();
    let denied: Vec<&str> = log
        .lines()
        .filter(|l| l.contains("\"event\":\"push.denied\""))
        .collect();
    assert_eq!(denied.len(), 1, "{log}");
    assert!(denied[0].contains("\"member\":\"wang\""));
    assert!(denied[0].contains("guests cannot write main"));
    for line in log.lines() {
        assert!(
            line.starts_with("{\"ts\":\"") && line.ends_with('}'),
            "malformed: {line}"
        );
    }
}

// ───────────────────────────── OpenSSH integration ─────────────────────────────

#[test]
fn authorized_keys_are_locked_down() {
    let w = World::new();
    let _ = w.wutong();
    let out = w.relay_ok(&["authorized-keys"]);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 4, "{out}");
    for l in &lines {
        assert!(l.starts_with("restrict,command=\""), "{l}");
        assert!(l.contains(" shell "), "{l}");
    }
    let one = w.relay_ok(&["authorized-keys", "--key", key_for(3).blob_b64()]);
    assert_eq!(one.lines().count(), 1);
    assert!(w
        .relay_ok(&["authorized-keys", "--key", key_for(99).blob_b64()])
        .is_empty());

    let ak = w.tmp.0.join("authorized_keys");
    fs::write(&ak, "ssh-ed25519 AAAA admin-key-kept\n").unwrap();
    w.relay_ok(&["sync-keys", "--file", ak.to_str().unwrap()]);
    w.relay_ok(&["sync-keys", "--file", ak.to_str().unwrap()]); // idempotent
    let text = fs::read_to_string(&ak).unwrap();
    assert!(text.starts_with("ssh-ed25519 AAAA admin-key-kept\n"));
    assert_eq!(text.matches("# BEGIN TEAHOUSE").count(), 1);
    assert_eq!(text.matches("restrict,command=").count(), 4);
}

#[test]
fn post_receive_refreshes_authorized_keys_when_seats_change() {
    let w = World::new();
    let ak = w.tmp.0.join("ak");
    w.relay_ok(&[
        "init",
        "--binary",
        BIN,
        "--authorized-keys",
        ak.to_str().unwrap(),
    ]);
    let host = w.wutong();
    assert_eq!(
        fs::read_to_string(&ak)
            .unwrap()
            .matches("restrict,")
            .count(),
        4
    );
    w.write(
        &host,
        ".teahouse/members.toml",
        &w.members_toml(&[("chenli", "host", 1)]),
    );
    w.commit(&host, "只剩老板\n\nProblem: meta");
    w.push_ok(&host, &["main"]);
    assert_eq!(
        fs::read_to_string(&ak)
            .unwrap()
            .matches("restrict,")
            .count(),
        1
    );
}

// ───────────────────────────── 继承 · succession ─────────────────────────────

#[test]
fn the_steward_can_reopen_the_teahouse_elsewhere_and_history_survives() {
    let w = World::new();
    let _ = w.wutong();
    let zhou = w.clone("zhou", "wutong");
    w.write(&zhou, "works/0001/plan.md", "plan");
    w.commit(&zhou, "plan\n\nProblem: 0001");
    w.push_ok(&zhou, &["main"]);
    let last = w.main_oid("wutong");

    // The old relay disappears. The steward stands up a new one.
    let w2 = World {
        tmp: Tmp::new("relay2"),
        root: PathBuf::new(),
        keys: w.keys.clone(),
    };
    let w2 = World {
        root: w2.tmp.0.join("relay"),
        ..w2
    };
    w2.relay_ok(&["init", "--binary", BIN]);

    // Only the steward's key works.
    let o = w2.relay(&[
        "adopt",
        "wutong",
        "--from",
        zhou.to_str().unwrap(),
        "--key",
        &w.key_file("wang"),
    ]);
    assert!(
        !o.status.success() && String::from_utf8_lossy(&o.stderr).contains("only the steward"),
        "{}",
        s(&o)
    );
    let o = w2.relay(&[
        "adopt",
        "wutong",
        "--from",
        zhou.to_str().unwrap(),
        "--key",
        &w.key_file("outsider"),
    ]);
    assert!(!o.status.success(), "{}", s(&o));
    assert!(
        !w2.root.join("houses/wutong.git").exists(),
        "failed adoption leaves nothing behind"
    );

    w2.relay_ok(&[
        "adopt",
        "wutong",
        "--from",
        zhou.to_str().unwrap(),
        "--key",
        &w.key_file("zhou"),
    ]);
    let members = w2.relay_ok(&["members", "wutong"]);
    assert!(members.contains("host     zhou"), "{members}");
    assert!(members.contains("guest    chenli"), "{members}");

    // History is the same history: the succession commit sits on top of it.
    let new_main = w2.main_oid("wutong");
    let parent = Command::new("git")
        .arg("--git-dir")
        .arg(w2.root.join("houses/wutong.git"))
        .args(["rev-parse", &format!("{new_main}^")])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&parent.stdout).trim(), last);

    // The new host works; the old host is now a guest there.
    let zhou2 = w2.clone("zhou", "wutong");
    w2.write(&zhou2, "README.md", "茶馆照常营业\n");
    w2.commit(&zhou2, "照常营业\n\nProblem: meta");
    w2.push_ok(&zhou2, &["main"]);
    let chen2 = w2.clone("chenli", "wutong");
    w2.write(&chen2, "works/0001/x.md", "x");
    w2.commit(&chen2, "x\n\nProblem: 0001");
    w2.push_denied(&chen2, &["main"], "guests cannot write main");
    w2.relay_ok(&["check"]);
}

#[test]
fn opening_validates_its_inputs() {
    let w = World::new();
    w.person("a", 1);
    let k = w.key_file("a");
    for (args, needle) in [
        (
            vec![
                "open",
                "Bad",
                "--name",
                "x",
                "--host-id",
                "a",
                "--host-name",
                "A",
                "--host-key",
                &k,
            ],
            "teahouse id",
        ),
        (
            vec![
                "open",
                "ok",
                "--name",
                "x",
                "--host-id",
                "A!",
                "--host-name",
                "A",
                "--host-key",
                &k,
            ],
            "host id",
        ),
        (
            vec![
                "open",
                "ok",
                "--name",
                "x",
                "--host-id",
                "a",
                "--host-name",
                "A",
                "--host-key",
                "ssh-rsa AAAA",
            ],
            "ssh-ed25519",
        ),
        (
            vec![
                "open",
                "ok",
                "--name",
                "",
                "--host-id",
                "a",
                "--host-name",
                "A",
                "--host-key",
                &k,
            ],
            "must not be empty",
        ),
    ] {
        let o = w.relay(&args);
        assert!(
            !o.status.success() && String::from_utf8_lossy(&o.stderr).contains(needle),
            "{args:?}\n{}",
            s(&o)
        );
    }
    w.relay_ok(&[
        "open",
        "ok",
        "--name",
        "x",
        "--host-id",
        "a",
        "--host-name",
        "A",
        "--host-key",
        &k,
    ]);
    let o = w.relay(&[
        "open",
        "ok",
        "--name",
        "x",
        "--host-id",
        "a",
        "--host-name",
        "A",
        "--host-key",
        &k,
    ]);
    assert!(String::from_utf8_lossy(&o.stderr).contains("already exists"));
}

#[test]
fn concurrent_pushes_never_corrupt_main() {
    let w = World::new();
    let _ = w.wutong();
    let a = w.clone("chenli", "wutong");
    let b = w.clone("zhou", "wutong");
    for i in 0..8 {
        w.write(&a, &format!("works/0001/a{i}.md"), "a");
        w.commit(&a, &format!("a{i}\n\nProblem: 0001"));
        w.write(&b, &format!("works/0001/b{i}.md"), "b");
        w.commit(&b, &format!("b{i}\n\nProblem: 0001"));
    }
    let (ra, rb) = std::thread::scope(|sc| {
        let ta = sc.spawn(|| w.push(&a, &["main"]).status.success());
        let tb = sc.spawn(|| w.push(&b, &["main"]).status.success());
        (ta.join().unwrap(), tb.join().unwrap())
    });
    assert!(
        ra ^ rb,
        "exactly one of two diverging pushes wins (a={ra}, b={rb})"
    );
    let winner = if ra { &a } else { &b };
    assert_eq!(
        w.main_oid("wutong"),
        w.git_ok(winner, &["rev-parse", "HEAD"])
    );
    w.relay_ok(&["check"]);
}

// ───────────────────────────── 讨论与公告 · talk and board ─────────────────────────────

#[test]
fn guests_post_to_talk_directly_but_only_their_own_new_posts() {
    let w = World::new();
    let _ = w.wutong();
    let wang = w.clone("wang", "wutong");
    let post = "talk/0001/20261018T213000Z-wang.md";

    // A guest adds their own post straight to main.
    w.write(&wang, post, "晚上七点以后基本没位置。\n");
    w.commit(&wang, "留言\n\nProblem: 0001");
    w.push_ok(&wang, &["main"]);

    // …but cannot edit or delete it afterwards (written, not spoken).
    w.write(&wang, post, "改口了\n");
    w.commit(&wang, "改留言\n\nProblem: 0001");
    w.push_denied(
        &wang,
        &["main"],
        "guests cannot write main except adding their own posts",
    );
    w.git_ok(&wang, &["reset", "--quiet", "--hard", "origin/main"]);
    fs::remove_file(wang.join(post)).unwrap();
    w.commit(&wang, "删留言\n\nProblem: 0001");
    w.push_denied(
        &wang,
        &["main"],
        "guests cannot write main except adding their own posts",
    );

    // Posting in someone else's name, or sneaking a result in with a post.
    w.git_ok(&wang, &["reset", "--quiet", "--hard", "origin/main"]);
    w.write(&wang, "talk/0001/20261018T213100Z-li.md", "冒名\n");
    w.commit(&wang, "x\n\nProblem: 0001");
    w.push_denied(&wang, &["main"], "not talk/0001/20261018T213100Z-li.md");
    w.git_ok(&wang, &["reset", "--quiet", "--hard", "origin/main"]);
    w.write(&wang, "talk/0001/20261018T213200Z-wang.md", "ok\n");
    w.write(&wang, "works/0001/plan.md", "smuggled\n");
    w.commit(&wang, "x\n\nProblem: 0001");
    w.push_denied(&wang, &["main"], "not works/0001/plan.md");

    // A post must sit under the problem its commit faces.
    w.git_ok(&wang, &["reset", "--quiet", "--hard", "origin/main"]);
    w.write(&wang, "talk/0001/20261018T213300Z-wang.md", "ok\n");
    w.commit(&wang, "x\n\nProblem: board");
    w.push_denied(&wang, &["main"], "different topic");

    // The steward moderates.
    let zhou = w.clone("zhou", "wutong");
    fs::remove_file(zhou.join(post)).unwrap();
    w.commit(&zhou, "移除留言\n\nProblem: 0001");
    w.push_ok(&zhou, &["main"]);
}

#[test]
fn board_notices_and_replies() {
    let w = World::new();
    let _ = w.wutong();
    let zhou = w.clone("zhou", "wutong");
    w.write(
        &zhou,
        "board/2026-10-25-meeting.md",
        "周六上午十点，楼下集合。\n",
    );
    w.commit(&zhou, "贴公告\n\nProblem: board");
    w.push_ok(&zhou, &["main"]);

    let wang = w.clone("wang", "wutong");
    w.write(
        &wang,
        "talk/board/2026-10-25-meeting/20261018T080000Z-wang.md",
        "我去。\n",
    );
    w.commit(&wang, "回应公告\n\nProblem: board");
    w.push_ok(&wang, &["main"]);

    // `board` is for the board only.
    w.git_ok(&zhou, &["pull", "--quiet", "--ff-only"]);
    w.write(&zhou, "works/0001/x.md", "x");
    w.commit(&zhou, "x\n\nProblem: board");
    w.push_denied(&zhou, &["main"], "may only change board/");

    // Guests do not post notices themselves; they propose them.
    w.write(&wang, "board/my-notice.md", "x");
    w.commit(&wang, "x\n\nProblem: board");
    w.push_denied(&wang, &["main"], "guests cannot write main except");
    w.push_ok(&wang, &["HEAD:refs/proposals/wang/notice"]);
}

/// A post's stamp, `offset` seconds from now.
fn stamp_in(offset: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    teahouse_core::rules::format_stamp(now + offset)
}

#[test]
fn replies_need_a_real_notice_or_problem() {
    let w = World::new();
    let host = w.wutong();
    let wang = w.clone("wang", "wutong");

    // No such notice: refused, for a guest and for the host alike.
    w.write(
        &wang,
        "talk/board/no-such-notice/20261018T080000Z-wang.md",
        "?\n",
    );
    w.commit(&wang, "回应\n\nProblem: board");
    w.push_denied(
        &wang,
        &["main"],
        "which does not exist (no board/no-such-notice.md)",
    );
    w.git_ok(&wang, &["reset", "--quiet", "--hard", "origin/main"]);

    // A post under a problem that does not exist, even in a host's meta commit.
    w.write(&host, "talk/0002/20261018T080000Z-chenli.md", "?\n");
    w.commit(&host, "x\n\nProblem: meta");
    w.push_denied(
        &host,
        &["main"],
        "answers problem 0002, which does not exist",
    );
    w.git_ok(&host, &["reset", "--quiet", "--hard", "origin/main"]);

    // Once the notice is up, the same reply lands.
    let zhou = w.clone("zhou", "wutong");
    w.write(&zhou, "board/meeting.md", "周六开会。\n");
    w.commit(&zhou, "贴公告\n\nProblem: board");
    w.push_ok(&zhou, &["main"]);
    w.git_ok(&wang, &["pull", "--quiet", "--ff-only"]);
    w.write(
        &wang,
        "talk/board/meeting/20261018T080000Z-wang.md",
        "我去。\n",
    );
    w.commit(&wang, "回应\n\nProblem: board");
    w.push_ok(&wang, &["main"]);

    // Taking a notice down later is moderation; its replies may stay.
    w.git_ok(&zhou, &["pull", "--quiet", "--ff-only"]);
    fs::remove_file(zhou.join("board/meeting.md")).unwrap();
    w.commit(&zhou, "撤公告\n\nProblem: board");
    w.push_ok(&zhou, &["main"]);
}

#[test]
fn post_stamps_may_not_run_ahead_of_the_relay() {
    let w = World::new();
    let _ = w.wutong();
    let wang = w.clone("wang", "wutong");

    // Up to ten minutes ahead is clock skew, and fine.
    let soon = format!("talk/0001/{}-wang.md", stamp_in(5 * 60));
    w.write(&wang, &soon, "快到了\n");
    w.commit(&wang, "x\n\nProblem: 0001");
    w.push_ok(&wang, &["main"]);

    // Further ahead is refused, whatever the year…
    for stamp in [stamp_in(20 * 60), "20990101T000000Z".to_string()] {
        let post = format!("talk/0001/{stamp}-wang.md");
        w.write(&wang, &post, "未来\n");
        w.commit(&wang, "x\n\nProblem: 0001");
        w.push_denied(&wang, &["main"], "minutes in the future");
        w.git_ok(&wang, &["reset", "--quiet", "--hard", "origin/main"]);
    }
    // …also on a proposal.
    w.write(&wang, "talk/0001/20990101T000000Z-wang.md", "未来\n");
    w.commit(&wang, "x\n\nProblem: 0001");
    w.push_denied(
        &wang,
        &["HEAD:refs/proposals/wang/x"],
        "minutes in the future",
    );
    w.git_ok(&wang, &["reset", "--quiet", "--hard", "origin/main"]);

    // Earlier stamps are fine (written offline, pushed later). Order on
    // screen comes from main's history, so an old stamp gains nothing.
    w.write(
        &wang,
        "talk/0001/19990101T000000Z-wang.md",
        "很久以前写的\n",
    );
    w.commit(&wang, "x\n\nProblem: 0001");
    w.push_ok(&wang, &["main"]);

    // Not a real date: not a post at all.
    w.write(&wang, "talk/0001/20260230T000000Z-wang.md", "二月三十日\n");
    w.commit(&wang, "x\n\nProblem: 0001");
    w.push_denied(&wang, &["main"], "except adding their own posts");
}

#[test]
fn posts_and_notices_are_text_of_at_most_32_kib() {
    let w = World::new();
    let _ = w.wutong();
    let wang = w.clone("wang", "wutong");
    let write_bytes = |dir: &Path, path: &str, bytes: &[u8]| {
        let p = dir.join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, bytes).unwrap();
    };

    // Exactly 32 KiB of Chinese text (10 922 three-byte characters + 2 bytes).
    let long = format!("{}\n\n", "茶".repeat(10_922));
    assert_eq!(long.len(), 32 * 1024);
    w.write(&wang, "talk/0001/20261018T080000Z-wang.md", &long);
    w.commit(&wang, "长留言\n\nProblem: 0001");
    w.push_ok(&wang, &["main"]);

    for (bytes, why) in [
        (format!("{long}x").into_bytes(), "limited to 32 KiB"),
        (b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec(), "not UTF-8 text"),
        (b"\x1b[2J\x1b[31mhello".to_vec(), "U+001B"),
    ] {
        write_bytes(&wang, "talk/0001/20261018T080100Z-wang.md", &bytes);
        w.commit(&wang, "x\n\nProblem: 0001");
        w.push_denied(&wang, &["main"], why);
        w.push_denied(&wang, &["HEAD:refs/proposals/wang/x"], why);
        w.git_ok(&wang, &["reset", "--quiet", "--hard", "origin/main"]);
    }

    // A symlink is not a post, even if its name looks like one.
    std::os::unix::fs::symlink(
        "../../.teahouse/members.toml",
        wang.join("talk/0001/20261018T080200Z-wang.md"),
    )
    .unwrap();
    w.commit(&wang, "x\n\nProblem: 0001");
    w.push_denied(&wang, &["main"], "not a symlink");
    w.git_ok(&wang, &["reset", "--quiet", "--hard", "origin/main"]);

    // Notices too: text only, attachments go to works/.
    let zhou = w.clone("zhou", "wutong");
    write_bytes(&zhou, "board/photo.png", b"\x89PNG\r\n\x1a\n\0\0");
    w.commit(&zhou, "x\n\nProblem: board");
    w.push_denied(&zhou, &["main"], "board/photo.png is not UTF-8 text");
    w.git_ok(&zhou, &["reset", "--quiet", "--hard", "origin/main"]);
    w.write(&zhou, "board/long.md", &format!("{long}x"));
    w.commit(&zhou, "x\n\nProblem: board");
    w.push_denied(&zhou, &["main"], "board/long.md is 32769 bytes");
    w.git_ok(&zhou, &["reset", "--quiet", "--hard", "origin/main"]);
    write_bytes(&zhou, "works/0001/photo.png", b"\x89PNG\r\n\x1a\n\0\0");
    w.commit(&zhou, "照片\n\nProblem: 0001");
    w.push_ok(&zhou, &["main"]);
}
