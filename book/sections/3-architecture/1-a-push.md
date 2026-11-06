# 一次推送走过的路

全景里画过一张图。这一节拿放大镜，跟着一次被拒绝的推送走完 Relay 的全程。读完以后，你就知道这七千行代码是怎么组织起来的，以及为什么非这样组织不可。

## 小测验

Relay 是一个常驻的服务吗？它监听哪个端口？

答案是：它不是常驻服务，它不监听任何端口。如果这让你意外，这一节正是为你写的。

## 把系统想成一座有三道门的房子

设计安全系统，第一个问题是：**检查放在哪里？** 只有一个答案是对的，检查必须放在一个绕不过去的地方。这叫**完全仲裁**（complete mediation）。

如果 Relay 是一个自己写的网络服务，我们就要自己解析网络数据、处理认证、防御畸形输入，这些地方每一处都可能出错。茶馆反过来想：有没有已经存在、已经绕不过去的关口，让我们把检查插进去？

有，而且有两个。

- OpenSSH 接受连接时，可以指定**只能运行某一条命令**。
- git 接受推送时，会先运行 **hook**，hook 可以否决。

Relay 就是插在这两个关口上的一个普通命令行程序。每次有人连接，它启动，判决，退出。真正常驻的是 sshd，没有新的端口要防守，也没有长期运行的进程会积累状态或者泄漏内存。

```text
git push
  │  ssh
  ▼
sshd                    第一道门：你真的持有这把钥匙吗？
  │  强制命令
  ▼
teahouse-relay shell    第二道门：这把钥匙在这间茶馆有座位吗？
  │  exec
  ▼
git receive-pack        收下提交，放进隔离区，main 不动
  │  pre-receive
  ▼
teahouse-relay hook     第三道门：这次改动符合法则吗？
  │  退出状态 0
  ▼
main 前进一步
```

## 先让它发生一次

在仓库根目录运行 `./scripts/demo.sh`。这个脚本在你的电脑上开一个临时茶馆，用的是真的 git、真的钥匙、真的签名，只是把 sshd 换成了本地转接。第 4 步，伙计把座位表里自己改成老板，然后推送：

```text
▌ 4. 伙计想给自己升成老板
  ✗ 被拒绝（符合预期）· refused, as it should be
    ✗ refs/heads/main: commit 2e697e4356: the steward may only change problems/, works/, board/ and talk/, not .teahouse/members.toml / 伙计只能修改 problems/、works/、board/、talk/
```

现在一道门一道门地看。

## 第一道门：OpenSSH

ssh 的认证本身很优雅：服务器不需要知道你的私钥，它发一个随机挑战，你用私钥签名，它用公钥验证。私钥从不离开设备，服务器被攻破也偷不到它。

OpenSSH 有一个文件 `authorized_keys`，一行一把允许登录的公钥。这一行前面可以加选项。Relay 给出的每一行都是这样：

```rust,ignore
// relay/src/admin.rs
                "restrict,command=\"{binary} --root {root_s} shell {}\" {} teahouse",
```

`command=` 叫**强制命令**（forced command）：不管客户端想运行什么，sshd 只运行这一条，客户端原本的请求放进环境变量 `SSH_ORIGINAL_COMMAND`。`restrict` 关掉端口转发、终端等一切其他能力。

所以，即使你拿着有效的钥匙，也得不到服务器上的 shell。`{}` 处填的是这把公钥，于是程序启动时就知道“是谁在敲门”。这个身份由 sshd 验证过，不是客户端自己说的。

这里有个细节。座位表随时会变，每次都重写文件容易出错。sshd 还支持 `AuthorizedKeysCommand`：连接时现问一个程序。配置文件里只有一句话：

```text
AuthorizedKeysCommand /usr/lib/teahouse/teahouse-relay --root /var/lib/teahouse authorized-keys --key %k
```

从此认证的数据来自座位表，只有一个事实来源。

## 第二道门：shell

`relay/src/shell.rs` 要处理的是一段来自客户端的文字 `SSH_ORIGINAL_COMMAND`，它完全不可信。`parse_request` 只接受四种写法，其余一律拒绝。它不把这段文字交给任何 shell 去解释：这样就不会有命令注入。

接下来查座位。这里有个细节：茶馆不存在、你没有座位，这两种情况给出完全相同的回答，免得外人能试探哪些茶馆存在：

```rust,ignore
// relay/src/shell.rs
const NOT_FOUND: &str =
    "teahouse not found, or you have no seat in it / 茶馆不存在，或者你在这里没有座位";
```

都通过之后，它做了一件有趣的事。它先把身份放进环境变量，再调用 `exec`：

```rust,ignore
// relay/src/shell.rs
    cmd.env("TEAHOUSE_KEY", key.blob_b64());
    cmd.env("TEAHOUSE_HOUSE", &id);
// …
    let err = cmd.exec();
```

这就是第 1.1 节预告的那个系统调用。`exec` 让当前进程**变成**另一个程序：内存换掉，进程号不变。它成功时不会返回，所以这一行之后如果还有代码在运行，那一定是失败了。

为什么不是启动一个子进程再等它？因为 git 要直接和客户端通过标准输入输出对话。`exec` 之后，sshd 连着的就是 git 本身，Relay 不需要转发一个字节，也就没有一行转发数据的代码要审计。

环境变量会被子进程继承。Relay 就是这样把“这是谁”这条信息，穿过 git，一路送到第三道门的。

## 第三道门：pre-receive

git 收到推送后，有一个很多人不知道的细节：新对象先放进**隔离区**（quarantine），这时 `main` 没有动，别人也看不到。然后 git 运行 `pre-receive`，把这次要更新的分支从标准输入传进去。hook 的退出状态不是 0，隔离区整个丢弃，就像什么都没发生过。

所以判决拥有最理想的条件：要检查的东西已经摆在面前，而世界还没有被改变。一次推送里只要有一处违规，全部作废，不会留下半截状态。

`pre_receive` 的开头是全系统最重要的两行：

```rust,ignore
// relay/src/hook.rs
    let members = members_at(&git, MAIN_REF)?;
    let Some(pusher) = members.by_key(&key) else {
```

`MAIN_REF` 此刻还指向推送之前的 `main`。权限来自**现有的**座位表，而不是这次推送带来的那张。

回到伙计的例子。他在提交里把自己写成了老板。如果 Relay 读的是他推送的版本，这一招就成功了。这种错误叫**用被检查的对象来决定检查标准**，很多真实的提权漏洞都是这样来的。

这一点被写在文件最开头的注释里，这是作者最想让你知道的一句话：

```rust,ignore
// relay/src/hook.rs
//! hands us. Authority always comes from the seating chart on the **current**
//! main — never from the commits being pushed — so a push cannot promote the
//! person pushing it.
```

## 判决本身

`check_update` 先判断对分支的操作，`check_commit` 再逐个提交、逐个路径检查。历史只能增长这一条，靠的是 git 自己：

```rust,ignore
// relay/src/hook.rs
            if !cx.git.is_ancestor(&u.old, &u.new)? {
```

新的 `main` 必须是旧 `main` 的后代。这一个判断就挡住了强制推送，连老板也不例外。

再看 `is_ancestor` 的实现，它展示了什么叫**失败即关闭**（fail closed）：

```rust,ignore
// relay/src/git.rs
        match out.status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => bail!(
```

git 的约定是 0 表示是，1 表示不是，其他数字表示出错了。写代码的人很容易只判断是不是 0，于是“出错”就混进了“不是”，甚至“是”。这里三种情况分得清清楚楚，第三种变成错误，错误让 hook 退出，推送被拒绝。**看不懂的情况，一律不放行。**

伙计撞上的那一行是：

```rust,ignore
// relay/src/hook.rs
        if on_main && role != Role::Guest && !may_write_on_main(role, p) {
```

真正的规则是 `core/` 里的纯函数：

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

这就是分层的回报。`relay/` 负责和 git 打交道，把事实取出来；`core/` 负责根据事实作判断，不碰文件，不碰网络，可以用单元测试把每条规则的边界都试一遍。设计的时候把 I/O 推到边上，核心逻辑就能保持纯粹，这个模式到处都用得上。

## 这一节学到的

- 检查要放在绕不过去的地方。Relay 插在 sshd 的强制命令和 git 的 hook 上，自己不开端口。
- `exec` 让进程变成 git，环境变量把身份传过去。
- git 的隔离区让 hook 在世界改变之前判决，一次推送全有或全无。
- 权限永远来自推送之前的 `main`。
- 规则是纯函数，I/O 在外层。不认识的情况一律拒绝。

## 练习

1. 在 `relay/src/hook.rs` 里找到 `pre_receive`。如果一次推送里一个分支违规、一个分支合法，合法的那个会进去吗？
2. 假设 `check_update` 里忘了判断 `is_ancestor`，老板能做什么？别的成员的副本会发生什么？

<details>
<summary>答案</summary>

1. 不会。所有分支的问题都收集进 `problems`，只要不是空的，`pre_receive` 就返回错误，hook 以非零状态退出，git 放弃整次推送。
2. 不过，`relay/src/admin.rs` 还设置了 `receive.denyNonFastForwards`，git 自己也会拒绝，这是第二层防线。如果这一层也没有，老板就能改写历史，抹掉别人的提交。别的成员的副本里还留着旧历史，和 Relay 的新历史对不上。这正是每人一份完整副本的价值：改写是抹不干净的。下一节会讲到，今天的引擎并不会为此报警，这是一个已知的缺口。

</details>
