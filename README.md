# Teahouse

*Architecture No. 1 in Rust, Op. 1*

**English** · [中文](README.zh.md) · [Deutsch](README.de.md)

## Preface

For a group of people to get something done, they need three things: a list of problems everyone can see, a place where the results are kept, and rules agreed on in advance. A group chat provides none of them.

Teahouse provides all three. A teahouse is a shared directory with at most fifteen seats. There are three kinds of seat: host, steward and guest. Every write must face a problem and be signed by the device of the person who makes it. Who may write directly and who may only propose is decided by the seat and enforced by the server.

Every member's device holds the complete history of the teahouse. The server can be shut down; the teahouse does not disappear with it.

### Seats

The **host** may change anything, including the seating chart: who sits in the teahouse, and in which seat.

The **steward** writes problems, results and notices directly and accepts or declines the proposals of others. The steward is also the heir: if the host's server is gone, the steward reopens the teahouse from their own copy.

A **guest** reads everything and takes part in discussion. To add or change content, a guest submits a proposal, which the host or the steward accepts or declines. Both outcomes remain in the history.

A seat may be held by a person or by an AI. An AI is marked as such wherever it appears; what it may do is decided by its seat, as it is for a person.

### Principles

- **Nothing is rewritten.** The history only grows. Every change is a new entry; nobody, the host included, can alter or erase an earlier one.
- **Authority comes from the seating chart.** The relay judges every write by the chart as it stands before that write, so no write can promote its own author.
- **Identity stays on the device.** Each device generates its own key, and the key never leaves it. There are no accounts and no passwords.
- **Fifteen seats.** Fifteen is about the largest group that can still act together. A larger group should become several teahouses.

## Status

Teahouse is in early testing. It runs on Android and Windows, with a server (the relay) on Linux. Content is not yet encrypted: whoever runs the relay can read it, so the relay should belong to someone the members trust. Keep a copy of anything important elsewhere.

## Getting started

**To join a teahouse**, install the app, choose *Join a teahouse* and paste the whole invitation you received (`TH-XXXX-XXXX-XXXX-XXXX@address`).

**To open a teahouse**, install a relay on a Linux server as described in [Running a relay](docs/DEPLOY.md), adding `--found`. The installer prints a founding invitation; whoever redeems it becomes the host.

**To look first**, choose *Look at the example teahouses first* in the app. The examples are fictional and stay on your device.

## Download

Built automatically from the latest code:

- Android: [teahouse-android-arm64.apk](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-android-arm64.apk)
- Windows: [teahouse-windows-x64-setup.exe](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-windows-x64-setup.exe)
- Relay (Linux server): [teahouse-relay-linux-x86_64.tar.gz](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-relay-linux-x86_64.tar.gz)
- All files: [Releases → nightly](https://github.com/letsEstel/teahouse/releases/tag/nightly)

## Documentation

The following documents are currently in Chinese.

- [User guide](docs/USER-GUIDE.md): using a teahouse, and answers to common questions
- [Running a relay](docs/DEPLOY.md): opening and maintaining a teahouse
- [For developers](docs/DEVELOP.md): the code, the laws the relay enforces, and the interfaces

## License

Teahouse is released under the [GNU Affero General Public License v3.0](LICENSE) (AGPL-3.0-only). Anyone may use, modify and redistribute it. Whoever offers a modified version to others as a service, including by running a relay, must make the modified source code available to its users.
