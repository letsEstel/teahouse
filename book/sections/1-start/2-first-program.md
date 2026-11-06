# 第一个程序

这一节，我们装好 Rust，让电脑显示一句话。

## 程序是什么

程序是写给电脑的一串指令。我们用人能读的文字写指令，这种文字叫**源代码**。电脑不能直接执行源代码，要先由另一个程序把它翻译一遍，这个翻译程序叫**编译器**。Rust 的编译器叫 `rustc`。

## 安装 Rust

打开 <https://rustup.rs>，按网页上的说明安装。

- **macOS 和 Linux**：网页上给出一条命令，复制到终端运行，遇到提问时按回车选默认项。
- **Windows**：下载 `rustup-init.exe` 并运行，它可能会要求先安装微软的 C++ 工具，按提示操作。

装好以后，**关掉终端再打开一个新的**，然后运行：

```sh
cargo --version
```

显示一行版本号就成功了。`cargo` 是 Rust 自带的管家，负责创建项目、调用编译器、运行程序。

## 创建项目

先走进上一节建的文件夹，再让 `cargo` 创建一个项目：

```sh
cd teahouse-book
cargo new seats
cd seats
```

`cargo new seats` 建了一个叫 `seats` 的文件夹，里面有：

```text
Cargo.toml      项目的说明
src/main.rs     源代码
```

用任意文本编辑器打开 `src/main.rs`。你会看到 `cargo` 已经写好了一个程序。

## 读懂这三行

```rust
fn main() {
    println!("Hello, world!");
}
```

```text,output
Hello, world!
```

一行一行看：

- `fn main()` 定义一段名叫 `main` 的指令。程序运行时，总是从 `main` 开始。
- `{` 和 `}` 之间是这段指令的内容。
- `println!("...")` 让电脑显示一行字。引号里的内容原样显示。
- 每条指令以分号 `;` 结束，就像一句话以句号结束。

## 运行它

在 `seats` 文件夹里运行：

```sh
cargo run
```

`cargo` 先调用编译器，再运行翻译好的程序。最后一行就是程序的输出。

## 改一个字

把程序改成这样，保存，再运行一次 `cargo run`：

```rust
fn main() {
    println!("欢迎来到茶馆");
}
```

```text,output
欢迎来到茶馆
```

恭喜，你改了一个程序。

## 这一节学到的

- 源代码是写给人读的指令，编译器把它翻译给电脑。
- 程序从 `main` 开始执行。
- `println!` 显示一行字，`cargo run` 编译并运行。

## 练习

让程序一共显示两行：第一行是你的名字，第二行是 `我要开一间茶馆`。

<details>
<summary>答案</summary>

指令从上到下依次执行。把名字换成你自己的：

```rust
fn main() {
    println!("王阿姨");
    println!("我要开一间茶馆");
}
```

```text,output
王阿姨
我要开一间茶馆
```

</details>
