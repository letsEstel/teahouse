//| # 第一章 座位表
//|
//| 一间茶馆最要紧的东西不是内容，是一张表：谁坐在这里，坐的是哪种座位。这张表是
//| 茶馆仓库里的一个文本文件，`.teahouse/members.toml`。Relay 判断一次写入能不能
//| 进来，第一步就是拿写入者的钥匙到这张表上找人。
//|
//| 这一章讲的就是读写这张表的全部代码。你现在读的不是代码的讲解稿：这一页就是
//| `core/src/members.rs` 这个文件本身，所有代码一行不少，按文件里的顺序排列，
//| 编译器编译的也正是这些字。
//|
//| 读完你会认识 Rust 最常用的几样东西：`enum`、`struct`、`match`、`Option`、
//| `Result`、`?`，以及测试。你不需要事先会 Rust，也不需要事先会编程。
//|
//| 文件开头是一段英文注释，列出这张表必须遵守的七条规矩。`//!` 开头的行是写给
//| 这个文件的说明，编译器不执行它们。后面的代码就是在落实这七条。
//! `.teahouse/members.toml`: who sits in a teahouse and in which role.
//!
//! Invariants (说明书 第四节):
//! 1. 1 ≤ members ≤ 15
//! 2. exactly one host (老板)
//! 3. at most one steward (伙计)
//! 4. everyone else is a guest (茶客)
//! 5. member ids and keys are unique
//! 6. `kind = "machine"` marks a machine (an AI, a CI job, a script); it is a
//!    label, not a rank (any role), and `agent` appears only on machines
//!    (法则第 17 条, docs/MCP.md). The older spelling `kind = "ai"` means
//!    the same and is kept as written.
//! 7. a member has 1–5 devices: `key`, plus `[[device]]` tables naming them
//!    (`version = 2`); every key in the chart is unique (法则第 18 条,
//!    docs/DEVICES.md)

//| `use` 把别的文件里的名字拿到这里来用。`PublicKey` 是一把公钥，下一章再细看；
//| 这一章只要知道每把钥匙对应一台设备，后面有一章专门讲它。`crate` 指当前这个代码包 `teahouse-core`。
use crate::key::PublicKey;
use crate::toml::{self, Value};
use std::fmt;

//| `const` 定义一个永远不变的值，`pub` 表示别的文件也能用它。`usize` 是“用来数数的
//| 整数“，`i64` 是 64 位整数，`&str` 是一段文字。下面的 `MAX_MEMBERS` 是全书最重要的
//| 数字之一：Relay 的法则里，十五个座位的上限由它决定。
//|
//| > 想动手的话，先看本章最后的“试一试”：把 15 改成 16，看看会发生什么。
pub const MEMBERS_PATH: &str = ".teahouse/members.toml";
pub const MAX_MEMBERS: usize = 15;
/// The format of a chart without `[[device]]`; it is written whenever
/// nobody has more than one device, so such charts never change.
pub const FORMAT_VERSION: i64 = 1;
/// The format with `[[device]]` tables (docs/DEVICES.md).
pub const FORMAT_VERSION_DEVICES: i64 = 2;
/// Devices (keys) per member, the first included (法则第 18 条).
pub const MAX_DEVICES: usize = 5;

//| ## 三种座位
//|
//| `enum` 定义一种只能取固定几个值的类型。`Role` 只有三个值，所以一个座位不可能是
//| “副老板”或者“管理员”：这样的值在程序里写不出来。
//|
//| `#[derive(...)]` 让编译器自动写几样常用能力：`Debug` 能打印出来，`Clone` 和 `Copy`
//| 能复制，`PartialEq`、`Eq` 能比较是否相等。`Hash`、`PartialOrd`、`Ord` 是排序和
//| 查找用的。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Role {
    /// 老板 — full write access, merges proposals, manages seats.
    Host,
    /// 伙计 — writes problems/works/board directly; designated heir.
    Steward,
    /// 茶客 — reads everything, writes only proposals.
    Guest,
}

//| `impl` 给一个类型添加方法。下面第一个方法把文件里的文字变成 `Role`。
//|
//| 文字可能写错，所以它不直接返回 `Role`，而是返回 `Option<Role>`。`Option` 是 Rust
//| 表达“可能没有”的方式：有就是 `Some(值)`，没有就是 `None`。Rust 没有其他语言里的
//| `null`，调用的人必须处理 `None` 这种情况，否则编译不过。
//|
//| `match` 依次比较，走第一个对得上的分支；`_` 表示“其余一切”。下面这段是真的会运行
//| 的例子，`cargo test` 每次都执行它：
impl Role {
    /// ```
    /// use teahouse_core::Role;
    ///
    /// assert_eq!(Role::parse("steward"), Some(Role::Steward));
    /// assert_eq!(Role::parse("admin"), None); // 茶馆里没有这种座位
    /// ```
    pub fn parse(s: &str) -> Option<Role> {
        match s {
            "host" => Some(Role::Host),
            "steward" => Some(Role::Steward),
            "guest" => Some(Role::Guest),
            _ => None,
        }
    }

    //| 反过来，把 `Role` 变回文字。这里的 `match` 没有 `_`，三个值全列出来了，这是有意的。
    //| Rust 要求 `match` 覆盖所有可能：以后要是加第四种座位，所有这样的 `match` 都会编译
    //| 失败，每一处都得有人想一想新座位该怎么办。
    //|
    //| 返回类型 `&'static str` 意思是“一段在程序运行期间一直存在的文字”。写在代码里的
    //| 字符串都是这样。
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Host => "host",
            Role::Steward => "steward",
            Role::Guest => "guest",
        }
    }

    pub fn zh(self) -> &'static str {
        match self {
            Role::Host => "老板",
            Role::Steward => "伙计",
            Role::Guest => "茶客",
        }
    }
}

//| ## 人和机器
//|
//| 座位表只分人和机器。`#[default]` 说明没写的时候算 `Human`。
//|
//| `Ai` 这个值看起来多余：它和 `Machine` 意思一样。它留着，是因为早期的座位表里写的
//| 就是 `ai`，而且有些旧客户端只认这个词。Relay 改写一张旧表时要原样写回去，所以这里
//| 记住“当初是怎么写的”，判断时再用 `is_machine` 把两者看成一回事。
/// Whether a seat is held by a person or a machine (docs/MCP.md §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Kind {
    #[default]
    Human,
    Machine,
    /// A machine, spelled `ai` as charts and invitations made before
    /// machines were named so. Kept as written: a chart the relay
    /// rewrites (pairing, invitations) must stay readable by clients that
    /// know only `ai`.
    Ai,
}

impl Kind {
    pub fn parse(s: &str) -> Option<Kind> {
        match s {
            "human" => Some(Kind::Human),
            "machine" => Some(Kind::Machine),
            "ai" => Some(Kind::Ai),
            _ => None,
        }
    }

    /// As written in the chart.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Human => "human",
            Kind::Machine => "machine",
            Kind::Ai => "ai",
        }
    }

    pub fn is_machine(self) -> bool {
        self != Kind::Human
    }

    /// What people are shown: `human` or `machine`.
    pub fn label(self) -> &'static str {
        if self.is_machine() {
            "machine"
        } else {
            "human"
        }
    }
}

//| `impl fmt::Display for Role` 的写法和上面的 `impl Role` 不同：它是在实现一份约定。
//| `Display` 这份约定规定“这个类型怎么显示给人看”，实现之后 `format!("{role}")` 就能用
//| 了。Rust 把这种约定叫 trait，后面还会反复遇到。
impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

//| ## 一个座位上有什么
//|
//| `struct` 把几样东西捆成一个整体。`Member` 就是座位表上的一行。
//|
//| 这里出现了三种新写法：
//|
//| - `String` 是自己拥有的一段文字。和前面的 `&str` 相比，`&str` 只是“借来看一眼”。
//| - `Option<String>` 是可能没有的文字。人没有 `agent`，只有机器才有。
//| - `Vec<Device>` 是长度可变的一串 `Device`。第一台设备的钥匙在 `key` 里，其余的
//|   放在这里，所以这串可以是空的。
//|
//| “拥有”和“借用”是 Rust 最有特点的地方：同一时刻谁对一块内存负责，编译器全程
//| 检查。这也是 Rust 不需要垃圾回收、又不容易写出内存错误的原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub role: Role,
    pub key: PublicKey,
    /// `human` unless the chart says `kind = "machine"` (or `"ai"`).
    pub kind: Kind,
    /// What machine it is (`agent = "Claude Code"`, `"GitHub Actions"`),
    /// for display only.
    pub agent: Option<String>,
    /// Further devices, beyond the one in `key` (`[[device]]`).
    pub devices: Vec<Device>,
}

/// One more device of a member: its key, and a name people recognise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub name: Option<String>,
    pub key: PublicKey,
}

//| `human` 是创建一个人的座位的便捷写法。参数类型 `impl Into<String>` 的意思是“任何能
//| 变成 `String` 的东西“，所以 `"陈立"` 这样的文字可以直接传进来。
//|
//| `keys` 把一个人的所有钥匙连成一串：`once` 产生只含一个元素的序列，`chain` 接上另一个
//| 序列，`map` 对每个元素做变换。这种串起来的写法叫迭代器。`|d| &d.key` 是一个没有名字的
//| 小函数，叫闭包，意思是“给我一个 d，我还你它的 key”。
impl Member {
    /// A person's seat (`kind` human, no `agent`).
    pub fn human(
        id: impl Into<String>,
        name: impl Into<String>,
        role: Role,
        key: PublicKey,
    ) -> Member {
        Member {
            id: id.into(),
            name: name.into(),
            role,
            key,
            kind: Kind::Human,
            agent: None,
            devices: Vec::new(),
        }
    }

    /// Every key of this member: `key` first, then the devices.
    pub fn keys(&self) -> impl Iterator<Item = &PublicKey> {
        std::iter::once(&self.key).chain(self.devices.iter().map(|d| &d.key))
    }

    pub fn has_key(&self, key: &PublicKey) -> bool {
        self.keys().any(|k| k == key)
    }

    /// The name of the device with this key (`None` for an unnamed one).
    pub fn device_name(&self, key: &PublicKey) -> Option<&str> {
        self.devices
            .iter()
            .find(|d| &d.key == key)
            .and_then(|d| d.name.as_deref())
    }

    //| 上面几个方法的参数都是 `&self`，只读。这个方法是 `&mut self`，它要改动座位。
    //|
    //| 返回值 `Result<(), MembersError>` 是 Rust 表达“可能失败”的方式：成功是 `Ok(值)`，
    //| 失败是 `Err(错误)`。`()` 表示成功时没什么要返回的。`Option` 说“可能没有”，`Result`
    //| 说“可能出错，并且说明原因”。这是一个人最后一台设备不能移除的地方：调用的人会收到
    //| 一个 `Err`，程序不会崩溃，也没法假装没发生过，这就是 `Result` 的用处。
    /// Removes the device with this key; the next device takes the place of
    /// the first one if that is the one removed. Refuses the last device.
    pub fn remove_key(&mut self, key: &PublicKey) -> Result<(), MembersError> {
        if !self.has_key(key) {
            return e(format!(
                "{} is not a device of {:?}",
                key.fingerprint(),
                self.id
            ));
        }
        if self.devices.is_empty() {
            return e(format!(
                "this is the last device of {:?}; a member keeps at least one / 每个成员至少保留一台设备",
                self.id
            ));
        }
        if &self.key == key {
            self.key = self.devices.remove(0).key;
        } else {
            self.devices.retain(|d| &d.key != key);
        }
        Ok(())
    }
}

//| ## 整张表
//|
//| `Members` 是整张座位表：茶馆的名字、招牌，和所有成员。
//|
//| `MembersError(pub String)` 是只包着一段文字的类型。为什么不直接用 `String` 当错误？
//| 因为类型是有含义的：看到函数返回 `MembersError`，就知道这是座位表的问题。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Members {
    pub name: String,
    pub signboard: Option<String>,
    pub members: Vec<Member>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembersError(pub String);

impl fmt::Display for MembersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "members.toml: {}", self.0)
    }
}

impl std::error::Error for MembersError {}

//| 这个只有一行的函数名字很短，因为它到处都要用：`return e("...")` 比写全一个 `Err`
//| 省事得多。`<T>` 是泛型，意思是它能假装返回任何类型的成功值，反正它永远只返回失败。
fn e<T>(msg: impl Into<String>) -> Result<T, MembersError> {
    Err(MembersError(msg.into()))
}

//| ## 把文件读进来
//|
//| 成员 id 会出现在 `refs/proposals/<id>/…` 这样的路径里，所以规定得很死，只允许小写
//| 字母、数字、`-`、`_`。`&&` 表示“并且”，三个条件全成立才算合格。
/// Member ids: lowercase ASCII, digits, `-` and `_`; 1–32 chars; starts alnum.
/// They appear in ref names (`refs/proposals/<id>/…`) so they must be boring.
///
/// ```
/// use teahouse_core::members::valid_member_id;
///
/// assert!(valid_member_id("ai-1"));
/// assert!(!valid_member_id("Wang"));          // 不能有大写
/// assert!(!valid_member_id("a/b"));           // 不能有斜杠，否则能冒充别的提案路径
/// assert!(!valid_member_id("王阿姨"));         // 中文名写在 name 里，id 只用 ASCII
/// ```
pub fn valid_member_id(id: &str) -> bool {
    let b = id.as_bytes();
    (1..=32).contains(&b.len())
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter()
            .all(|&c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_')
}

fn take_str(t: &toml::Table, key: &str, ctx: &str) -> Result<String, MembersError> {
    match t.get(key) {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(_) => e(format!("{ctx}.{key} must be a string")),
        None => e(format!("{ctx}.{key} is required")),
    }
}

fn check_keys(t: &toml::Table, allowed: &[&str], ctx: &str) -> Result<(), MembersError> {
    for k in t.keys() {
        if !allowed.contains(&k.as_str()) {
            return e(format!("unknown field {ctx}.{k}"));
        }
    }
    Ok(())
}

//| 注意长度用的是 `s.chars().count()`，数的是字符。`s.len()` 数的是字节，一个汉字占
//| 三个字节，用它的话中文名字的上限就只有英文的三分之一。
fn check_text(s: &str, field: &str, max_chars: usize) -> Result<(), MembersError> {
    if s.trim().is_empty() {
        return e(format!("{field} must not be empty"));
    }
    if s.chars().count() > max_chars {
        return e(format!("{field} is longer than {max_chars} characters"));
    }
    if s.chars().any(|c| c.is_control() && c != '\n') {
        return e(format!("{field} contains control characters"));
    }
    Ok(())
}

//| `parse` 是这一章最长的函数，做的事情只有一件：把文本变成 `Members`，或者说明为什么
//| 不行。里面到处是一个小小的问号：
//|
//| ```text
//| let doc = toml::parse(text).map_err(...)?;
//| ```
//|
//| `?` 的意思是：如果前面的结果是 `Err`，立刻把这个错误返回给调用者；如果是 `Ok`，取出
//| 里面的值继续走。没有它的话，每一步都要写一个 `match`。
//|
//| 这个函数非常挑剔。不认识的字段、不认识的表，一律拒绝，而不是忽略。理由是：座位表
//| 决定谁能进门，这里读不懂的东西，宁可拒绝，也不要猜它的意思。
impl Members {
    /// ```
    /// use teahouse_core::{Members, PublicKey};
    ///
    /// let key = PublicKey::from_point(&[1; 32]).openssh();
    /// let chart = format!(
    ///     "version = 1\n\n[teahouse]\nname = \"梧桐里\"\n\n\
    ///      [[member]]\nid = \"chen\"\nname = \"陈立\"\nrole = \"host\"\nkey = \"{key}\"\n"
    /// );
    /// assert_eq!(Members::parse(&chart).unwrap().host().name, "陈立");
    ///
    /// // 不认识的座位类型、多写的字段，都不是"忽略"，是拒绝
    /// assert!(Members::parse(&chart.replace("host", "admin")).is_err());
    /// assert!(Members::parse(&format!("{chart}\nadmin = true\n")).is_err());
    /// ```
    pub fn parse(text: &str) -> Result<Members, MembersError> {
        //| 第一步：解析成通用的 TOML 结构，再确认最外层只有 `version`。`Some(Value::Int(v @ (A | B)))`
        //| 这种写法是 `match` 的本事：不仅比较，还能拆开数据，并把里面的值绑定到 `v` 上。
        let doc = toml::parse(text).map_err(|err| MembersError(err.to_string()))?;
        check_keys(&doc.root, &["version"], "(root)")?;
        let version = match doc.root.get("version") {
            Some(Value::Int(v @ (FORMAT_VERSION | FORMAT_VERSION_DEVICES))) => *v,
            Some(Value::Int(v)) => return e(format!("unsupported format version {v}")),
            _ => return e("version = 1 is required"),
        };
        for name in doc.tables.keys() {
            if name != "teahouse" {
                return e(format!("unknown table [{name}]"));
            }
        }
        for name in doc.arrays.keys() {
            match name.as_str() {
                "member" => {}
                "device" if version == FORMAT_VERSION_DEVICES => {}
                "device" => return e("[[device]] needs version = 2"),
                _ => return e(format!("unknown table [[{name}]]")),
            }
        }
        //| `let Some(x) = ... else { ... }` 的意思是：如果是 `Some`，就取出值继续；否则执行
        //| `else` 里的代码，那里必须离开这个函数。这种写法让“出错就走”的情况留在最前面，正常
        //| 流程不用越缩越深。
        let Some(house) = doc.tables.get("teahouse") else {
            return e("[teahouse] table is required");
        };
        check_keys(house, &["name", "signboard"], "teahouse")?;
        let name = take_str(house, "name", "teahouse")?;
        let signboard = match house.get("signboard") {
            None => None,
            Some(Value::Str(s)) => Some(s.clone()),
            Some(_) => return e("teahouse.signboard must be a string"),
        };

        //| Rust 里的变量默认不能改，要改必须写 `mut`。这里逐行读 `[[member]]`，放进 `members`。
        //| `ok_or_else` 把 `Option` 变成 `Result`：`Role::parse` 只会说“没有”，这里补上具体的
        //| 出错说明。
        let mut members = Vec::new();
        for (i, t) in doc.arrays.get("member").into_iter().flatten().enumerate() {
            let ctx = format!("member[{i}]");
            check_keys(t, &["id", "name", "role", "key", "kind", "agent"], &ctx)?;
            let id = take_str(t, "id", &ctx)?;
            let mname = take_str(t, "name", &ctx)?;
            let role_s = take_str(t, "role", &ctx)?;
            let role = Role::parse(&role_s).ok_or_else(|| {
                MembersError(format!(
                    "{ctx}.role {role_s:?} must be host, steward or guest"
                ))
            })?;
            let key = PublicKey::parse(&take_str(t, "key", &ctx)?)
                .map_err(|err| MembersError(format!("{ctx}.key: {err}")))?;
            let kind = match t.get("kind") {
                None => Kind::Human,
                Some(Value::Str(k)) => Kind::parse(k).ok_or_else(|| {
                    MembersError(format!("{ctx}.kind {k:?} must be human or machine"))
                })?,
                Some(_) => return e(format!("{ctx}.kind must be a string")),
            };
            let agent = match t.get("agent") {
                None => None,
                Some(Value::Str(a)) => Some(a.clone()),
                Some(_) => return e(format!("{ctx}.agent must be a string")),
            };
            members.push(Member {
                id,
                name: mname,
                role,
                key,
                kind,
                agent,
                devices: Vec::new(),
            });
        }
        //| `[[device]]` 写在成员后面，每条指明属于谁。`iter_mut` 表示要边找边改，找不到主人
        //| 就拒绝。
        for (i, t) in doc.arrays.get("device").into_iter().flatten().enumerate() {
            let ctx = format!("device[{i}]");
            check_keys(t, &["member", "name", "key"], &ctx)?;
            let owner = take_str(t, "member", &ctx)?;
            let name = match t.get("name") {
                None => None,
                Some(Value::Str(n)) => Some(n.clone()),
                Some(_) => return e(format!("{ctx}.name must be a string")),
            };
            let key = PublicKey::parse(&take_str(t, "key", &ctx)?)
                .map_err(|err| MembersError(format!("{ctx}.key: {err}")))?;
            let Some(m) = members.iter_mut().find(|m| m.id == owner) else {
                return e(format!("{ctx}.member {owner:?} has no seat"));
            };
            m.devices.push(Device { name, key });
        }

        //| 最后这一步是这个函数的关键：读完了不算数，必须再过一遍 `validate`。不论一张表是从
        //| 哪里来的，经过的都是同一道检查。
        let m = Members {
            name,
            signboard,
            members,
        };
        m.validate()?;
        Ok(m)
    }

    //| ## 规矩写成代码
    //|
    //| 这一段就是文件开头那七条规矩的代码。人数上限 15 在这里，恰好一个老板在这里，
    //| 最多一个伙计也在这里。Relay 收到一次改动座位表的推送时，调用的就是
    //| `Members::parse`，于是也会走到这里。所以这几行不是建议，是一扇关着的门。
    //|
    //| 下面的例子真的坐满了十五个人，再加第十六个：
    /// Checks every invariant. Called on parse and before serialising.
    ///
    /// ```
    /// use teahouse_core::{Member, Members, PublicKey, Role};
    ///
    /// let key = |n: u8| PublicKey::from_point(&[n; 32]);
    /// let mut house = Members {
    ///     name: "梧桐里三号院".into(),
    ///     signboard: None,
    ///     members: vec![Member::human("chen", "陈立", Role::Host, key(1))],
    /// };
    /// for n in 2..=15 {
    ///     let guest = Member::human(format!("m{n}"), format!("客人{n}"), Role::Guest, key(n));
    ///     house.members.push(guest);
    /// }
    /// assert!(house.validate().is_ok()); // 十五人，坐满了
    ///
    /// house.members.push(Member::human("m16", "第十六位", Role::Guest, key(16)));
    /// assert_eq!(
    ///     house.validate().unwrap_err().to_string(),
    ///     "members.toml: 16 members; a teahouse has at most 15 seats",
    /// );
    /// ```
    pub fn validate(&self) -> Result<(), MembersError> {
        check_text(&self.name, "teahouse.name", 64)?;
        if let Some(s) = &self.signboard {
            check_text(s, "teahouse.signboard", 280)?;
        }
        let n = self.members.len();
        if n == 0 {
            return e("a teahouse needs at least one member (the host)");
        }
        if n > MAX_MEMBERS {
            return e(format!(
                "{n} members; a teahouse has at most {MAX_MEMBERS} seats"
            ));
        }
        //| `filter` 留下符合条件的，`count` 数一数。`hosts != 1` 同时拦住了“没有老板”和“两个
        //| 老板“。
        let hosts = self.members.iter().filter(|m| m.role == Role::Host).count();
        if hosts != 1 {
            return e(format!("exactly one host is required, found {hosts}"));
        }
        let stewards = self
            .members
            .iter()
            .filter(|m| m.role == Role::Steward)
            .count();
        if stewards > 1 {
            return e(format!("at most one steward is allowed, found {stewards}"));
        }
        //| `enumerate` 给每个元素配上序号。`&self.members[..i]` 取出它前面的那一段，用来查重：
        //| 只和前面的比，每一对只比一次。
        for (i, m) in self.members.iter().enumerate() {
            if !valid_member_id(&m.id) {
                return e(format!(
                    "member id {:?} is invalid (lowercase letters, digits, '-', '_'; max 32)",
                    m.id
                ));
            }
            check_text(&m.name, &format!("member {:?} name", m.id), 64)?;
            match &m.agent {
                Some(_) if !m.kind.is_machine() => {
                    return e(format!(
                        "member {:?}: `agent` is only for machines (kind = \"machine\")",
                        m.id
                    ))
                }
                Some(a) => check_text(a, &format!("member {:?} agent", m.id), 64)?,
                None => {}
            }
            // 法则第 18 条: 1–5 devices, named devices told apart.
            if 1 + m.devices.len() > MAX_DEVICES {
                return e(format!(
                    "member {:?} has {} devices; at most {MAX_DEVICES} / 每人最多 {MAX_DEVICES} 台设备",
                    m.id,
                    1 + m.devices.len()
                ));
            }
            for (j, d) in m.devices.iter().enumerate() {
                if let Some(n) = &d.name {
                    check_text(n, &format!("member {:?} device name", m.id), 32)?;
                    if m.devices[..j].iter().any(|o| o.name.as_ref() == Some(n)) {
                        return e(format!("member {:?} has two devices named {n:?}", m.id));
                    }
                }
            }
            for other in &self.members[..i] {
                if other.id == m.id {
                    return e(format!("duplicate member id {:?}", m.id));
                }
            }
        }
        //| 最后查钥匙。Relay 是凭钥匙认人的，一把钥匙只要属于两个座位，“这是谁”就没有答案了。
        //| 所以先把所有人的所有钥匙摊平成一串，再逐个查重。
        // Every key, of every device, belongs to exactly one seat.
        let all: Vec<(&str, &PublicKey)> = self
            .members
            .iter()
            .flat_map(|m| m.keys().map(move |k| (m.id.as_str(), k)))
            .collect();
        for (i, (id, k)) in all.iter().enumerate() {
            if let Some((other, _)) = all[..i].iter().find(|(_, o)| o == k) {
                if other == id {
                    return e(format!("member {id:?} lists the same key twice"));
                }
                return e(format!("members {other:?} and {id:?} share the same key"));
            }
        }
        Ok(())
    }

    /// The member any of whose devices has this key.
    //| ## 找人
    //|
    //| `by_key` 是整个项目最常被走到的路径之一：OpenSSH 确认了你手里有这把钥匙，Relay 用
    //| `by_key` 查它是谁。查不到返回 `None`，那就是没有座位。
    //|
    //| 下面的 `host` 用了 `.expect(...)`：如果找不到就让程序立刻崩溃。这里敢这么写，是因为
    //| 一张过了 `validate` 的表必然恰好有一个老板。不过编译器并不保证这一点：`Members` 的
    //| 字段都是 `pub`，本章的例子就直接拼出过一张表。所以这是约定：从文件读进来的表都走过
    //| `parse`，手工拼的表用之前要自己 `validate`。`.expect` 里的文字记的就是这个前提。
    pub fn by_key(&self, key: &PublicKey) -> Option<&Member> {
        self.members.iter().find(|m| m.has_key(key))
    }

    pub fn by_id(&self, id: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.id == id)
    }

    pub fn host(&self) -> &Member {
        self.members
            .iter()
            .find(|m| m.role == Role::Host)
            .expect("validated: exactly one host")
    }

    pub fn steward(&self) -> Option<&Member> {
        self.members.iter().find(|m| m.role == Role::Steward)
    }

    //| ## 写回文件
    //|
    //| `to_toml` 把 `Members` 写回文本，第一行又是 `validate`：错的表写不出去。读和写都守着
    //| 同一道门。没有人用多台设备时写 `version = 1`，这样的茶馆，文件格式就不会变。
    /// Canonical serialisation. Validates first; never writes a broken file.
    pub fn to_toml(&self) -> Result<String, MembersError> {
        self.validate()?;
        let q = toml::quote;
        let mut s = String::new();
        s.push_str("# 茶馆成员表 · Teahouse members\n");
        s.push_str("# Only the host may change this file. The relay enforces it.\n");
        let has_devices = self.members.iter().any(|m| !m.devices.is_empty());
        let version = if has_devices {
            FORMAT_VERSION_DEVICES
        } else {
            FORMAT_VERSION
        };
        s.push_str(&format!("version = {version}\n\n[teahouse]\n"));
        s.push_str(&format!("name = {}\n", q(&self.name)));
        if let Some(sb) = &self.signboard {
            s.push_str(&format!("signboard = {}\n", q(sb)));
        }
        for m in &self.members {
            s.push_str("\n[[member]]\n");
            s.push_str(&format!("id = {}\n", q(&m.id)));
            s.push_str(&format!("name = {}\n", q(&m.name)));
            s.push_str(&format!("role = {}\n", q(m.role.as_str())));
            if m.kind.is_machine() {
                s.push_str(&format!("kind = {}\n", q(m.kind.as_str())));
            }
            if let Some(a) = &m.agent {
                s.push_str(&format!("agent = {}\n", q(a)));
            }
            s.push_str(&format!("key = {}\n", q(&m.key.openssh())));
        }
        for m in &self.members {
            for d in &m.devices {
                s.push_str("\n[[device]]\n");
                s.push_str(&format!("member = {}\n", q(&m.id)));
                if let Some(n) = &d.name {
                    s.push_str(&format!("name = {}\n", q(n)));
                }
                s.push_str(&format!("key = {}\n", q(&d.key.openssh())));
            }
        }
        Ok(s)
    }

    //| ## 继承
    //|
    //| `succeed` 是整个文件里最短、也最有茶馆特色的函数：伙计升为老板，原来的老板留一个
    //| 茶客的座位。`self.clone()` 复制一份来改，原来的表不动；改完再过一次 `validate`。
    //|
    //| Relay 的 `adopt` 命令重开茶馆时用的就是它。下面这个例子跑的是同一段逻辑：
    /// 继承: the steward becomes host; the previous host keeps a guest seat.
    ///
    /// ```
    /// use teahouse_core::{Member, Members, PublicKey, Role};
    ///
    /// let key = |n: u8| PublicKey::from_point(&[n; 32]);
    /// let house = Members {
    ///     name: "梧桐里三号院".into(),
    ///     signboard: None,
    ///     members: vec![
    ///         Member::human("chen", "陈立", Role::Host, key(1)),
    ///         Member::human("zhou", "周小满", Role::Steward, key(2)),
    ///     ],
    /// };
    /// let next = house.succeed().unwrap();
    /// assert_eq!(next.host().id, "zhou");
    /// assert_eq!(next.by_id("chen").unwrap().role, Role::Guest);
    /// ```
    pub fn succeed(&self) -> Result<Members, MembersError> {
        let Some(steward) = self.steward() else {
            return e("this teahouse has no steward, so there is no heir");
        };
        let steward_id = steward.id.clone();
        let mut next = self.clone();
        for m in &mut next.members {
            if m.id == steward_id {
                m.role = Role::Host;
            } else if m.role == Role::Host {
                m.role = Role::Guest;
            }
        }
        next.validate()?;
        Ok(next)
    }
}

//| ## 证明
//|
//| 这一节是测试。`#[cfg(test)]` 表示这些代码只在运行测试时才编译，发布的程序里没有。
//| `#[test]` 标出一个测试函数，`assert!` 是“这件事必须成立，否则算失败”。
//|
//| 它们也是这张表的说明书：想知道“16 个人行不行”“两个老板行不行”，这里写得最清楚，
//| 而且每次都真的跑过。`invariants` 测试把文件开头的规矩逐条试了一遍。
#[cfg(test)]
mod tests {
    use super::*;

    fn key(n: u8) -> PublicKey {
        PublicKey::from_point(&[n; 32])
    }

    fn member(id: &str, role: Role, n: u8) -> Member {
        Member::human(id, id.to_uppercase(), role, key(n))
    }

    fn sample() -> Members {
        Members {
            name: "梧桐里三号院".into(),
            signboard: Some("车位、垃圾、楼道灯，咱们自己管。".into()),
            members: vec![
                member("chenli", Role::Host, 1),
                member("zhou", Role::Steward, 2),
                member("wang", Role::Guest, 3),
            ],
        }
    }

    #[test]
    fn roundtrip() {
        let m = sample();
        let text = m.to_toml().unwrap();
        assert_eq!(Members::parse(&text).unwrap(), m);
    }

    #[test]
    fn invariants() {
        let mut m = sample();
        m.members.clear();
        assert!(m.validate().is_err(), "zero members");

        let mut m = sample();
        for i in 0..13 {
            m.members
                .push(member(&format!("g{i}"), Role::Guest, 10 + i));
        }
        assert_eq!(m.members.len(), 16);
        assert!(m.validate().is_err(), "16 members");
        m.members.pop();
        assert!(m.validate().is_ok(), "15 members is fine");

        let mut m = sample();
        m.members[2].role = Role::Host;
        assert!(m.validate().is_err(), "two hosts");

        let mut m = sample();
        m.members[0].role = Role::Guest;
        assert!(m.validate().is_err(), "no host");

        let mut m = sample();
        m.members[2].role = Role::Steward;
        assert!(m.validate().is_err(), "two stewards");

        let mut m = sample();
        m.members[2].id = "zhou".into();
        assert!(m.validate().is_err(), "duplicate id");

        let mut m = sample();
        m.members[2].key = key(1);
        assert!(m.validate().is_err(), "shared key");

        for bad in [
            "",
            "Upper",
            "-x",
            "has space",
            "a/b",
            "x".repeat(33).as_str(),
        ] {
            let mut m = sample();
            m.members[2].id = bad.into();
            assert!(m.validate().is_err(), "bad id {bad:?}");
        }
    }

    #[test]
    fn strict_schema() {
        let good = sample().to_toml().unwrap();
        assert!(Members::parse(&good.replace("version = 1", "version = 3")).is_err());
        assert!(Members::parse(&good.replace("version = 1\n", "")).is_err());
        assert!(Members::parse(&format!("{good}\nadmin = true\n")).is_err());
        assert!(Members::parse(&good.replace("role = \"guest\"", "role = \"admin\"")).is_err());
        assert!(Members::parse(&good.replace("[teahouse]", "[teahouse]\nowner = \"x\"")).is_err());
        assert!(Members::parse(&format!("{good}\n[extra]\n")).is_err());
    }

    #[test]
    fn succession() {
        let next = sample().succeed().unwrap();
        assert_eq!(next.host().id, "zhou");
        assert_eq!(next.by_id("chenli").unwrap().role, Role::Guest);
        assert!(next.steward().is_none());

        let mut no_heir = sample();
        no_heir.members.retain(|m| m.role != Role::Steward);
        assert!(no_heir.succeed().is_err());
    }

    #[test]
    fn machines_are_marked_in_any_role() {
        let k = |n: u8| key(n).openssh();
        let chart = |host_extra: &str, extra: &str| {
            format!(
                "version = 1\n[teahouse]\nname = \"开发组\"\n\n[[member]]\nid = \"u\"\nname = \"用户\"\nrole = \"host\"\n{host_extra}key = \"{}\"\n\n[[member]]\nid = \"ai-2\"\nname = \"2 号\"\n{extra}key = \"{}\"\n",
                k(1),
                k(2)
            )
        };
        let m = Members::parse(&chart(
            "",
            "role = \"guest\"\nkind = \"ai\"\nagent = \"Claude Code\"\n",
        ))
        .unwrap();
        let ai = m.by_id("ai-2").unwrap();
        assert_eq!(
            (ai.kind, ai.agent.as_deref()),
            (Kind::Ai, Some("Claude Code"))
        );
        assert_eq!(m.host().kind, Kind::Human);
        // Round trip, and humans carry no kind line.
        let t = m.to_toml().unwrap();
        assert_eq!(Members::parse(&t).unwrap(), m);
        assert_eq!(t.matches("kind = ").count(), 1);

        // 法则第 17 条: a label, not a rank. An AI may be the steward, or the host.
        assert!(Members::parse(&chart("", "role = \"steward\"\nkind = \"ai\"\n")).is_ok());
        let ai_host = Members::parse(&chart("kind = \"ai\"\n", "role = \"guest\"\n")).unwrap();
        assert_eq!(ai_host.host().kind, Kind::Ai);
        // …and an AI steward inherits like anyone.
        let m2 = Members::parse(&chart("", "role = \"steward\"\nkind = \"ai\"\n")).unwrap();
        assert_eq!(m2.succeed().unwrap().host().id, "ai-2");

        // Machines: `machine`, and the older `ai`, each kept as written.
        let m = Members::parse(&chart(
            "",
            "role = \"guest\"\nkind = \"machine\"\nagent = \"GitHub Actions\"\n",
        ))
        .unwrap();
        let ci = m.by_id("ai-2").unwrap();
        assert_eq!((ci.kind, ci.kind.label()), (Kind::Machine, "machine"));
        assert_eq!(
            (Kind::Ai.label(), Kind::Human.label()),
            ("machine", "human")
        );
        assert!(m.to_toml().unwrap().contains("kind = \"machine\"\n"));
        let old = Members::parse(&chart("", "role = \"guest\"\nkind = \"ai\"\n")).unwrap();
        assert!(old.to_toml().unwrap().contains("kind = \"ai\"\n"));

        for (bad, why) in [
            (
                "role = \"guest\"\nagent = \"Claude Code\"\n",
                "only for machines",
            ),
            ("role = \"guest\"\nkind = \"robot\"\n", "human or machine"),
            ("role = \"guest\"\nkind = 1\n", "must be a string"),
        ] {
            let err = Members::parse(&chart("", bad)).unwrap_err();
            assert!(err.0.contains(why), "{bad}: {err}");
        }
    }

    #[test]
    fn devices() {
        let k = |n: u8| key(n).openssh();
        let base = format!(
            "version = 2\n[teahouse]\nname = \"梧桐里\"\n\n[[member]]\nid = \"wang\"\nname = \"王阿姨\"\nrole = \"host\"\nkey = \"{}\"\n\n[[member]]\nid = \"li\"\nname = \"李\"\nrole = \"guest\"\nkey = \"{}\"\n",
            k(1),
            k(2)
        );
        let dev = |member: &str, name: Option<&str>, n: u8| {
            let name = name
                .map(|x| format!("name = \"{x}\"\n"))
                .unwrap_or_default();
            format!(
                "\n[[device]]\nmember = \"{member}\"\n{name}key = \"{}\"\n",
                k(n)
            )
        };
        let m =
            Members::parse(&(base.clone() + &dev("wang", Some("手机"), 3) + &dev("wang", None, 4)))
                .unwrap();
        let wang = m.by_id("wang").unwrap();
        assert_eq!(wang.keys().count(), 3);
        assert_eq!(
            m.by_key(&key(3)).unwrap().id,
            "wang",
            "any device names the member"
        );
        assert_eq!(wang.device_name(&key(3)), Some("手机"));
        // Round trip; a chart with devices is version 2, without them version 1.
        let t = m.to_toml().unwrap();
        assert!(t.contains("version = 2") && t.contains("[[device]]"));
        assert_eq!(Members::parse(&t).unwrap(), m);
        let mut plain = m.clone();
        plain.members[0].devices.clear();
        assert!(
            plain.to_toml().unwrap().starts_with("# 茶馆成员表")
                && plain.to_toml().unwrap().contains("version = 1")
        );

        // Removing: another device can go; the first one is replaced by the
        // next; the last one stays.
        let mut w = wang.clone();
        w.remove_key(&key(1)).unwrap();
        assert_eq!((w.key.clone(), w.devices.len()), (key(3), 1));
        w.remove_key(&key(4)).unwrap();
        let err = w.remove_key(&key(3)).unwrap_err();
        assert!(err.0.contains("last device"), "{err}");
        assert!(w.remove_key(&key(9)).is_err());

        for (bad, why) in [
            (
                base.replace("version = 2", "version = 1") + &dev("wang", None, 3),
                "needs version = 2",
            ),
            (base.clone() + &dev("zhao", None, 3), "has no seat"),
            (base.clone() + &dev("wang", None, 2), "share the same key"),
            (base.clone() + &dev("wang", None, 1), "same key twice"),
            (
                base.clone() + &dev("wang", Some("手机"), 3) + &dev("wang", Some("手机"), 4),
                "two devices named",
            ),
            (
                base.clone()
                    + &dev("wang", None, 3)
                    + &dev("wang", None, 4)
                    + &dev("wang", None, 5)
                    + &dev("wang", None, 6)
                    + &dev("wang", None, 7),
                "at most 5",
            ),
            (
                base.clone() + "\n[[device]]\nmember = \"wang\"\ncolor = \"red\"\nkey = \"x\"\n",
                "unknown field",
            ),
        ] {
            let err = Members::parse(&bad).unwrap_err();
            assert!(err.0.contains(why), "{why}: {err}");
        }
        // Five devices is fine.
        let five = base
            + &dev("wang", None, 3)
            + &dev("wang", None, 4)
            + &dev("wang", None, 5)
            + &dev("wang", None, 6);
        assert_eq!(
            Members::parse(&five)
                .unwrap()
                .by_id("wang")
                .unwrap()
                .keys()
                .count(),
            5
        );
    }
}

//| ## 试一试
//|
//| 这本书可以跑。在仓库根目录运行：
//|
//| ```sh
//| cargo test -p teahouse-core
//| ```
//|
//| 下面三个实验都真的做过，结果原样贴在这里（行号省略了，你那边的数字可能不同）。
//| 做完记得用 `git checkout core/` 改回来。
//|
//| **实验一：多开一个座位。** 把 `MAX_MEMBERS` 从 15 改成 16，再跑测试：
//|
//| ```text
//| ---- members::tests::invariants stdout ----
//| panicked at core/src/members.rs:
//| 16 members
//|
//| failures:
//|     members::tests::invariants
//| ```
//|
//| 改动一个数字，代码能编译，但测试立刻指出“16 members”这条失败了。规矩写成测试以后，
//| 谁也不能不声不响地改掉它。
//|
//| **实验二：忘掉一种座位。** 在 `Role::zh` 里删掉 `Role::Guest => "茶客",` 这一行：
//|
//| ```text
//| error[E0004]: non-exhaustive patterns: `Role::Guest` not covered
//|   --> core/src/members.rs
//|    |
//|    |         match self {
//|    |               ^^^^ pattern `Role::Guest` not covered
//| ```
//|
//| 这次连编译都过不去。前面说 `match` 必须覆盖所有可能，就是这个意思：这种错误到不了
//| 运行的时候。
//|
//| **实验三：放宽伙计的路径。** 这一处在 `core/src/rules.rs` 里，以后会讲到：把
//| `may_write_on_main` 的路径判断改成永远为真，让伙计什么都能写：
//|
//| ```text
//| ---- rules::tests::paths stdout ----
//| assertion failed: !may_write_on_main(Role::Steward, ".teahouse/members.toml")
//| ```
//|
//| 测试抓住的正好是最危险的那件事：伙计改了座位表，也就能给自己升成老板。
