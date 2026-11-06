# 所有权：没有垃圾回收，也没有内存错误

这一部分写给有编程经验的读者。变量、循环、函数你都会，这里只讲 Rust 真正不同的地方。这些不同不是语法上的怪癖，它们是一个明确目标的结果：**不牺牲性能，在编译时消灭一整类错误**。

## 小测验：这一节要不要跳

下面的代码能不能编译？为什么？

```rust,ignore
fn main() {
    let names = vec![String::from("陈立")];
    let copy = names;
    println!("{}", names.len());
}
```

如果你能立刻答出“不能，`names` 的所有权已经移给 `copy`”，并且说得出为什么不能像 Java 那样两边都指向同一个列表，这一节可以略读。

## 为什么需要它

程序用完内存要还回去。几十年来，语言设计者只有两种办法。

**手动管理**，比如 C 和 C++：程序员自己 `malloc` 和 `free`。这样最快，但人总会犯错：用了已经释放的内存，释放了两次，忘记释放，写过了边界。这些错误不只是崩溃，它们是攻击者最常利用的漏洞。微软和 Chrome 团队都公布过相近的数字：他们的严重安全漏洞里，大约七成是内存安全问题。

**垃圾回收**，比如 Java、Go、Python：运行时有一个程序不断检查哪些内存没人用了。这样安全，但要付出运行时开销，程序的暂停时间也不好预测。

Rust 走了第三条路：**让编译器在编译时算清楚每块内存什么时候释放**。运行时什么也不用做，错误的程序根本编译不过。

代价是你要接受一套规则，这一节就讲这套规则。它们的严格，是把别的语言留到运行时的问题提前到了你眼前。

## 栈和堆

要理解规则，先要知道数据放在哪里。

**栈**上的数据大小固定，函数返回时自动消失，非常快。`u32`、`bool` 都放在这里。

**堆**上的数据大小可变，能活得比创建它的函数更久，由程序显式申请。`String` 的文字放在堆上，栈上只有三个数：指向堆的指针、长度、容量：

```text
    栈                    堆
  name ┌─────────┐      ┌──────────────┐
       │ 指针 ───┼─────▶│ 梧 桐 里     │
       │ 长度 9  │      └──────────────┘
       │ 容量 9  │
       └─────────┘
```

长度是 9，因为 UTF-8 里一个汉字占三个字节。问题来了：这块堆内存谁负责释放？

## 三条规则

1. 每个值有且只有一个**所有者**。
2. 所有者离开作用域时，值被释放。
3. 所有权可以转移，转移之后旧名字不能再用。

第二条我们可以亲眼看见。给类型实现 `Drop`，它被释放时就会运行我们的代码：

```rust
struct Seat {
    name: &'static str,
}

impl Drop for Seat {
    fn drop(&mut self) {
        println!("{} 离座", self.name);
    }
}

fn main() {
    let host = Seat { name: "陈立" };
    {
        let guest = Seat { name: "王阿姨" };
        println!("里面：{}", guest.name);
    }
    println!("外面：{}", host.name);
}
```

```text,output
里面：王阿姨
王阿姨 离座
外面：陈立
陈立 离座
```

离开花括号的那一刻，释放自动发生。这个时刻是确定的，没有垃圾回收器，编译器在那里插入了一次调用。这个办法来自 C++ 的 RAII，文件、锁、网络连接都可以这样管理：离开作用域就关闭，不可能忘。

## 移动

如果 `let other = name;` 只是复制栈上那三个数，就有两个指针指向同一块堆内存。两个所有者离开作用域时都要释放，这就是著名的**重复释放**。

Rust 的解决办法很简单：赋值把所有权**移动**过去，旧名字作废。

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

C 里这段代码能编译，然后在运行时悄悄出错。这里它进不了可执行文件。

传参也是移动：把 `String` 传给函数，就把所有权交了出去。

如果真要一份独立的数据，用 `.clone()`。它真的复制堆内存，开销写在明面上。而 `u32` 这样完全存在栈上的小类型实现了 `Copy`，赋值就是复制，没有所有权的问题。

## 借用

每次都交出所有权太不方便。大多数时候我们只想用一下。`&` 创建一个**引用**，这叫**借用**：

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

引用就是一个指针，运行时没有任何额外开销。但编译器要保证它永远有效，于是有了 Rust 的核心规则：

> 在任意时刻，要么有任意多个只读引用（`&T`），要么有且只有一个可写引用（`&mut T`）。

这条规则的道理是：**共享和修改，二者不可同时存在**。许多人同时读没有问题，一个人独自改也没有问题，一边读一边改才会出事。

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

这不是吹毛求疵。`push` 可能发现容量不够，于是申请一块更大的内存，把数据搬过去，释放旧的，这时 `first` 就指向了已释放的内存。这种错误在 C++ 里真实存在，叫**迭代器失效**。Java 用运行时异常来防，Rust 直接编译不过。

同一条规则还带来一个意外的收获：既然同一时刻只能有一个人写，**多线程之间的数据竞争**也在编译时消失了。

## 生命周期

引用不能比它指向的数据活得更久：

<p class="wont">这段故意编译不过</p>

```rust,compile_fail,E0597
fn main() {
    let longest;
    {
        let name = String::from("陈立");
        longest = &name;
    }
    println!("{longest}");
}
```

```text,error
error[E0597]: `name` does not live long enough
```

编译器追踪每个引用的有效范围，这个范围叫**生命周期**。多数时候它能自己推断。函数同时接收和返回引用时，有时要用 `'a` 这样的标注说清楚“返回的引用来自哪个参数”。

第一部分见过的 `&'static str` 就是这个意思：`'static` 表示这段文字在程序的整个运行期间都有效，写在代码里的字符串都是如此。

茶馆的代码里有个好例子：

```rust,ignore
// core/src/rules.rs
pub enum RefKind<'a> {
    Main,
    /// `refs/proposals/<owner>/<name>`
    Proposal {
        owner: &'a str,
        name: &'a str,
    },
    Other,
}
```

解析一个分支名时，`owner` 和 `name` 不复制文字，只是借用原字符串中的一段。`'a` 告诉编译器：这个 `RefKind` 不能比原字符串活得久。这样解析不用申请内存，又绝对安全。

## String 和 &str

茶馆里到处都是这两个类型。`String` 拥有堆上的文字，可以增长。`&str` 是对一段文字的借用，可以指向 `String` 的一部分，也可以指向写在程序里的字面量。

写函数的惯例：参数用 `&str`，这样两种都能传；要长期保存时才用 `String`。

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

- Rust 用编译时的所有权检查，代替手动管理和垃圾回收。
- 每个值一个所有者，离开作用域时确定地释放。
- 赋值和传参默认移动；`Copy` 类型除外；`.clone()` 显式复制。
- 共享和修改不能同时存在，这同时消灭了悬垂指针和数据竞争。
- 生命周期保证引用不会活过它指向的数据。

## 练习

修好小测验里的代码，让 `names` 和 `copy` 都能用，然后想一想：`clone` 和借用，这里哪个更好？

<details>
<summary>答案</summary>

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

如果 `copy` 只是用来读，写 `let copy = &names;` 更好，不用复制数据。`clone` 适合两边要各自修改的时候。

</details>
