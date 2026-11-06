# 所有权：Rust 最不一样的地方

这一部分写给有编程经验的读者。Python、JavaScript、Java、C 的经验都算数。这里不讲变量和循环，只讲 Rust 和它们不一样的地方。

## 小测验：这一部分要不要跳

下面的代码能不能编译？为什么？

```rust,ignore
fn main() {
    let names = vec![String::from("陈立")];
    let copy = names;
    println!("{}", names.len());
}
```

答得出“不能，`names` 已经被移走了”，这一节可以略读。下一节的小测验也答得出的话，请直接去第三部分。

## 一个值只有一个主人

在 Python 或 Java 里，`copy = names` 让两个名字指向同一个列表。谁负责释放它，由垃圾回收器在运行时判断。

Rust 没有垃圾回收器。它的规则是：**每个值只有一个主人，主人离开作用域时，值就被释放。** 赋值会把主人换掉：

<p class="wont">这段故意编译不过</p>

```rust,compile_fail,E0382
fn main() {
    let name = String::from("梧桐里");
    let other = name;
    println!("{name} {other}");
}
```

```text,error
error[E0382]: borrow of moved value: `name`
```

`let other = name;` 之后，`name` 不再拥有这段文字。这里编译器不是在挑剔：如果两个变量都算主人，这段内存就会被释放两次。C 程序里这是一类真实的安全漏洞，Rust 让它编译不过。

`i32` 这样的小类型例外，它们实现了 `Copy`，赋值时直接复制。

## 借用

大多数时候我们只想看一眼，不想接管。`&` 表示借用：

```rust
fn count(names: &Vec<String>) -> usize {
    names.len()
}

fn main() {
    let names = vec![String::from("陈立"), String::from("周小满")];
    println!("{} 人", count(&names));
    println!("{}", names[0]);
}
```

```text,output
2 人
陈立
```

`count` 只是借走，用完就还，`names` 还是 `main` 的。

## 读者很多，写者一个

要在借用时修改，写 `&mut`。Rust 的规则只有一条：**同一时刻，要么有任意多个只读借用，要么只有一个可写借用。**

<p class="wont">这段故意编译不过</p>

```rust,compile_fail,E0502
fn main() {
    let mut seats = vec![String::from("陈立")];
    let first = &seats[0];
    seats.push(String::from("周小满"));
    println!("{first}");
}
```

```text,error
error[E0502]: cannot borrow `seats` as mutable because it is also borrowed as immutable
```

这不是吹毛求疵。`push` 可能让 `Vec` 换一块更大的内存，这时 `first` 指向的旧地址就失效了。Java 在遍历时改列表会在运行时抛异常；Rust 在编译时就拦下来。

## String 和 &str

茶馆的代码里两者都很常见：`String` 自己拥有文字，`&str` 借用一段文字。函数参数一般写 `&str`，这样两种都能传进来：

```rust
fn greet(name: &str) -> String {
    format!("欢迎，{name}")
}

fn main() {
    let owned = String::from("陈立");
    println!("{}", greet(&owned));
    println!("{}", greet("周小满"));
}
```

```text,output
欢迎，陈立
欢迎，周小满
```

## 这一节学到的

- 每个值只有一个主人，赋值和传参默认会移走它。
- `&` 只读借用，`&mut` 可写借用，两者不能同时存在。
- 这些规则都在编译时检查，运行时没有额外开销。

## 练习

修好小测验里的代码，让它能打印出 `1`，而且 `copy` 和 `names` 都还能用。

<details>
<summary>答案</summary>

借用它，或者明确地复制一份：

```rust
fn main() {
    let names = vec![String::from("陈立")];
    let copy = names.clone();
    println!("{} {}", names.len(), copy.len());
}
```

```text,output
1 1
```

`.clone()` 真的复制一份数据。Rust 把这个开销写在明面上，不会替你悄悄做。

</details>
