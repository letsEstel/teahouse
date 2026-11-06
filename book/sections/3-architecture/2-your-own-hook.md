# 亲手造一道门

上一节读了别人造的门，这一节自己造一道。用不到十五行 shell，你会得到和 Relay 核心机制完全相同的东西。这种设计的美在于机制极小，力量全部来自 Unix 和 git 已有的约定。

## hook 是什么

**hook** 是软件留给你的一个插口：到了某个时刻，我来运行你指定的程序，并且听从它的结果。

git 的 hook 就是仓库 `hooks/` 目录里、名字固定的可执行文件。整个约定只有三条：

1. git 在固定时刻运行这个文件。
2. 通过标准输入传数据：每个要更新的分支一行，`旧哈希 新哈希 分支名`。
3. 读取它的退出状态：0 放行，其他否决。

看，第 1.1 节的退出状态和标准输入，全用上了。这个程序可以用任何语言写，git 根本不关心。茶馆用 Rust，这里用 shell，协议完全一样。

## 准备一个“服务器”

服务器上的仓库不需要工作文件，这种仓库叫**裸仓库**（bare repository）。Relay 的 `houses/<id>.git` 就是这样的目录。在一个空目录里运行：

```sh
git init --bare -b main house.git
```

## 写 hook

创建文件 `house.git/hooks/pre-receive`：

```sh
#!/bin/sh
# 每个被推送的提交都必须写明 Problem:
while read old new ref; do
  for commit in $(git rev-list "$new" --not --all); do
    if ! git log -1 --format=%B "$commit" | grep -q '^Problem: '; then
      echo "拒绝：提交 $commit 没有写 Problem:" >&2
      exit 1
    fi
  done
done
```

逐行读：

- `#!/bin/sh` 告诉操作系统用哪个程序解释这个文件。
- `while read old new ref` 从标准输入一行行读，拆成三个变量。
- `git rev-list "$new" --not --all` 列出 `new` 能到达、现有分支都到达不了的提交，也就是这次带来的新提交。隔离区里的提交，hook 看得见。
- `git log -1 --format=%B` 取出提交说明，交给 `grep` 查找。
- `>&2` 写到标准错误，git 会把它转给推送的人。
- `exit 1` 否决。

让它可以执行。这一步不能少：文件不可执行时，git 会**跳过**这个 hook，只给一句提示，推送照样成功。

```sh
chmod +x house.git/hooks/pre-receive
```

想一想这个默认值：hook 没配好时一切放行，这是失败即打开，方向是错的。所以 Relay 把 hook 放在一个由它管理的目录里，`teahouse-relay check` 会核对每间茶馆的 `core.hooksPath` 有没有指向那里。

## 试一试

```sh
git clone house.git mine
cd mine
echo 42 > parking.txt
git add .
git commit -m "加上车位数"
git push origin main
```

```text
remote: 拒绝：提交 274ce3a811212e56caae96f2b7e606118fd5e54b 没有写 Problem:
 ! [remote rejected] main -> main (pre-receive hook declined)
error: failed to push some refs to '…/house.git'
```

（哈希会不同。）`remote:` 开头的那一行就是你的 `echo`，它穿过 git 回到了推送者的屏幕。

把提交说明补上，再推送：

```sh
git commit --amend -m "加上车位数

Problem: 0001"
git push origin main
```

这次成功了。你实现了茶馆法则的第 7 条的一部分：每个提交都面向一个问题。

## 这个 hook 哪里不够好

写出能运行的程序很容易，这个项目真正的功夫在下面这张表：

| 问题 | 这个 hook | 真实的 Relay |
|---|---|---|
| 谁在推送？ | 不知道 | sshd 验证钥匙，环境变量传给 hook |
| `git rev-list` 自己失败了？ | 循环不执行，**放行** | 任何错误都拒绝 |
| 没有经过 Relay 直接触发 hook？ | 放行 | 缺环境变量就拒绝 |
| 删除分支时 `new` 全是 0？ | 命令报错，放行 | 先判断操作类型 |
| 提交说明写两行 `Problem:`？ | 放行 | 恰好一行，格式也要对 |
| 提交很多，拖垮服务器？ | 无上限 | 单次最多 5000 个提交 |

注意第二行。shell 里 `for` 取到空列表时什么也不做，脚本最后退出状态是 0。出错和“没问题”看起来一模一样，这就是失败即打开。

这也是 Relay 用 Rust 写的原因。`Result` 让错误不可能被无声地吞掉，`?` 让每次失败都向上传到 hook 的退出状态：

```rust,ignore
// relay/src/error.rs
/// The relay's single error type: a human-readable message.
/// Errors are reported, never swallowed; the process exits non-zero.
```

这句注释就是第二部分那些语言特性落到实处的样子。

## 这一招到处能用

你学到的不只是 git。这个模式的结构是：**一个成熟系统留出的拦截点，加上一个判决程序，判决由退出状态表达。**

- `command=` 强制命令：限制一把 ssh 钥匙能做什么。
- `sudoers` 里的命令白名单：限制一个用户能以 root 运行什么。
- systemd 的 `ExecStartPre`：服务启动前先运行检查，失败就不启动。
- Kubernetes 的准入控制器：资源写入前交给你判断。

以后你面对一个新需求，先问：这件事有没有现成的、绕不过去的关口？能不能插一个小程序上去，而不是自己造一个系统？这是这本书最想教给你的设计习惯。

## 这一节学到的

- hook 的协议只有三条：固定时刻运行、标准输入传数据、退出状态定结果。
- 裸仓库是服务器上的仓库；hook 文件必须可执行，否则被跳过。
- shell 里错误很容易被吞掉，安全判决要失败即关闭。
- 找现成的关口，插一个小判决程序，这是一种通用的设计。

## 练习

1. 改写 hook，只有 `Problem:` 后面是四位数字时才放行。
2. 修掉上表第二行的问题：`git rev-list` 失败时必须拒绝推送。提示：先把结果存进变量，再判断退出状态。

<details>
<summary>答案</summary>

第一题，把 `grep` 的条件换成：

```sh
grep -Eq '^Problem: [0-9]{4}$'
```

第二题：

```sh
#!/bin/sh
while read old new ref; do
  commits=$(git rev-list "$new" --not --all) || {
    echo "拒绝：读不出这次的提交" >&2
    exit 1
  }
  for commit in $commits; do
    if ! git log -1 --format=%B "$commit" | grep -Eq '^Problem: [0-9]{4}$'; then
      echo "拒绝：提交 $commit 没有写 Problem:" >&2
      exit 1
    fi
  done
done
```

`||` 在 shell 里的意思是：左边失败了才执行右边，这和第 1.4 节 Rust 的短路是同一个想法。

</details>
