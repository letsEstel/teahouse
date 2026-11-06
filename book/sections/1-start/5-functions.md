# 把步骤打包成函数

上一节的练习里，“人数合不合格”这个判断写在 `main` 里。如果程序有十个地方要用它，就得写十遍。这一节，我们给一段指令起个名字，到处使用。这样的一段指令叫**函数**。

你已经写过一个函数了：`main`。

## 写一个函数

```rust
fn welcome() {
    println!("欢迎来到茶馆");
}

fn main() {
    welcome();
    welcome();
}
```

```text,output
欢迎来到茶馆
欢迎来到茶馆
```

`fn welcome()` 定义函数，`welcome();` 运行它，这叫**调用**。函数写在 `main` 的前面或后面都可以。

## 给函数传值

函数可以接收值。接收的值叫**参数**：

```rust
fn show(seated: u32) {
    println!("现在 {seated} 人");
}

fn main() {
    show(9);
    show(10);
}
```

```text,output
现在 9 人
现在 10 人
```

`seated: u32` 说明参数的名字和**类型**。类型说明这是哪一种值。`u32` 是不带负号的整数，人数不可能是负的，正好合适。

为什么 `let` 不用写类型，这里却要写？函数内部，编译器能自己推断出来；函数的参数是这段代码对外的约定，Rust 要求写清楚。

## 让函数给出答案

函数还可以算出一个值，交还给调用它的地方：

```rust
fn has_free_seat(seated: u32) -> bool {
    seated < 15
}

fn main() {
    let answer = has_free_seat(9);
    println!("{answer}");
    println!("{}", has_free_seat(15));
}
```

```text,output
true
false
```

`-> bool` 说明这个函数交还一个布尔值。交还的那个值叫**返回值**。

注意 `seated < 15` 后面没有分号。函数最后一行不写分号时，这一行的值就是返回值。

## 让电脑替你检查

函数写好了，怎么知道它是对的？可以手动运行，一个个看结果。更好的办法是把检查也写成代码，这叫**测试**。

```rust,test
fn has_free_seat(seated: u32) -> bool {
    seated < 15
}

#[test]
fn nine_people_can_add_one() {
    assert!(has_free_seat(9));
}

#[test]
fn fifteen_people_is_full() {
    assert!(!has_free_seat(15));
}

fn main() {}
```

`#[test]` 标出一个测试。`assert!(...)` 表示括号里的结果必须是 `true`，否则这个测试失败。`!` 把 `true` 和 `false` 翻过来，`!has_free_seat(15)` 的意思是“没有空座”。

运行：

```sh
cargo test
```

```text,output
running 2 tests
test fifteen_people_is_full ... ok
test nine_people_can_add_one ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

以后谁把 15 改错了，`cargo test` 马上会失败。

## 这和真实的茶馆有什么关系

茶馆的法则之一是一间茶馆最多 15 人。在真实的代码 `core/src/members.rs` 里，有这样两行：

```text
pub const MAX_MEMBERS: usize = 15;
```

```text
if n > MAX_MEMBERS {
```

现在你已经认得其中的 `if` 和比较了。`const` 定义一个永远不变的值，`usize` 是另一种整数类型。Relay 判断一次写入能不能进茶馆时，靠的就是这样一层层的函数和测试。后面的部分会带你读到它们。

## 这一节学到的

- `fn` 定义函数，`名字()` 调用函数。
- 参数要写类型，`->` 说明返回值的类型。
- 最后一行不写分号，它的值就是返回值。
- `#[test]`、`assert!` 和 `cargo test` 让电脑替你检查。

## 练习

写一个函数 `free_seats`，接收当前人数，返回空座数量。再写两个测试：9 人时返回 6，15 人时返回 0。提示：比较两个值是否相等，用 `assert_eq!(甲, 乙);`。

<details>
<summary>答案</summary>

```rust,test
fn free_seats(seated: u32) -> u32 {
    15 - seated
}

#[test]
fn nine_people() {
    assert_eq!(free_seats(9), 6);
}

#[test]
fn fifteen_people() {
    assert_eq!(free_seats(15), 0);
}

fn main() {}
```

```text,output
running 2 tests
test fifteen_people ... ok
test nine_people ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

</details>
