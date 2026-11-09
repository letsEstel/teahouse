//! 大云晓钟
//!
//! The inbox (0026): what is new for this device's member, worked out from
//! the room's own history; nothing comes from a push service.
//!
//! * A step concerns me when someone else (never I) mentions me in a post
//!   (`@<my name>` or `@<my id>`), decides one of my proposals, does
//!   something on a problem I raised or own, or puts up a notice. One step
//!   makes one notification, the most specific of these.
//! * Open proposals of others are waiting for the host and the steward.
//! * A history changed at the relay (0020) is one too.
//!
//! Each device keeps its own record (`teahouse-inbox.json` in the clone's
//! git directory): where the inbox starts, so what was there before is not
//! news, and what has been seen. A step's notification is its commit id, so
//! it stays the same however often it is worked out.

use super::Snapshot;
use crate::error::Result;
use crate::format;
use crate::model::{Act, Notification, NotificationKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;
use teahouse_core::rules::{format_rfc3339, TalkTopic};
use teahouse_core::Role;

const FILE: &str = "teahouse-inbox.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Record {
    /// The commit the inbox starts after: what was on `main` before it is
    /// not news.
    pub since: Option<String>,
    /// Notifications already seen.
    #[serde(default)]
    pub seen: BTreeSet<String>,
}

impl Record {
    pub fn load(git_dir: &Path) -> Option<Record> {
        let b = std::fs::read(git_dir.join(FILE)).ok()?;
        serde_json::from_slice(&b).ok()
    }

    pub fn save(&self, git_dir: &Path) -> Result<()> {
        let b =
            serde_json::to_vec(self).map_err(|e| crate::error::Error::internal(e.to_string()))?;
        std::fs::write(git_dir.join(FILE), b)?;
        Ok(())
    }
}

/// Called around every sync: the first time, the inbox starts at what this
/// device had before (`before`), or, for a fresh clone, at what it got.
pub fn start(git_dir: &Path, before: Option<String>, after: Option<String>) {
    if Record::load(git_dir).is_none() {
        let _ = Record {
            since: before.or(after),
            seen: BTreeSet::new(),
        }
        .save(git_dir);
    }
}

impl Snapshot {
    /// What is new for this device's member and not seen yet, newest first.
    pub fn inbox(&self) -> Result<Vec<Notification>> {
        let record = self.inbox_record()?;
        Ok(self
            .notifications(&record)?
            .into_iter()
            .filter(|n| !record.seen.contains(&n.id))
            .collect())
    }

    /// Marks notifications as seen: these, or all there are now.
    pub fn mark_seen(&self, ids: Option<&[String]>) -> Result<()> {
        let mut record = self.inbox_record()?;
        match ids {
            Some(ids) => record.seen.extend(ids.iter().cloned()),
            None => {
                // Everything up to now: the steps by moving the start, the
                // rest (proposals waiting, a changed history) by id.
                let now = self.notifications(&record)?;
                record.since = Some(self.head.clone());
                record.seen = now
                    .into_iter()
                    .filter(|n| {
                        n.kind == NotificationKind::Proposal || n.kind == NotificationKind::History
                    })
                    .map(|n| n.id)
                    .collect();
            }
        }
        record.save(self.git.dir())
    }

    fn inbox_record(&self) -> Result<Record> {
        match Record::load(self.git.dir()) {
            Some(r) => Ok(r),
            // Never synced through this engine (a room made locally): it
            // starts now.
            None => {
                let r = Record {
                    since: Some(self.head.clone()),
                    seen: BTreeSet::new(),
                };
                r.save(self.git.dir())?;
                Ok(r)
            }
        }
    }

    fn notifications(&self, record: &Record) -> Result<Vec<Notification>> {
        let Some(me) = self.me.as_ref() else {
            return Ok(Vec::new());
        };
        // A start the history no longer has (it was changed): from now on.
        let start = match &record.since {
            Some(h) => match self.steps.iter().position(|s| &s.hash == h) {
                Some(i) => i + 1,
                None => self.steps.len(),
            },
            None => 0,
        };
        let tokens = mention_tokens(&self.members);
        let mut out = Vec::new();
        for s in &self.steps[start..] {
            let by = self.by(s);
            if by == me.id {
                continue;
            }
            let act = format::infer_act(s.act.first().map(String::as_str), &s.changes);
            let problem = s
                .problem
                .first()
                .filter(|p| p.len() == 4 && p.bytes().all(|b| b.is_ascii_digit()))
                .cloned();
            let mut n = Notification {
                id: s.hash.clone(),
                house: self.id.clone(),
                kind: NotificationKind::Notice,
                by: by.clone(),
                title: s.subject.clone(),
                when: format_rfc3339(s.time),
                problem: problem.clone(),
                proposal: None,
                notice: None,
                post: None,
            };
            // 1. Someone names me in a post.
            let mentioned = s
                .changes
                .iter()
                .filter(|(c, _)| *c == 'A')
                .find_map(|(_, p)| {
                    let post = format::parse_post_path(p)?;
                    let text = self.text(p).ok()??;
                    mentioned_ids(&text, &tokens)
                        .contains(&me.id.as_str())
                        .then(|| (p.clone(), post.topic))
                });
            if let Some((post, topic)) = mentioned {
                n.kind = NotificationKind::Mention;
                n.post = Some(post);
                match topic {
                    TalkTopic::Board(notice) => n.notice = Some(notice.to_string()),
                    TalkTopic::Proposal { owner, name } => {
                        n.proposal = Some(format!("{owner}/{name}"))
                    }
                    TalkTopic::Problem(_) => {}
                }
                out.push(n);
                continue;
            }
            // 2. One of my proposals was decided.
            if matches!(act, Act::Adopt | Act::Decline) {
                if let Some(p) = s
                    .proposal
                    .first()
                    .filter(|p| p.split_once('/').is_some_and(|(o, _)| o == me.id))
                {
                    n.kind = NotificationKind::Decision;
                    n.proposal = Some(p.clone());
                    out.push(n);
                    continue;
                }
            }
            // 3. Something on a problem I raised or own.
            if let Some(pid) = &problem {
                let mine = self.problem_path(pid).is_some_and(|(_, path)| {
                    let owner = self
                        .text(path)
                        .ok()
                        .flatten()
                        .map(|t| format::parse_problem(&t, pid, "").owner);
                    owner.flatten().as_deref() == Some(me.id.as_str())
                        || self.added_step(path).is_some_and(|o| self.by(o) == me.id)
                });
                let about_it = matches!(
                    act,
                    Act::Note
                        | Act::AddWork
                        | Act::UpdateWork
                        | Act::SetStatus
                        | Act::UpdateProblem
                        | Act::Adopt
                        | Act::Decline
                );
                if mine && about_it {
                    n.kind = NotificationKind::Problem;
                    out.push(n);
                    continue;
                }
            }
            // 4. A new notice.
            if act == Act::Notice {
                n.notice = s
                    .changes
                    .iter()
                    .find_map(|(c, p)| (*c == 'A').then(|| format::parse_notice_path(p)).flatten())
                    .map(str::to_string);
                out.push(n);
            }
        }
        // Newest first; within one second, the later step first.
        out.reverse();
        // Proposals waiting for the host or the steward: again when reworked.
        if matches!(me.role, Role::Host | Role::Steward) {
            for p in self.proposals(false)? {
                if p.by == me.id {
                    continue;
                }
                out.push(Notification {
                    id: format!("proposal:{}:{}", p.id, p.when),
                    house: self.id.clone(),
                    kind: NotificationKind::Proposal,
                    by: p.by.clone(),
                    title: p.title.clone(),
                    when: p.when.clone(),
                    problem: Some(p.problem.clone()).filter(|x| !x.is_empty()),
                    proposal: Some(p.id.clone()),
                    notice: None,
                    post: None,
                });
            }
        }
        // The relay's history was changed (0020).
        if let Some(h) = crate::sync::Record::load(self.git.dir()).history {
            out.push(Notification {
                id: format!("history:{}:{}:{}", self.head, h.dropped, h.added),
                house: self.id.clone(),
                kind: NotificationKind::History,
                by: String::new(),
                title: format!("-{} +{}", h.dropped, h.added),
                when: String::new(),
                problem: None,
                proposal: None,
                notice: None,
                post: None,
            });
        }
        out.sort_by(|a, b| b.when.cmp(&a.when));
        Ok(out)
    }
}

/// `@<name>` and `@<id>` of every member, longest first, so `@陈立国` is not
/// taken for `@陈立`.
fn mention_tokens(members: &teahouse_core::Members) -> Vec<(String, &str)> {
    let mut out: Vec<(String, &str)> = Vec::new();
    for m in &members.members {
        for s in [m.name.as_str(), m.id.as_str()] {
            if !s.is_empty() && !out.iter().any(|(t, _)| t[1..] == *s) {
                out.push((format!("@{s}"), m.id.as_str()));
            }
        }
    }
    out.sort_by_key(|t| std::cmp::Reverse(t.0.len()));
    out
}

/// The members a text names, by the interface's rule (app/src/lib/mentions.ts,
/// `splitMentions`), so that mail addresses and domains are not mentions:
/// no letter, digit or `._%+-` right before the `@`; and a name ending in a
/// letter or digit must not run on into a letter, digit, `_` or `.x` (a
/// Chinese name may: `@陈立辛苦了`).
fn mentioned_ids<'a>(text: &str, tokens: &[(String, &'a str)]) -> Vec<&'a str> {
    let ends_word = |t: &str| t.chars().last().is_some_and(|c| c.is_ascii_alphanumeric());
    let runs_on = |after: &str| {
        let mut cs = after.chars();
        match cs.next() {
            Some(c) if c.is_ascii_alphanumeric() || c == '_' => true,
            Some('.') => cs.next().is_some_and(|c| c.is_ascii_alphanumeric()),
            _ => false,
        }
    };
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(off) = text[i..].find('@') {
        let at = i + off;
        let glued = text[..at]
            .chars()
            .last()
            .is_some_and(|c| c.is_ascii_alphanumeric() || "._%+-".contains(c));
        let hit = if glued {
            None
        } else {
            tokens.iter().find(|(t, _)| {
                text[at..].starts_with(t.as_str())
                    && !(ends_word(t) && runs_on(&text[at + t.len()..]))
            })
        };
        match hit {
            Some((t, id)) => {
                if !out.contains(id) {
                    out.push(*id);
                }
                i = at + t.len();
            }
            None => i = at + 1,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mentions_are_not_mail_addresses() {
        let tokens: Vec<(String, &str)> = vec![
            ("@陈立国".into(), "chen2"),
            ("@陈立".into(), "chen"),
            ("@li".into(), "li"),
        ];
        let ids = |t: &str| mentioned_ids(t, &tokens);
        let none: Vec<&str> = Vec::new();
        assert_eq!(ids("@li 你好"), ["li"]);
        assert_eq!(ids("写信给 chen@li.com 吧"), none);
        assert_eq!(ids("@li.com 是域名"), none);
        assert_eq!(ids("@lisa 不是 li"), none);
        assert_eq!(ids("@li_x"), none);
        assert_eq!(ids("a+@li"), none);
        assert_eq!(ids("好的@li。"), ["li"]);
        assert_eq!(ids("找 @li."), ["li"]);
        assert_eq!(ids("@陈立辛苦了"), ["chen"]);
        assert_eq!(ids("@陈立国来了"), ["chen2"]);
        assert_eq!(ids("@li @li @陈立"), ["li", "chen"]);
    }
}
