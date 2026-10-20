# 开一个 Relay · Deploying a relay

> 给老板（或者帮老板运维的人）看的。目标：一台最便宜的 Linux 服务器，几分钟装好。
> 技术细节和茶馆的法见 [RELAY.md](RELAY.md)。

## 你需要

- **一台 Linux 服务器**，Debian 12 或 Ubuntu 22.04 及以上，有公网地址。最便宜的一档就够（1 核、2 GB 内存、20 GB 硬盘），例如 Hetzner、Netcup，每月几欧元。
- 自己电脑上能用 `ssh` 登录这台服务器（用 root 或能 `sudo` 的账号）。
- 只是自己试用的话，不需要服务器：见文末"在自己电脑上试"。

## 方式一：安装脚本（推荐）

1. 下载 [teahouse-relay-linux-x86_64.tar.gz](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-relay-linux-x86_64.tar.gz)（仓库私有时需要先登录 GitHub；也可以在 Releases → nightly 里找到）。
2. 把它传到服务器上，解开，运行安装脚本：
   ```sh
   scp teahouse-relay-linux-x86_64.tar.gz root@<服务器地址>:
   ssh root@<服务器地址>
   tar xzf teahouse-relay-linux-x86_64.tar.gz && cd teahouse-relay
   sh deploy/install.sh --binary ./teahouse-relay
   ```
3. 脚本最后会打印 Relay 的地址，例如 `teahouse@203.0.113.7`；加了 `--found` 时还会打印开张邀请（见下一节）。

脚本做的事：装好 OpenSSH 和 git；把 Relay 装到 `/usr/lib/teahouse`（一路都只有 root 可写，sshd 要求如此），并在 `/usr/local/bin` 放一个链接；建一个没有特权的系统用户 `teahouse`；初始化 `/var/lib/teahouse`；配好 sshd（只影响 `teahouse` 这一个用户）；开启每日完整性检查。**重复运行是安全的**，升级时换上新的 `teahouse-relay` 再跑一遍即可。

## 方式二：Docker

```sh
docker build -f deploy/docker/Dockerfile -t teahouse-relay .
docker run -d --name teahouse -p 22:22 -v teahouse:/var/lib/teahouse --restart unless-stopped teahouse-relay
```
服务器自己的 ssh 已经占了 22 端口时，改成 `-p 2222:22`，Relay 地址就是 `teahouse@<服务器地址>:2222`。
所有数据（茶馆、附件、邀请、审计日志、Relay 和 sshd 的钥匙）都在 `teahouse` 这个卷里；换新镜像升级不会改变任何人的身份。

## 开第一间茶馆

**用开张邀请（推荐）**：安装时多给两个选项，脚本最后打印一行完整邀请：
```sh
sh deploy/install.sh --binary ./teahouse-relay --found wutong --name "梧桐里三号院"
```
```
开张邀请 · Founding invitation（一次性，7 天内有效）
    TH-7KQ2-M9XF-3RDP-WC5L@203.0.113.7
```
把这一整行发给要当老板的人（微信、邮件都行）。他在客户端里点"凭邀请入座"，粘贴这一行，写上自己的名字，这间茶馆就开张了，他就是老板。**谁都不用碰公钥**：客户端自己生成设备钥匙，兑换时交给 Relay。

- `wutong` 是茶馆的编号（小写字母、数字和 `-`），出现在地址里；`--name` 是给人看的名字，可以是中文。可选 `--signboard "一句话招牌"`。
- 邀请里的地址默认是这台服务器的第一个 IP。有域名，或者服务器在 NAT 后面时，加 `--address relay.example.org`（端口不是 22 时写成 `relay.example.org:2222`）。
- 邀请只能用一次，7 天内有效。过期或者发错了人，再生成一个就行（见下）；只要还没人兑换，茶馆就不存在。
- 已经装好的 Relay，随时可以再开一间：
  ```sh
  sudo -u teahouse teahouse-relay --root /var/lib/teahouse found wutong --name "梧桐里三号院" --address 203.0.113.7
  ```
  Docker 下：`docker exec -u teahouse teahouse teahouse-relay found wutong --name "梧桐里三号院" --address 203.0.113.7:2222`
- 兑换走的是 sshd 的 AuthorizedKeysCommand（安装脚本和 Docker 都是这样配的）。手动配置成 authorized_keys 文件的 Relay 不认识陌生钥匙，没法兑换邀请。

**已经有老板的公钥时**，也可以直接开张：
```sh
sh deploy/install.sh --binary ./teahouse-relay \
  --open wutong --name "梧桐里三号院" --host-name "陈立" --host-key @device.pub
```
（`device.pub` 是客户端应用数据目录里的 `teahouse/device.pub`。）Docker 下：`docker exec -u teahouse teahouse teahouse-relay open wutong --name … --host-id chenli --host-name 陈立 --host-key "ssh-ed25519 AAAA…"`

之后的成员都靠老板在客户端里点"邀请人入座"。

## 日常运维

| 做什么 | 命令 |
|---|---|
| 看有哪些茶馆 | `sudo -u teahouse teahouse-relay list` |
| 健康检查 | `sudo -u teahouse teahouse-relay check` |
| 谁做了什么 | `/var/lib/teahouse/log/audit.log` |
| 登录被拒、端口转发尝试 | `journalctl -u ssh` |
| 备份 | 备份整个 `/var/lib/teahouse`（每个成员手里也都有完整副本） |
| 升级 | 换上新的 `teahouse-relay`，再运行一次 `install.sh` |

## 在自己电脑上试（不需要服务器）

在 WSL 或任何 Linux 里，从仓库根目录运行 `scripts/dev-stack.sh`：它会起一个本地 Relay，种好六间示例茶馆，再打开界面。手机连不到你电脑上的 WSL，要和手机或朋友一起用，就得用上面的服务器。

## 安装方式本身有自动测试

`.github/workflows/deploy.yml` 在全新的 Ubuntu 上真的跑一遍 `install.sh`，再构建并运行 Docker 镜像，然后用真实的 ssh 和 git 检查：老板能看到茶馆、能克隆、签名的推送被接受、未签名的被拒绝、陌生钥匙进不来、拿不到 shell、陌生钥匙只能兑换邀请（`scripts/deploy-test.sh`）。
