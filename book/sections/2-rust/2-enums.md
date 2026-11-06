# enum、Option 和 Result

茶馆的代码几乎每个函数都用到这一节的东西。

## 小测验

下面的 `?` 做了什么？

```rust,ignore
let key = PublicKey::parse(&text)?;
```

答得出“出错就把错误返回给调用者，否则取出值”，并且知道这要求函数返回 `Result`，就可以去第三部分了。

## enum 是带数据的选项

大多数语言的枚举只是几个名字。Rust 的 `enum` 每个分支还能带数据：

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

`match` 必须覆盖所有分支。少写一个，编译不过。Relay 判断权限时用的就是这种结构，所以新增一种情况时，每一处判断都必须有人想过。

## Option 代替 null

Rust 没有 `null`。可能没有的值用 `Option`：

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

`Option<usize>` 不是 `usize`，不处理就用不了里面的数字。其他语言里的空指针错误，在这里编译不过。

## Result 代替异常

可能失败的函数返回 `Result`，错误是普通的值。`?` 让它用起来很轻：

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

`?` 遇到 `Err` 就立刻返回。函数的签名上写着它可能失败，调用者不能假装没看见。

## 在茶馆里

这是真实代码。座位类型来自文件里的文字，写错了就返回 `None`：

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

## 练习

给 `parse_seats` 加一条规则：0 人也要拒绝，错误文字是 `至少要有一个人`。

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

</details>
