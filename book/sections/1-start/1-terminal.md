# 和电脑说话

平时你用鼠标点图标，让电脑做事。程序员多用另一种方式：打字告诉电脑做什么。打字的地方叫**终端**。

这一节，我们打开终端，学会四条命令。

## 打开终端

- **Windows**：按 Win 键，输入 `PowerShell`，回车。
- **macOS**：按 Command + 空格，输入 `Terminal`，回车。
- **Linux**：按 Ctrl + Alt + T。

你会看到一个几乎空白的窗口，里面有一个闪烁的光标。它在等你输入。

## 我在哪里

电脑里的文件放在一层层的**文件夹**里，就像柜子里的抽屉。终端在任何时刻都站在某一个文件夹里。

输入下面这条命令，按回车：

```sh
pwd
```

`pwd` 的意思是 print working directory，显示当前所在的文件夹。你会看到类似这样的结果：

```text
/home/wang
```

这一串叫**路径**，表示 `home` 文件夹里的 `wang` 文件夹。在 Windows 上它看起来像 `C:\Users\wang`，意思是一样的。

## 这里有什么

```sh
ls
```

`ls` 是 list 的缩写，列出当前文件夹里的东西。你会看到 `Documents`、`Downloads` 这样的名字，和你在文件管理器里看到的是同一批文件夹。

## 新建一个文件夹，走进去

我们给这本书的练习单独建一个文件夹：

```sh
mkdir teahouse-book
```

`mkdir` 是 make directory，新建文件夹。这条命令成功时什么也不显示。没有消息就是好消息，这是终端的习惯。

再运行一次 `ls`，你会看到多了一个 `teahouse-book`。现在走进去：

```sh
cd teahouse-book
```

`cd` 是 change directory，换一个文件夹。运行 `pwd` 确认一下：

```text
/home/wang/teahouse-book
```

想回到上一层，用 `cd ..`。两个点表示上一层。

## 这一节学到的

| 命令 | 做什么 |
|---|---|
| `pwd` | 显示我在哪里 |
| `ls` | 列出这里有什么 |
| `mkdir 名字` | 新建文件夹 |
| `cd 名字` | 走进文件夹；`cd ..` 回到上一层 |

## 练习

在 `teahouse-book` 里新建一个叫 `test` 的文件夹，走进去，用 `pwd` 确认位置，再回到 `teahouse-book`。

<details>
<summary>答案</summary>

```sh
mkdir test
cd test
pwd
cd ..
```

最后一次 `pwd` 应该显示 `teahouse-book`。

</details>
