//! The JSON the engine returns: CLIENT-API.md §3, field for field.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Open,
    Active,
    Solved,
}

impl Status {
    pub fn parse(s: &str) -> Option<Status> {
        match s {
            "open" => Some(Status::Open),
            "active" => Some(Status::Active),
            "solved" => Some(Status::Solved),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkKind {
    Doc,
    Plan,
    Table,
    Image,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Act {
    OpenProblem,
    SetStatus,
    AddWork,
    UpdateWork,
    Note,
    RemoveNote,
    Notice,
    Propose,
    Adopt,
    Decline,
    Seat,
    Update,
}

impl Act {
    pub fn parse(s: &str) -> Option<Act> {
        Some(match s {
            "open-problem" => Act::OpenProblem,
            "set-status" => Act::SetStatus,
            "add-work" => Act::AddWork,
            "update-work" => Act::UpdateWork,
            "note" => Act::Note,
            "remove-note" => Act::RemoveNote,
            "notice" => Act::Notice,
            "propose" => Act::Propose,
            "adopt" => Act::Adopt,
            "decline" => Act::Decline,
            "seat" => Act::Seat,
            "update" => Act::Update,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncKind {
    Synced,
    Ahead,
    Behind,
    Offline,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncState {
    pub state: SyncKind,
    pub unpushed: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HouseSummary {
    pub id: String,
    pub name: String,
    pub me: String,
    pub my_role: &'static str,
    pub seats: usize,
    pub pending_proposals: usize,
    pub sync: SyncState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub role: &'static str,
    /// `human` or `ai` (docs/MCP.md §2): the interface marks AI members.
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    pub joined: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Event {
    pub by: String,
    pub act: Act,
    pub title: String,
    pub when: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    /// The post's path in the repository.
    pub id: String,
    pub by: String,
    pub text: String,
    /// When it entered the teahouse's `main`.
    pub when: String,
    /// The stamp in its file name.
    pub written_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Notice {
    pub id: String,
    pub by: String,
    pub title: String,
    pub text: String,
    pub when: String,
    pub replies: Vec<Note>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Blob {
    pub hash: String,
    #[serde(rename = "type")]
    pub mime: String,
    pub cached: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub purged: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Work {
    pub id: String,
    pub kind: WorkKind,
    pub title: String,
    pub by: String,
    pub when: String,
    pub size: u64,
    /// One line under the title: frontmatter `note:` (for a table, in its
    /// `.csv.meta.md`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<Blob>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemSummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub status: Status,
    pub by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub opened: String,
    pub updated: String,
    pub work_count: usize,
    pub pending_proposals: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Section {
    pub heading: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Problem {
    #[serde(flatten)]
    pub summary: ProblemSummary,
    pub body: Vec<Section>,
    pub works: Vec<Work>,
    pub events: Vec<Event>,
    pub notes: Vec<Note>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LineKind {
    Same,
    Add,
    Del,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiffLine {
    pub kind: LineKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Change {
    Add,
    Modify,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileChange {
    pub path: String,
    pub title: String,
    pub kind: WorkKind,
    pub change: Change,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<Vec<DiffLine>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub by: String,
    pub problem: String,
    pub when: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub files: Vec<FileChange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Adopted,
    Declined,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Decided {
    pub id: String,
    pub title: String,
    pub by: String,
    pub result: Outcome,
    pub when: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct House {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signboard: Option<String>,
    pub founded: String,
    pub capacity: usize,
    pub me: String,
    pub members: Vec<Member>,
    pub problems: Vec<ProblemSummary>,
    pub recent: Vec<Event>,
    pub proposals: Vec<Proposal>,
    pub decided: Vec<Decided>,
    pub board: Vec<Notice>,
    pub sync: SyncState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Dir,
    Problem,
    Doc,
    Plan,
    Table,
    Image,
    File,
    Notice,
    Note,
    Members,
}

impl From<WorkKind> for NodeKind {
    fn from(k: WorkKind) -> NodeKind {
        match k {
            WorkKind::Doc => NodeKind::Doc,
            WorkKind::Plan => NodeKind::Plan,
            WorkKind::Table => NodeKind::Table,
            WorkKind::Image => NodeKind::Image,
            WorkKind::File => NodeKind::File,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArchiveNode {
    /// The repository path.
    pub id: String,
    /// `problems | works | board | talk | meta`: the interface translates it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<&'static str>,
    /// Only what people wrote: titles, names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub kind: NodeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ArchiveNode>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryEntry {
    pub by: String,
    pub when: String,
    pub act: Act,
    pub problem: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<Vec<DiffLine>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileView {
    pub node: ArchiveNode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub history: Vec<HistoryEntry>,
}

/// What someone wrote that the relay refused (CLIENT-API.md §4 `drafts`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Draft {
    pub id: String,
    /// The command it was written for (`note`, `openProblem`, …).
    pub kind: String,
    pub title: String,
    pub text: String,
    pub when: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
