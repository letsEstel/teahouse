# 自己写一个 hook

上一节的 Relay 看起来复杂，核心机制却只有一个：git 在接受推送之前，会运行你指定的程序。这一节我们用十行 shell 亲手做一个。

## 准备一个“服务器”

服务器上的仓库不需要工作文件，这种仓库叫**裸仓库**。在任意空文件夹里运行：

```sh
git init --bare -b main house.git
```

## 写 hook

hook 就是放在 `hooks/` 文件夹里、名字固定的可执行文件。创建 `house.git/hooks/pre-receive`：

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

逐行看：

- git 从标准输入给 hook 传数据，每个被推送的分支一行：旧哈希、新哈希、分支名。
- `git rev-list "$new" --not --all` 列出这次带来的新提交。
- `%B` 取出提交说明，`grep -q` 查找，`!` 取反。
- 写到标准错误的文字（`>&2`）会显示给推送的人。
- `exit 1` 让 git 放弃这次推送。

然后让它可以执行：

```sh
chmod +x house.git/hooks/pre-receive
```

## 试一试

```sh
git clone house.git mine
cd mine
echo 42 > parking.txt
git add .
git commit -m "加上车位数"
git push origin main
```

结果（哈希和路径会不同）：

```text
remote: 拒绝：提交 274ce3a811212e56caae96f2b7e606118fd5e54b 没有写 Problem:
 ! [remote rejected] main -> main (pre-receive hook declined)
error: failed to push some refs to
```

把提交说明补上，再推送：

```sh
git commit --amend -m "加上车位数

Problem: 0001"
git push origin main
```

这次成功了。你刚刚实现了茶馆法则的第 7 条：每个提交都面向一个问题。

## 这和 Relay 差在哪里

| 这个 hook | 真实的 Relay |
|---|---|
| shell，十行 | Rust，有完整的测试 |
| 谁都能推送 | sshd 强制命令，先查座位 |
| 只查一条规则 | 法则一条条检查，见 `core/src/rules.rs` |
| 输入出错时可能放行 | 失败即关闭：没有明确通过的都拒绝 |

最后一行最要紧。写安全相关的 hook，要把“看不懂”当作“拒绝”。

## 练习

改写这个 hook，只有 `Problem:` 后面是四位数字时才放行。

<details>
<summary>答案</summary>

把 `grep` 的条件换成：

```sh
grep -Eq '^Problem: [0-9]{4}$'
```

`-E` 打开扩展正则，`[0-9]{4}` 表示四位数字，`$` 表示行尾。

</details>
