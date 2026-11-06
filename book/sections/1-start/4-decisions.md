# 让程序做判断

到现在为止，程序只会从上到下一条条执行。这一节，我们让它根据情况走不同的路。

## if

新客人来了，还有没有座位？

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

`seated < 15` 是一个问题，答案只有两种：`true`（是）或者 `false`（否）。答案是 `true` 时执行第一对花括号里的指令，否则执行 `else` 后面的。

把第一行改成 `let seated = 9;` 再运行，程序就走另一条路了。

## 比较的写法

| 写法 | 意思 |
|---|---|
| `a < b` | a 小于 b |
| `a > b` | a 大于 b |
| `a <= b` | a 小于或等于 b |
| `a >= b` | a 大于或等于 b |
| `a == b` | a 等于 b |
| `a != b` | a 不等于 b |

注意，比较相等要写两个等号。上一节讲过，一个等号的意思是把值放进变量。

## 不止两条路

茶馆有三种座位：老板、伙计、茶客。用 `else if` 可以接着问：

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

程序从上到下一个个问，走第一个答案是 `true` 的分支，后面的不再看。

## 两个条件

`&&` 表示“并且”，`||` 表示“或者”：

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

`invited` 这个变量存的就是 `true`。这种只有是和否两个值的类型叫**布尔值**。

## 这一节学到的

- `if 条件 { ... } else { ... }` 让程序走不同的路。
- 条件的答案是 `true` 或 `false`。
- 比较相等写 `==`，`&&` 是并且，`||` 是或者。

## 练习

一间茶馆的人数必须在 1 到 15 之间。写一个程序：变量 `seated` 符合这个范围时显示 `合格`，否则显示 `不合格`。分别用 0、9、16 试一试。

<details>
<summary>答案</summary>

```rust
fn main() {
    let seated = 16;
    if seated >= 1 && seated <= 15 {
        println!("合格");
    } else {
        println!("不合格");
    }
}
```

```text,output
不合格
```

</details>
