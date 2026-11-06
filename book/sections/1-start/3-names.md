# 给东西起名字

程序要记住一些东西：茶馆叫什么，坐了几个人。这一节，我们给这些东西起名字。

## let

把 `src/main.rs` 改成这样，然后 `cargo run`：

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

`let seats = 15;` 的意思是：把 15 这个值记下来，起名叫 `seats`。这样的名字叫**变量**。

`println!` 的引号里，`{seats}` 会换成变量的值。

这里有两种不同的值。`15` 是数字，`"梧桐里三号院"` 是文字，文字要用双引号括起来。

## 用变量算数

数字可以用 `+`、`-`、`*`、`/` 计算，`*` 是乘，`/` 是除：

```rust
fn main() {
    let seats = 15;
    let seated = 9;
    let free = seats - seated;
    println!("还有 {free} 个空座");
}
```

```text,output
还有 6 个空座
```

## 变量默认不能改

又来了一位客人，我们想把 `seated` 改成 10：

<p class="wont">这段故意编译不过</p>

```rust,compile_fail,E0384
fn main() {
    let seated = 9;
    println!("现在 {seated} 人");
    seated = 10;
    println!("现在 {seated} 人");
}
```

运行 `cargo run`，编译器拒绝了：

```text,error
error[E0384]: cannot assign twice to immutable variable `seated`
```

这是你遇到的第一条报错。请慢慢读它：

- `error` 说明这是错误，`E0384` 是这种错误的编号。
- 后面这句话的意思是：不能给不可改变的变量 `seated` 第二次赋值。
- 真实的报错下面还会画出是哪一行，并且给出建议。

Rust 的变量默认不能改。这是有意的设计：一个值不会变，读程序的人要操心的事就少了一件。

## mut

确实要改的时候，用 `mut` 说明这个变量会变：

```rust
fn main() {
    let mut seated = 9;
    println!("现在 {seated} 人");
    seated = 10;
    println!("现在 {seated} 人");
}
```

```text,output
现在 9 人
现在 10 人
```

`=` 在这里的意思不是“等于”，而是“把右边的值放进左边的变量”。

## 这一节学到的

- `let 名字 = 值;` 定义一个变量。
- 变量默认不能改，要改的话写 `let mut`。
- 编译器的报错是在帮你，先读它说了什么。

## 练习

茶馆现在坐了 3 个人。写一个程序，先显示空座数量，然后又来 2 个人，再显示一次空座数量。

<details>
<summary>答案</summary>

```rust
fn main() {
    let seats = 15;
    let mut seated = 3;
    println!("空座 {} 个", seats - seated);
    seated = seated + 2;
    println!("空座 {} 个", seats - seated);
}
```

```text,output
空座 12 个
空座 10 个
```

`seated = seated + 2;` 先算出右边的 5，再放回 `seated`。`{}` 是另一种写法：它会换成逗号后面算出来的值。

</details>
