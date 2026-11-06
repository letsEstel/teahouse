# 函数：控制复杂度的唯一办法

一个程序写到几百行，人就记不住了。茶馆有三万行。没有人能同时想着三万行代码，能做的只有一件事：把一段做好的事包起来，起个名字，以后只想名字，不想里面。这就是**函数**，也是这本书最重要的一个想法。

## 定义和调用

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

`fn welcome()` 是**定义**，这时什么都不执行。`welcome();` 是**调用**，这时程序跳进函数，做完再跳回来。函数定义的先后不重要，写在 `main` 后面也可以。

## 参数和返回值

一个有用的函数像一台机器：放进原料，出来成品。

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

`seated: u32` 是**参数**，说明它要接收什么。`-> bool` 说明它交还什么，这叫**返回值**。

这一行就是这个函数对外的全部承诺，叫**函数签名**。调用它的人只需要读这一行。局部变量的类型编译器可以推断，签名却必须写全，这是 Rust 故意的：签名是不同部分之间的合同，合同要写清楚，而且修改函数内部时，不会意外改变合同。

## 语句和表达式

注意 `seated < 15` 后面没有分号。这是 Rust 的一个核心设计。

**表达式**算出一个值，比如 `1 + 2`、`seated < 15`，上一节的 `if` 也是。**语句**做一件事，不产生值。在表达式后面加分号，它就变成语句，值被丢掉。

函数体最后一个不带分号的表达式，就是返回值。加上分号是初学者最常见的错误：

<p class="wont">这段故意编译不过</p>

```rust,compile_fail,E0308
fn has_free_seat(seated: u32) -> bool {
    seated < 15;
}

fn main() {
    println!("{}", has_free_seat(9));
}
```

```text,error
error[E0308]: mismatched types
```

函数答应返回 `bool`，分号却把值扔掉了。想提前返回时用 `return`：

```rust
fn describe(seated: u32) -> &'static str {
    if seated == 0 {
        return "空的";
    }
    if seated >= 15 {
        return "满了";
    }
    "有人"
}

fn main() {
    println!("{}", describe(0));
    println!("{}", describe(9));
    println!("{}", describe(15));
}
```

```text,output
空的
有人
满了
```

把不正常的情况先处理掉，正常流程留在最后，这种写法叫**提前返回**。茶馆的代码几乎都是这样写的。`&'static str` 这个类型第二部分会讲，现在把它当成一段文字。

## 函数调用时发生了什么

函数能互相调用：

```rust
fn free_seats(seated: u32) -> u32 {
    15 - seated
}

fn has_free_seat(seated: u32) -> bool {
    free_seats(seated) > 0
}

fn main() {
    println!("{}", has_free_seat(9));
}
```

```text,output
true
```

运行时，内存里有一块区域叫**栈**，每调用一个函数就在上面放一块地方，存它的参数和局部变量，返回时拿掉：

```text
  free_seats     seated = 9       ← 正在运行
  has_free_seat  seated = 9
  main
```

调用像叠盘子，后放的先拿。每个函数都有自己的 `seated`，它们同名，但互不相干。`free_seats` 改不到 `main` 的变量，这种隔离正是函数能帮你控制复杂度的原因：看一个函数时，你只需要考虑它自己。

## 纯函数

`has_free_seat` 有一个好性质：结果只取决于参数，也不改变外面的任何东西。这样的函数叫**纯函数**。同样的输入永远得到同样的输出，它不读文件，不上网，不看时钟。

纯函数最容易理解，也最容易测试。茶馆的架构正是围绕这一点建立的：`core/` 里的法则全是纯函数，读文件、跑 git 这些事都放在外层的 `relay/`。这样，决定谁能写什么的代码不需要启动服务器就能测试。

## 让电脑替你检查

你怎么知道函数是对的？手动运行几次，看一看。可是下个月改代码时，你还会再试一遍吗？

把检查写成代码，让电脑每次都跑一遍，这就是**测试**：

```rust,test
fn has_free_seat(seated: u32) -> bool {
    seated < 15
}

#[test]
fn fourteen_people_leave_one_seat() {
    assert!(has_free_seat(14));
}

#[test]
fn fifteen_people_is_full() {
    assert!(!has_free_seat(15));
}

fn main() {}
```

`#[test]` 标出一个测试函数。`assert!(...)` 要求括号里是 `true`，否则测试失败。`!` 把真假翻过来。运行：

```sh
cargo test
```

```text,output
running 2 tests
test fifteen_people_is_full ... ok
test fourteen_people_leave_one_seat ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

注意我们测的是 14 和 15，不是 9 和 100。错误最爱出现在边界上：`<` 写成 `<=`，就差这一个字符。

现在把 `seated < 15` 改成 `seated <= 15`，这是一个真实可能犯的错误，再运行：

```rust,test,fails
fn has_free_seat(seated: u32) -> bool {
    seated <= 15
}

#[test]
fn fifteen_people_is_full() {
    assert!(!has_free_seat(15));
}

fn main() {}
```

```text,error
test fifteen_people_is_full ... FAILED
assertion failed: !has_free_seat(15)
```

测试不仅告诉你错了，还告诉你错在哪条规则上。在茶馆这样的项目里，这一点决定了你敢不敢改代码。

一段没有测试的代码，下一个维护者只能祈祷。有测试的代码，他可以放手去改，改错了测试会喊。所以**测试也是写给人的**：它是永远不会过时的说明书。

## 这和真实的茶馆有什么关系

你现在认识的东西已经够读真实代码的一个片段了。第 1.3 节的练习问，人数超过上限会怎样。茶馆的做法是在最前面拦住它：

```rust,ignore
// core/src/members.rs
        if n > MAX_MEMBERS {
            return e(format!(
                "{n} members; a teahouse has at most {MAX_MEMBERS} seats"
            ));
        }
```

`if`、比较、提前返回，全都认识。而 `MAX_MEMBERS` 的定义你也看得懂了：

```rust,ignore
// core/src/members.rs
pub const MAX_MEMBERS: usize = 15;
```

它的测试真的试了十六个人：

```rust,ignore
// core/src/members.rs
        assert_eq!(m.members.len(), 16);
        assert!(m.validate().is_err(), "16 members");
        m.members.pop();
        assert!(m.validate().is_ok(), "15 members is fine");
```

看，边界上的 15 和 16。你已经在读这个项目了。

## 这一节学到的

- 函数把一段事情包起来，起个名字，这是控制复杂度的办法。
- 函数签名是合同；最后一个不带分号的表达式是返回值。
- 每次调用都有自己的一块栈空间，函数之间互相隔离。
- 纯函数最好懂也最好测，茶馆的法则都是纯函数。
- 测试把检查写成代码，要测边界。

## 练习

写函数 `free_seats`，接收人数，返回空座数量。写测试：0 人时返回 15，15 人时返回 0。比较两个值相等用 `assert_eq!(甲, 乙);`。

再想一想：`free_seats(16)` 会怎样？函数应该负责处理这种情况，还是调用它的人应该保证不发生？

<details>
<summary>答案</summary>

```rust,test
fn free_seats(seated: u32) -> u32 {
    15 - seated
}

#[test]
fn nobody_here() {
    assert_eq!(free_seats(0), 15);
}

#[test]
fn full() {
    assert_eq!(free_seats(15), 0);
}

fn main() {}
```

```text,output
running 2 tests
test full ... ok
test nobody_here ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`free_seats(16)` 会 panic，因为 `u32` 不能是负数。这种问题没有唯一的答案，但茶馆的选择是：在数据进入系统的地方检查一次，之后的代码都可以信任它。第三部分还会回到这个想法。

</details>
