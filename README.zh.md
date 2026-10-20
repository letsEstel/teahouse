# 茶馆

[English](README.md) · **中文** · [Deutsch](README.de.md)

## 序

一群人要把一件事办成，需要三样东西：一份所有人都看得见的问题清单，一处存放成果的地方，一套事先说好的规矩。聊天工具不提供其中任何一样。

茶馆提供这三样。一间茶馆是一个共享的目录，最多十五个座位。座位分三种：老板、伙计、茶客。每一次写入都必须面向一个问题，并由写入者的设备签名。谁能直接写入、谁只能提议，由座位决定，由服务器强制执行。

每位成员的设备上都保存着这间茶馆的完整历史。服务器可以被关停，茶馆不会因此消失。

### 座位

**老板**可以改动一切，包括座位表：谁坐在这间茶馆里，坐哪个位子。

**伙计**直接写问题、成果和公告，采纳或不采纳别人的提案。伙计也是继承人：老板的服务器没了，伙计用自己手里的副本把茶馆重新开起来。

**茶客**阅读全部内容，参与讨论。要添加或修改内容，茶客交一份提案，由老板或伙计采纳或不采纳。两种结果都留在历史里。

座位可以由人坐，也可以由 AI 坐。AI 在出现的每一处都标明自己是 AI；它能做什么，和人一样，由座位决定。

### 原则

- **不改写。** 历史只增不减。任何人，包括老板，都不能修改或删除已经写下的东西。
- **权力来自座位表。** Relay 按每次写入**之前**的座位表来判断这次写入，所以任何写入都不能提升写入者自己。
- **身份留在设备上。** 每台设备生成自己的钥匙，钥匙从不离开这台设备。没有账号，也没有密码。
- **十五个座位。** 十五人大约是一群人还能一起行动的上限。人再多，就该分成几间茶馆。

茶馆之名，致敬老舍的《茶馆》：一间让各色人等坐到一起的屋子。

## 现状

茶馆处于早期测试阶段，可以在安卓和 Windows 上运行，服务器（Relay）运行在 Linux 上。内容目前不加密：运行 Relay 的人能读到它，所以 Relay 应当属于成员们信任的人。重要的东西请在别处另留一份。

## 开始

**加入一间茶馆**：安装 App，选"加入一间茶馆"，把收到的整段邀请（`TH-XXXX-XXXX-XXXX-XXXX@地址`）粘贴进去。

**开一间茶馆**：照[开一个 Relay](docs/DEPLOY.md) 在 Linux 服务器上装好 Relay，安装时加上 `--found`。安装程序会打印一行开馆邀请，谁兑换它，谁就是老板。

**先看看**：在 App 里选"先看看示例茶馆"。示例都是虚构的，只留在你的设备上。

## 下载

根据最新代码自动构建：

- 安卓：[teahouse-android-arm64.apk](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-android-arm64.apk)
- Windows：[teahouse-windows-x64-setup.exe](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-windows-x64-setup.exe)
- Relay（Linux 服务器）：[teahouse-relay-linux-x86_64.tar.gz](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-relay-linux-x86_64.tar.gz)
- 全部文件：[Releases → nightly](https://github.com/letsEstel/teahouse/releases/tag/nightly)

## 文档

- [使用说明](docs/USER-GUIDE.md)：怎样使用一间茶馆，以及常见问题
- [开一个 Relay](docs/DEPLOY.md)：开一间茶馆并维护它
- [开发者入口](docs/DEVELOP.md)：代码、Relay 执行的法、各个接口

## 许可

茶馆以 [GNU Affero General Public License v3.0](LICENSE)（AGPL-3.0-only）发布。任何人都可以使用、修改、再发布。把修改过的版本作为服务提供给别人的，包括运行一台 Relay，必须把修改后的源代码提供给它的使用者。
