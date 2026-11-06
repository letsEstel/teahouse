# 程序是怎么运行起来的

上一节说程序就是硬盘上的文件。这一节我们自己造一个这样的文件，看清楚从一段文字到一个能运行的程序，中间发生了什么。

## 两种语言

处理器只懂**机器码**，一串串数字，每个数字是一条极简单的指令。人没法直接写它。所以我们用人能读的文字写**源代码**，再把它翻译成机器码。

翻译有两种做法。Python 和 JavaScript 用**解释器**：程序运行时一边读一边执行。Rust 和 C 用**编译器**：运行之前先把整个程序翻译好，得到一个独立的文件。

编译多花一步时间，换来两样东西。一是速度，运行时不用再翻译。二是编译器有机会在程序运行之前读完全部代码，把一大类错误拦下来。Rust 把第二点用到了极致，这是我们选它写安全系统的原因。

## 安装 Rust

Rust 项目提供了一个安装工具 `rustup`。打开 <https://rustup.rs>，把页面上的那一条命令复制到终端运行，遇到提问时按回车选默认项。

装好以后，**关掉终端再开一个新的**，然后检查：

```sh
rustc --version
cargo --version
```

各显示一行版本号就成功了。`rustc` 是编译器。`cargo` 是 Rust 的项目管家，创建项目、下载依赖、调用编译器、运行测试都归它管，平时你只和它打交道。

## 创建项目

```sh
cd ~/teahouse-book
cargo new seats
cd seats
ls -a
```

```text
.  ..  .git  .gitignore  Cargo.toml  src
```

`cargo` 不仅建好了文件，还顺手建了一个 git 仓库。`Cargo.toml` 是项目的说明：

```toml
[package]
name = "seats"
version = "0.1.0"
edition = "2024"

[dependencies]
```

`[dependencies]` 下面是空的，意思是这个程序不依赖别人的代码。Relay 的这一段也几乎是空的，第三部分会看到这是有意为之。

## 读懂第一个程序

打开 `src/main.rs`：

```rust
fn main() {
    println!("Hello, world!");
}
```

```text,output
Hello, world!
```

- `fn` 是 function 的缩写，`fn main()` 定义一段名叫 `main` 的指令。操作系统启动程序时，从 `main` 开始执行。
- 花括号 `{` `}` 框住这段指令的内容。
- `println!` 显示一行字。名字后面的感叹号说明它是**宏**，一种在编译时生成代码的东西，现在把它当作一条指令就好。
- 引号里的文字原样显示，叫**字符串**。
- 分号 `;` 结束一条语句，就像句号结束一句话。

每一行的缩进是给人看的，编译器不在乎，但 Rust 程序员都用四个空格。

## 编译和运行是两步

`cargo run` 把两步合在一起了。我们拆开看：

```sh
cargo build
ls target/debug/
```

```text
build  deps  examples  incremental  seats  seats.d
```

`target/debug/seats` 就是编译出来的程序，和 `/usr/bin/ls` 是同一种东西。直接运行它：

```sh
./target/debug/seats
```

```text
Hello, world!
```

开头的 `./` 表示“当前目录下的这个文件”。shell 默认只在几个固定的目录里找程序，当前目录不在其中，这是一项防止你误运行恶意文件的安全设计。

这个文件不需要 Rust 就能运行。把它复制到另一台同样系统的电脑上，一样能跑。部署 Relay 时做的就是这件事：编译出一个文件，传到服务器，运行。

看看它的退出状态：

```sh
./target/debug/seats
echo $?
```

```text
Hello, world!
0
```

`main` 正常结束，退出状态就是 0。

## 第一次报错

现在故意写错。删掉第一条语句末尾的分号，再加一行：

<p class="wont">这段故意编译不过</p>

```rust,compile_fail
fn main() {
    println!("欢迎来到茶馆")
    println!("请坐");
}
```

```text,error
error: expected `;`, found `println`
```

真实的报错下面还会画出位置，甚至告诉你 `help: add ; here`。

学编程的人常把报错当成批评。它其实是编译器在帮你读代码。请养成习惯：**先读第一条报错，只看第一条**。后面的报错常常是第一条引起的，改好第一条，它们就消失了。

注意，这个程序没有运行，一行都没有。错误在翻译时就被发现了，用户永远遇不到。这叫**编译时错误**。另一种在运行时才出现的错误，后面会遇到，它们要麻烦得多。

## 指令按顺序执行

```rust
fn main() {
    println!("欢迎来到茶馆");
    println!("请坐");
}
```

```text,output
欢迎来到茶馆
请坐
```

程序从 `main` 的第一行开始，一条接一条往下走。到现在为止这就是全部的规则，接下来几节要做的，就是让这条直线变得有用：记住东西，做判断，重复。

## 这一节学到的

- 编译器把源代码翻译成机器码，得到一个独立的程序文件。
- `cargo new` 建项目，`cargo build` 编译，`cargo run` 编译并运行。
- 程序从 `main` 开始，语句按顺序执行。
- 编译时错误在程序运行前就被拦下来；读报错时先看第一条。

## 练习

1. 让程序显示三行：你的名字、你想开的茶馆的名字、`欢迎`。
2. 故意把 `println!` 写成 `printn!`，运行 `cargo build`，读懂报错。编译器有没有猜到你想写什么？

<details>
<summary>答案</summary>

```rust
fn main() {
    println!("王阿姨");
    println!("梧桐里三号院");
    println!("欢迎");
}
```

```text,output
王阿姨
梧桐里三号院
欢迎
```

第二题的报错是 ``cannot find macro `printn` ``，下面还会建议一个相近的名字 `println`。编译器确实在猜你的意思。

</details>
