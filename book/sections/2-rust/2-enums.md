# 用类型表达“不可能”

上一节讲 Rust 怎样消灭内存错误。这一节讲它怎样消灭另一类更常见的错误：逻辑上不该出现的状态。这是我认为 Rust 对日常编程最大的贡献，也是读懂茶馆代码的关键。

## 小测验

下面的 `?` 做了什么？`main` 能写这一行吗？

```rust,ignore
let key = PublicKey::parse(&text)?;
```

如果你知道 `?` 在出错时把错误返回给调用者，并且会经过 `From` 转换错误类型，这一节可以略读，直接去第三部分。

## 坏的状态应该写不出来

用一般的语言写一条判决，你大概会这样设计：

```text
class Decision { bool allowed; String reason; }
```

这个结构允许四种组合，其中两种没有意义：允许但带着拒绝理由，拒绝却没有理由。程序的每个读者都要记住“这两种不会出现”，而编译器不知道这条约定。

Rust 的 `enum` 让你直接写出真正存在的情况：

```rust
enum Decision {
    Allow,
    Deny(String),
}

fn explain(d: Decision) -> String {
    match d {
        Decision::Allow => String::from("可以"),
        Decision::Deny(why) => format!("不行：{why}"),
    }
}

fn main() {
    println!("{}", explain(Decision::Allow));
    println!("{}", explain(Decision::Deny(String::from("茶馆没有这个分支"))));
}
```

```text,output
可以
不行：茶馆没有这个分支
```

每个分支可以带不同的数据。只有拒绝才有理由，没有理由的拒绝根本写不出来。这种类型在理论上叫**和类型**（sum type），函数式语言用了几十年，Rust 把它带进了系统编程。

这个例子不是编的。这是茶馆判决一个分支操作的真实类型：

```rust,ignore
// core/src/rules.rs
pub enum RefDecision {
    Allow,
    Deny(String),
}
```

## match 必须穷尽

`match` 要求覆盖所有分支。这件事的价值在**修改代码的时候**才显现。

比如以后茶馆加入第四种座位，`Role` 多了一个值。编译器会列出全部没有处理它的 `match`，一处都漏不掉。在别的语言里，你得靠全局搜索，再祈祷没有漏网。

所以写 `match` 时尽量不用 `_` 兜底。茶馆判断权限的函数是这样写的，三种座位一个不落：

```rust,ignore
// core/src/rules.rs
pub fn may_write_on_main(role: Role, path: &str) -> bool {
    match role {
        Role::Host => true,
        Role::Steward => STEWARD_DIRS
            .iter()
            .any(|d| path.starts_with(d) && path.len() > d.len()),
        Role::Guest => false,
    }
}
```

## Option：十亿美元的错误

托尼·霍尔在 1965 年发明了空引用。2009 年他公开道歉，称之为“十亿美元的错误”：它让每个引用都可能是空的，而类型系统对此一无所知。

Rust 没有 `null`。可能没有的值用 `Option<T>` 表示，它只是标准库里一个普通的 `enum`：

```rust
fn find(names: &[&str], who: &str) -> Option<usize> {
    names.iter().position(|n| *n == who)
}

fn main() {
    let names = ["陈立", "周小满"];
    match find(&names, "周小满") {
        Some(seat) => println!("第 {seat} 号座位"),
        None => println!("没有座位"),
    }
    println!("{:?}", find(&names, "路人"));
}
```

```text,output
第 1 号座位
None
```

`Option<usize>` 和 `usize` 是两个类型。不处理 `None`，就拿不到里面的数字，编译器不答应。

Relay 用钥匙找人，就是这样：

```rust,ignore
// relay/src/hook.rs
    let Some(pusher) = members.by_key(&key) else {
```

`let ... else` 的意思是：匹配上了就继续，否则必须离开函数。没有座位的情况不可能被忘记处理。

## Result：错误是一个值

异常有个毛病：函数的签名上看不出它会不会抛异常，错误可以穿过好几层调用，到达没人准备好的地方。

Rust 把可能的失败写进返回类型：

```rust
fn parse_seats(text: &str) -> Result<u32, String> {
    let n: u32 = text.parse().map_err(|_| format!("{text} 不是数字"))?;
    if n > 15 {
        return Err(format!("{n} 人太多了"));
    }
    Ok(n)
}

fn main() {
    println!("{:?}", parse_seats("9"));
    println!("{:?}", parse_seats("16"));
    println!("{:?}", parse_seats("九"));
}
```

```text,output
Ok(9)
Err("16 人太多了")
Err("九 不是数字")
```

问号 `?` 是这一切好用的原因。它的展开大致是：

```text
match 表达式 {
    Ok(值) => 值,
    Err(错误) => return Err(From::from(错误)),
}
```

成功时取出值，失败时立刻返回。注意 `From::from`：错误类型可以自动转换。Relay 只有一个错误类型，别的错误都能转成它，所以一连串 `?` 写起来干干净净：

```rust,ignore
// relay/src/error.rs
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error(format!("I/O error: {e}"))
    }
}
```

这段设计的结果是：错误处理不会被遗漏，也不会让正常逻辑变得难读。

## 解析，而不是检查

把这一节合起来，就得到一条影响深远的设计原则，Alexis King 在 2019 年把它概括成一句话：**解析，而不是检查**（Parse, don't validate）。

不好的写法是检查一个字符串，然后继续传这个字符串，后面每个地方都得相信“它被检查过”。好的写法是把它变成一个新类型，失败时拿不到这个类型：

```rust,ignore
// core/src/members.rs
    pub fn parse(s: &str) -> Option<Role> {
        match s {
            "host" => Some(Role::Host),
            "steward" => Some(Role::Steward),
            "guest" => Some(Role::Guest),
            _ => None,
        }
    }
```

走过这个函数之后，程序里拿到的是 `Role`，不再是文字。后面所有的代码都不用再担心 `"hsot"` 这种拼写错误，因为这样的值不存在。`Members::parse` 把这个想法用到了整张座位表上。

这就是第 1.5 节那个问题的答案：在数据进入系统的地方把关一次，把它变成一个不可能出错的类型，之后的代码就可以信任它。

## 这一节学到的

- `enum` 的分支可以带数据，让错误的状态写不出来。
- `match` 必须穷尽，修改代码时编译器会替你找出每一处。
- `Option` 代替 `null`，`Result` 代替异常，失败写在类型里。
- `?` 传播错误，`From` 转换错误类型。
- 在系统入口把数据解析成精确的类型，之后不用再检查。

## 练习

给 `parse_seats` 加一条规则：0 人也要拒绝，错误文字是 `至少要有一个人`。然后思考：这个函数返回的 `u32` 能保证“范围在 1 到 15”吗？如果想让类型来保证，该怎么设计？

<details>
<summary>答案</summary>

```rust
fn parse_seats(text: &str) -> Result<u32, String> {
    let n: u32 = text.parse().map_err(|_| format!("{text} 不是数字"))?;
    if n == 0 {
        return Err(String::from("至少要有一个人"));
    }
    if n > 15 {
        return Err(format!("{n} 人太多了"));
    }
    Ok(n)
}

fn main() {
    println!("{:?}", parse_seats("0"));
}
```

```text,output
Err("至少要有一个人")
```

`u32` 不能保证，调用者拿到后仍可能传错。可以定义 `struct SeatCount(u32);`，字段不公开，只有 `parse` 能造出它。这样只要手里有一个 `SeatCount`，它就一定合法。

</details>
