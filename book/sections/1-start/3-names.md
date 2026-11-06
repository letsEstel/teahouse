# 值、类型和名字

程序做的所有事情，归根结底是在内存里存放和改变一些值。这一节讲清楚值是什么，为什么每个值都有类型，以及 Rust 为什么默认不让你改变它们。

## 内存是一排格子

内存可以想象成一排编了号的格子，每格存一个字节，也就是 0 到 255 之间的一个数。程序里的一切，数字、文字、图片，最后都是这样的数。

那么内存里存着 `228`，它是什么意思？可能是数字 228，也可能是汉字的一部分。**格子里的数本身没有含义，含义来自我们怎么看待它。** “怎么看待”这件事就叫**类型**（type）。

## 用 let 给值起名字

```rust
fn main() {
    let name = "梧桐里三号院";
    let seats = 15;
    println!("{name} 有 {seats} 个座位");
}
```

```text,output
梧桐里三号院 有 15 个座位
```

`let seats = 15;` 把 15 存进内存，并给它起名 `seats`。这样的名字叫**变量**（variable）。引号里的 `{seats}` 会换成它的值。

起好名字是编程里最被低估的能力。`let n = 15;` 能运行，但三个月后没人知道 `n` 是什么。`seats` 自己就是一句说明。

## 类型

Rust 的每个值都有确定的类型。上面的代码没写，是因为编译器能推断出来。也可以写明，写在冒号后面：

```rust
fn main() {
    let seats: u32 = 15;
    let price: f64 = 2.5;
    let open: bool = true;
    let door: char = '门';
    let name: &str = "梧桐里";
    println!("{seats} {price} {open} {door} {name}");
}
```

```text,output
15 2.5 true 门 梧桐里
```

| 类型 | 是什么 | 例子 |
|---|---|---|
| `u32` | 不带负号的整数，占 32 位 | `15` |
| `i32` | 可以为负的整数，占 32 位 | `-3` |
| `f64` | 带小数的数 | `2.5` |
| `bool` | 只有 `true` 和 `false` | `true` |
| `char` | 一个字符，用单引号 | `'门'` |
| `&str` | 一段文字，用双引号 | `"梧桐里"` |

`u` 是 unsigned，不带符号。为什么有这么多种整数？因为位数决定了能表示多大的数，也决定了占多少内存，这些都是实实在在的代价。

类型不是累赘，它是编译器能替你检查的东西。数字和文字不能相加：

<p class="wont">这段故意编译不过</p>

```rust,compile_fail,E0277
fn main() {
    let seats = 15;
    let total = seats + "3";
    println!("{total}");
}
```

```text,error
error[E0277]: cannot add `&str` to `{integer}`
```

在 JavaScript 里，`15 + "3"` 得到 `"153"`，程序继续跑，错误要到很久以后才显现。Rust 在运行之前就拦下了。这一类错误在 Rust 里根本到不了用户那里。

## 算术，以及它的边界

```rust
fn main() {
    let seated = 9;
    println!("{}", 15 - seated);
    println!("{}", 7 / 2);
    println!("{}", 7 % 2);
    println!("{}", 7.0 / 2.0);
}
```

```text,output
6
3
1
3.5
```

`{}` 是另一种写法，它会换成逗号后面算出来的值。注意 `7 / 2` 是 3：整数相除，结果还是整数，余数被丢掉。`%` 取余数。

既然整数的位数是固定的，算出的结果放不下时会怎样？`u8` 只有 8 位，最大是 255：

```rust,should_panic
fn add_one(n: u8) -> u8 {
    n + 1
}

fn main() {
    println!("{}", add_one(255));
}
```

```text,error
attempt to add with overflow
```

这段能编译，但运行时程序立刻崩溃了。这是你遇到的第一个**运行时错误**（runtime error），Rust 叫它 panic。

很多语言在这里不会停下，而是悄悄绕回 0。这种事造成过真实的灾难和安全漏洞。Rust 的调试版本选择大声喊出来，不让错误的数字继续往下走。

## 变量默认不能改

又来了一位客人：

<p class="wont">这段故意编译不过</p>

```rust,compile_fail,E0384
fn main() {
    let seated = 9;
    println!("现在 {seated} 人");
    seated = 10;
    println!("现在 {seated} 人");
}
```

```text,error
error[E0384]: cannot assign twice to immutable variable `seated`
```

慢慢读这条报错：`E0384` 是错误编号，后面说不能给不可改变的变量第二次赋值。

大多数语言里变量默认能改，Rust 反过来，这是深思熟虑的选择。程序里最难查的错误，往往来自“这个值是谁、什么时候改掉的”。一个值如果不会变，读代码的人就可以放心，看到它的定义就知道了它的一切。

确实要变的时候，用 `mut` 明确说出来：

```rust
fn main() {
    let mut seated = 9;
    println!("现在 {seated} 人");
    seated = seated + 1;
    println!("现在 {seated} 人");
}
```

```text,output
现在 9 人
现在 10 人
```

`=` 在这里不是“等于”，是“把右边算出的值放进左边”。`seated = seated + 1` 在数学上不成立，在程序里的意思是先算右边，再放回去。

`mut` 是一个信号：读代码的人看到它，就知道这里要留神。茶馆的代码里，绝大多数变量都不带 `mut`。

## 常量

有些值在程序的一生中都不变，而且整个程序都要用，比如座位上限。这样的值用 `const`，写在函数外面，名字用大写，类型必须写明：

```rust
const MAX_SEATS: u32 = 15;

fn main() {
    let seated = 9;
    println!("还有 {} 个空座", MAX_SEATS - seated);
}
```

```text,output
还有 6 个空座
```

想改上限时只改一处。程序里到处写 `15`，叫**魔法数字**（magic number），是维护者的噩梦：你不知道哪个 15 是座位数，哪个只是碰巧也是 15。

## 这一节学到的

- 内存里只有数字，类型决定怎么理解它们。
- `let` 定义变量，编译器能推断类型，也可以用 `: 类型` 写明。
- 整数相除丢掉余数；超出范围时程序崩溃，而不是悄悄算错。
- 变量默认不可变，要改写 `mut`；全局不变的值用 `const`。

## 练习

茶馆现在坐了 3 个人。定义常量 `MAX_SEATS`，先显示空座数量，又来 2 个人，再显示一次。

<details>
<summary>答案</summary>

```rust
const MAX_SEATS: u32 = 15;

fn main() {
    let mut seated = 3;
    println!("空座 {} 个", MAX_SEATS - seated);
    seated = seated + 2;
    println!("空座 {} 个", MAX_SEATS - seated);
}
```

```text,output
空座 12 个
空座 10 个
```

想一想：如果 `seated` 超过 15，`MAX_SEATS - seated` 会怎样？`u32` 不能是负数，程序会 panic。真实的茶馆是怎么防止这件事的？答案在第 1.5 节。

</details>
