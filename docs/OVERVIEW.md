# The big picture: from one problem to thirty thousand lines

**English** · [中文](OVERVIEW.zh.md) · [Deutsch](OVERVIEW.de.md)

The easiest way to get lost in a large project is to start at the first line of the first file. Code is an answer, and you cannot read an answer without knowing the question. This page walks down from the top: what Teahouse is for, which decisions that forced, and only then the code.

## Layer one: the problem

A small group of people want to get something done. Say a housing block wants to sort out its parking. They usually have a group chat. Three months later it holds tens of thousands of messages and cannot answer the three most basic questions:

1. What needs doing, and how far along is it?
2. Where is the finished work, and which copy is the latest?
3. Who decides, and who may change what?

A group chat is a timeline. New messages push old ones away, and nothing stays put. The chat is not badly made; a timeline is simply the wrong shape for this job.

Teahouse gives the group three things instead: a list of problems everyone can see, a place where results are kept, and rules agreed in advance and carried out automatically.

## Layer two: the decisions

The first two are easy. The third is hard. Take "rules carried out automatically" seriously and a chain of decisions follows.

**Rules need someone to enforce them, so there must be a referee.** If the rules live only in each person's software, changing the software gets around them. They have to be judged somewhere outside the users' hands. That place is the **relay**.

**But the referee must not become a single point of failure.** If everything lived only on the relay, shutting it down would end the teahouse, and whoever ran it would hold everyone hostage. So every member's device keeps a complete copy, and the relay is only where the copies meet. The server can go away; the teahouse stays.

**Many copies mean syncing and history.** That is exactly the problem git solved. A teahouse is a git repository, and a write is a commit. Git brings one more property with it: every commit is named by a hash of its content, and that content includes the hash of the commit before it. Change anything in the past and every later name changes. History cannot be quietly rewritten, and that rests on mathematics, not on a promise.

**The referee must know who is who.** Accounts and passwords mean keeping a password database, the most stealable thing a service can own. Instead, each device generates its own key pair. The private key never leaves the device, and the public key goes on the seating chart. The chart itself is a file in the repository, so who may do what has a full history too.

**The rules must not be able to change themselves.** If one push could edit the seating chart and then be judged by the edited chart, anyone could promote themselves. So the relay always judges a push by the chart as it stood **before** that push.

These five decisions interlock. Remove any one and the rest have to be rethought. You will meet them again and again in the code.

One number deserves its own note: **fifteen seats**. The limit has nothing to do with performance; git carries projects like the Linux kernel, with thousands of contributors. Fifteen is an organisational judgement. Beyond it a group starts to divide up work, hold meetings and grow layers, and stops being a group getting one thing done. That is why the number is a law the relay enforces. When a job grows bigger, the answer is another teahouse, not a larger one.

## Layer three: reinvent nothing

In a security system, the most dangerous code is the code you wrote yourself. Teahouse stands as far as it can on things that have survived decades of attack:

| Need | What provides it | Under fire since |
|---|---|---|
| Authentication and encrypted transport | OpenSSH | 1999 |
| Storage, sync, history | git | 2005 |
| Interception and judgement | git hooks | 2005 |
| Signatures | ed25519 | 2011 |

So the relay has no network service of its own and parses no network protocol. It plugs in at two points: OpenSSH calls it after confirming a key, and git calls it before accepting commits. It does the one thing nobody else had done: **judge by seat**.

There is a classic paper on this way of thinking: Saltzer and Schroeder's *The Protection of Information in Computer Systems* (1975). Its principles still hold, and Teahouse uses four of them:

- **Economy of mechanism**: the less code to audit, the better. The relay uses no third-party libraries, so every line that decides who gets in is in this repository.
- **Fail-safe defaults**: when in doubt, refuse.
- **Complete mediation**: every commit and every path is checked, every time. Nobody is waved through because they passed a moment ago.
- **Least privilege**: a guest can only add their own posts; a steward cannot touch the seating chart.

## Layer four: four parts

```text
  ┌────────────── a member's device ───────────────┐
  │                                                │
  │   app/        interface   Svelte + TypeScript  │
  │     │ calls                                    │
  │   client/     engine      Rust: copy, sign, sync│
  │     │                                          │
  └─────┼──────────────────────────────────────────┘
        │ ssh + git
  ┌─────┼──────────────── server ──────────────────┐
  │     ▼                                          │
  │   sshd        OpenSSH, not written by us       │
  │     │                                          │
  │   relay/      gatekeeper  Rust, no dependencies│
  │     │                                          │
  │   core/       rules       Rust, pure, no I/O   │
  └────────────────────────────────────────────────┘
```

`core/` is used by both `relay/` and `client/`, so drawing it on the server side is not quite accurate: it is the text of the law, shared by both.

| Directory | About how many lines | Depends on | What it is |
|---|---|---|---|
| `core/` | 2,500 | nothing | seating chart, keys, rules |
| `relay/` | 7,000 | only `core` | the gatekeeper on the server |
| `client/` | 11,000 | libgit2 and others | the engine on every device |
| `app/` | 8,500 | Svelte, Tauri | the interface, phone and desktop shells |

Dependencies point one way: outer parts depend on inner ones, and `core/` depends on nothing. That direction sets the order for reading the code, and it tells you where a mistake costs the most.

## Layer five: the life of one post

Now string the parts together. Auntie Wang writes a post under problem 0001 and taps send.

1. The **interface** hands the text to the engine with a single command, `note`. The interface knows nothing of git or ssh.
2. The **engine** writes a file `talk/0001/<time>-wang.md` in the local copy, makes a commit with `Problem: 0001` in its message, and signs it with this device's private key.
3. The post is now on her device, and losing the network will not lose it. The engine goes on to try a push.
4. **ssh** connects to the server. sshd uses her public key to confirm she holds the private one, and then lets her run nothing except `teahouse-relay shell <key>`.
5. The **relay's shell** looks at the seating chart: does this key have a seat in this teahouse? If so, it replaces itself with `git receive-pack`.
6. **git** puts the new commit in quarantine, leaves `main` where it is, and runs the `pre-receive` hook.
7. The **relay's hook** examines the commit. Is the signature good? Is she the author? Is there a `Problem:` line? Does that problem exist? She is a guest, so does the change only add a post of her own? Is the timestamp more than ten minutes ahead of the server?
8. If everything passes, git moves `main`. If anything fails, the whole push is void, the engine receives the reason, and the interface shows it.
9. The next time other members sync, they receive the commit. From then on it lives on everyone's device, with its content, its signature and its place in history.

Of these nine steps, our code appears in 1, 2, 3, 5, 7 and 9. The rest belongs to others: older code, attacked far more often.

## Going further

- To use Teahouse: the [user guide](https://github.com/letsEstel/teahouse/blob/main/docs/USER-GUIDE.md) (in Chinese).
- To read the code: [for developers](https://github.com/letsEstel/teahouse/blob/main/docs/DEVELOP.md) (in Chinese). Start with `core/`: it is the smallest part and matters most.

If you lose track of what a piece of code is for, come back to this page and find its place in the diagram.
