# 判断：程序的分岔

前几节的程序是一条直线。真正有用的程序要根据情况走不同的路。1936 年，图灵证明了一件事：只要能读写数据、能根据数据决定下一步，一台机器就能计算一切可以计算的东西。判断就是这样一个看起来简单、实际上决定一切的能力。

茶馆的 Relay 整个就是一个巨大的判断：这次推送，放行还是拒绝？

## 真和假

判断的原料是**布尔值**（boolean），只有 `true` 和 `false`，以十九世纪的数学家布尔命名，他发现逻辑可以像代数一样计算。比较运算的结果就是布尔值：

```rust
fn main() {
    let seated = 15;
    let has_room = seated < 15;
    println!("{has_room}");
}
```

```text,output
false
```

| 写法 | 意思 |
|---|---|
| `a < b` | a 小于 b |
| `a > b` | a 大于 b |
| `a <= b` | a 小于或等于 b |
| `a >= b` | a 大于或等于 b |
| `a == b` | a 等于 b |
| `a != b` | a 不等于 b |

比较相等要写两个等号，因为一个等号已经表示“把值放进变量”了。C 语言里把 `==` 误写成 `=` 能通过编译，这个错误几十年来造成过无数漏洞。Rust 里它编译不过。

## if

```rust
fn main() {
    let seated = 15;
    if seated < 15 {
        println!("请坐");
    } else {
        println!("坐满了");
    }
}
```

```text,output
坐满了
```

条件为 `true` 时执行第一对花括号，否则执行 `else` 后面的。两条路只走一条。

条件必须是布尔值。有些语言允许写 `if seated`，把 0 当作假，Rust 不允许：

<p class="wont">这段故意编译不过</p>

```rust,compile_fail,E0308
fn main() {
    let seated = 9;
    if seated {
        println!("有人");
    }
}
```

```text,error
error[E0308]: mismatched types
```

严格的好处是，每个判断都把意思写在明面上。

## 不止两条路

```rust
fn main() {
    let role = "steward";
    if role == "host" {
        println!("可以修改座位表");
    } else if role == "steward" {
        println!("可以直接写成果");
    } else {
        println!("可以发起提案");
    }
}
```

```text,output
可以直接写成果
```

程序从上到下一个个问，走第一个成立的分支，后面的不再看。所以**分支的顺序有意义**。

注意最后的 `else`，它接住了所有没想到的情况。如果 `role` 是写错的 `"hsot"`，这段程序会把它当茶客，这恰好是安全的方向：不认识的人得到最小的权限。如果反过来，最后一个 `else` 写成“可以修改座位表”，一个错字就成了漏洞。

这就是全景里说的**默认拒绝**（fail-safe defaults）。写判断时永远问自己：没想到的情况，会落到哪一边？

## 组合条件

`&&` 表示“并且”，`||` 表示“或者”，`!` 表示“不是”：

| a | b | `a && b` | `a \|\| b` | `!a` |
|---|---|---|---|---|
| true | true | true | true | false |
| true | false | false | true | false |
| false | true | false | true | true |
| false | false | false | false | true |

```rust
fn main() {
    let seated = 9;
    let invited = true;
    if seated < 15 && invited {
        println!("请坐");
    } else {
        println!("请先拿到邀请");
    }
}
```

```text,output
请坐
```

`&&` 有一个重要性质，叫**短路**（short-circuit evaluation）：左边已经是 `false`，右边根本不执行，因为结果已经确定了。这不只是省时间，它让你可以先检查、再使用：

```rust
fn main() {
    let total = 0;
    let free = 0;
    if total != 0 && free / total > 0 {
        println!("还有空位");
    } else {
        println!("没有座位");
    }
}
```

```text,output
没有座位
```

整数除以 0 会让程序崩溃。这里因为左边是 `false`，除法根本没有发生。真实代码里到处是这种写法。下面是茶馆判断伙计能不能写某个路径的一行，你现在应该看得懂结构了：

```rust,ignore
// core/src/rules.rs
            .any(|d| path.starts_with(d) && path.len() > d.len()),
```

路径要以某个目录开头，**并且**比目录名更长。为什么要后半句？因为否则路径 `works/` 本身也算数，而它不是一个文件。这种细节是法则的边界，也是攻击者下手的地方。

## if 能算出一个值

Rust 里 `if` 不只是走路，它本身有值：

```rust
fn main() {
    let seated = 15;
    let sign = if seated < 15 { "营业中" } else { "客满" };
    println!("{sign}");
}
```

```text,output
客满
```

这里 `sign` 没有 `mut`。因为我们一次就算出了它的值，它不需要被改来改去。Rust 程序员偏爱这种写法。

## 这一节学到的

- 比较的结果是布尔值，`if` 根据它选择一条路。
- `&&` 和 `||` 会短路，可以用来先检查再使用。
- 分支的顺序有意义；要想清楚没想到的情况落到哪一边。
- `if` 是一个表达式，能算出值。

## 练习

一间茶馆的人数必须在 1 到 15 之间。用常量和 `&&` 写一个判断：`seated` 符合时显示 `合格`，否则显示 `不合格`。分别用 0、9、15、16 试一试。边界上的值最容易出错，所以这几个数都要试。

<details>
<summary>答案</summary>

```rust
const MAX_SEATS: u32 = 15;

fn main() {
    let seated = 15;
    if seated >= 1 && seated <= MAX_SEATS {
        println!("合格");
    } else {
        println!("不合格");
    }
}
```

```text,output
合格
```

</details>
