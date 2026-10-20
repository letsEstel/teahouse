# Teahouse · 茶馆

**A place where small groups (up to 15 people) get things done together.**

> Chat produces feelings; a teahouse produces results.

**English** · [中文](README.zh.md) · [Deutsch](README.de.md)

---

## What it is

Picture a group chat, except that it runs on getting things done instead of chatting:

- Someone raises a **problem**, such as "the stairwell light is broken, who fixes it?" or "which book does the reading club read this term?"
- Everyone writes **results** around that problem: plans, documents, tables, photos.
- Every problem has someone responsible and a status (in progress, solved), so you can see at a glance where things stand.
- Everything anyone has written is kept. Who wrote it and when is on record, and nobody can quietly change it afterwards.

Think of it as **a folder everyone shares, plus a set of rules**. The rules decide who can change things directly, who can only suggest a change, and who can let new people in.

## Why it "can't be killed"

Ordinary group chats and cloud drives live on someone else's server. If the server goes away, or an account is banned, everything is gone.

A teahouse is different: **every member's phone or computer holds a complete copy**. The server that hosts the teahouse (we call it the Relay) is only where everyone meets and compares notes. If the server disappears, any single member's copy can restore the whole teahouse, and the "steward" the host chose can take it over directly.

## Three roles

| Role | What they can do | Think of them as |
|---|---|---|
| **Host** (red square) | Change anything; decide who gets a seat and which one | The owner of the teahouse |
| **Steward** (yellow triangle) | Write problems, results and notices directly, accept other people's suggestions; take over the teahouse if the host is gone | The owner's trusted helper, and heir |
| **Guest** (blue circle) | Read everything and join discussions; to change content, they submit a **proposal** that the host or steward accepts | A regular who comes for tea |

AI can take a seat too, marked "AI" next to its name. Like a person, what it can do depends entirely on the seat it is given.

## Getting started

**Someone invited you:**
1. Download the app (see "Download" below).
2. Open it, choose "Join a teahouse", and paste the whole line of the invitation you were sent (it looks like `TH-XXXX-XXXX-XXXX-XXXX@address`).
3. That's it. No account, no password.

**You want to open your own:** you need a Linux server (a few euros a month is enough). Install it following [Running a Relay](docs/DEPLOY.md) and add `--found` during installation; you get a one-line founding invitation. Paste that line into the app and you are the host.

**You just want a look:** open the app and choose "Look at the example teahouses first". There are six made-up teahouses, all fake data: nothing syncs and nothing leaves your device.

## Download

Latest test build (rebuilt automatically after every code change):

- **Android phone**: [teahouse-android-arm64.apk](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-android-arm64.apk)
- **Windows computer**: [teahouse-windows-x64-setup.exe](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-windows-x64-setup.exe)
- **Server (Linux)**: [teahouse-relay-linux-x86_64.tar.gz](https://github.com/letsEstel/teahouse/releases/download/nightly/teahouse-relay-linux-x86_64.tar.gz); see [Running a Relay](docs/DEPLOY.md)
- All files and when they were updated: [Releases → nightly](https://github.com/letsEstel/teahouse/releases/tag/nightly)

This is an **early test version**: features are still being filled in and there may be bugs. Keep a copy of anything important somewhere else too.

## FAQ

**Do I need an account?** No. Your phone or computer generates a "key" of its own that proves you are you. The key stays on your device; you never see it and never have to remember a password.

**Can my phone and my computer be the same me?** Yes. On a device that already has a seat, tap "Add a device" and paste the pairing code it gives you into the new device. Up to 5 devices per person. (The pairing screen is still being built.)

**What if I lose my phone?** Remove it from one of your other devices, and it can never get in again. If you lose all of them, ask the host to invite you again.

**Can the person running the server read what we write?** Yes. Teahouse does not encrypt content yet, so the server should be in the hands of someone you trust, ideally the host's own server.

**Why at most 15 people?** About 15 is the limit for a small group that can really get things done together. With more people, it is time to split into several teahouses.

**Why "Teahouse"?** A tribute to Lao She's play *Teahouse*: a place where all kinds of people sit down and talk things through to a result.

## Documentation

The documents below are currently written in Chinese.

- [User guide](docs/USER-GUIDE.md): for everyone
- [Running a Relay](docs/DEPLOY.md): for anyone who wants to open their own teahouse
- [For developers](docs/DEVELOP.md): code layout, building, testing, seating an AI, and every developer document

## License

Teahouse is released under the [GNU Affero General Public License v3.0](LICENSE) (AGPL-3.0-only). Anyone may use, modify and redistribute it; but if you offer a modified version to others as a service (including running a Relay for other people), you must make the modified source code available to its users as well.
