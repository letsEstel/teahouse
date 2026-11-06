# 一次推送走过的路

这一部分写给要维护茶馆的人。这一节跟着一次被拒绝的推送走完全程，读完你就知道每段代码在哪里、为什么在那里。

## 小测验

茶馆的 Relay 是一个常驻的服务吗？它监听哪个端口？

答案是：不是，它什么端口都不监听。如果这让你意外，请往下读。

## 先让它发生一次

在仓库根目录运行 `./scripts/demo.sh`。第 4 步里，伙计想把自己改成老板：

```text
▌ 4. 伙计想给自己升成老板
  ✗ 被拒绝（符合预期）· refused, as it should be
    ✗ refs/heads/main: commit 2e697e4356: the steward may only change problems/, works/, board/ and talk/, not .teahouse/members.toml / 伙计只能修改 problems/、works/、board/、talk/
```

这个伙计用的是普通的 `git push`，走的是这条路：

```text
git push
  │  ssh
  ▼
sshd                    OpenSSH 验证钥匙
  │  强制命令
  ▼
teahouse-relay shell    这把钥匙有座位吗？
  │  exec
  ▼
git receive-pack        git 收下提交，暂不改动 main
  │  pre-receive
  ▼
teahouse-relay hook     按座位表判决，全过才放行
```

守门的不是常驻服务，而是一连串短命的进程。sshd 本身就是那个常驻服务，Relay 只在有人连接时运行，这样就没有新端口要保护，也没有内存泄漏要操心。

## 第一道门：OpenSSH

我们不写自己的认证。sshd 配置里的 `AuthorizedKeysCommand` 让 sshd 问 Relay：这把钥匙该怎么处理？Relay 回答的那一行里带着强制命令：

```rust,ignore
// relay/src/admin.rs
                "restrict,command=\"{binary} --root {root_s} shell {}\" {} teahouse",
```

`command=` 是 OpenSSH 的功能：不管客户端要运行什么，都只运行这条命令，客户端原本想运行的放进环境变量 `SSH_ORIGINAL_COMMAND`。`restrict` 关掉端口转发等其他能力。

## 第二道门：shell

`relay/src/shell.rs` 检查请求是不是 git 读写，这把钥匙在这间茶馆有没有座位。通过以后，它把身份放进环境变量，然后用 `exec` 把自己**替换**成 git：

```rust,ignore
// relay/src/shell.rs
    cmd.env("TEAHOUSE_KEY", key.blob_b64());
    cmd.env("TEAHOUSE_HOUSE", &id);
// …
    let err = cmd.exec();
```

`exec` 是 Unix 的系统调用：当前进程变成另一个程序，不会返回。所以如果下面的代码还在运行，那一定是出错了。

## 第三道门：pre-receive

git 收下提交后、移动 `main` 之前，会运行 `pre-receive` hook。这时新提交还在隔离区，hook 退出码不为 0 的话，整次推送作废。

```rust,ignore
// relay/src/hook.rs
    let members = members_at(&git, MAIN_REF)?;
    let Some(pusher) = members.by_key(&key) else {
```

这两行是整个安全模型的核心。`MAIN_REF` 此刻还是推送之前的 `main`，所以权限来自**现有的**座位表，而不是这次推送带来的那张。伙计在提交里把自己改成老板，也没有用：判决他的是旧表。

## 判决

`check_commit` 逐个提交、逐个路径检查。伙计撞上的是这一行：

```rust,ignore
// relay/src/hook.rs
        if on_main && role != Role::Guest && !may_write_on_main(role, p) {
```

真正的规则是一个纯函数，没有任何输入输出，所以很好测试：

```rust,ignore
// core/src/rules.rs
pub fn may_write_on_main(role: Role, path: &str) -> bool {
    match role {
        Role::Host => true,
        Role::Steward => STEWARD_DIRS
            .iter()
            .any(|d| path.starts_with(d) && path.len() > d.len()),
        Role::Guest => false,
    }
}
```

这就是茶馆的分层：`core/` 放纯规则，零依赖；`relay/` 负责把 git 里的事实喂给它。

## 这一节学到的

- Relay 没有常驻进程，靠 sshd 的强制命令和 git 的 hook 接在一起。
- 权限永远来自推送之前的 `main`。
- 规则是 `core/` 里的纯函数，I/O 在 `relay/`。

## 练习

在 `relay/src/hook.rs` 里找到 `pre_receive`，回答：如果一次推送里有一个分支违规、一个分支合法，合法的那个会进去吗？

<details>
<summary>答案</summary>

不会。所有分支的问题都收集进 `problems`，只要它不是空的，`pre_receive` 就返回错误，hook 以非零状态退出，git 放弃整次推送。这是法则的“全有或全无”。

</details>
