//! The views that boom computes from the log: the design book, whose
//! parts come from the level that each narrative binds and whose order
//! comes from the outline of the view `book` (views). Nothing stores a
//! view: each is computed from the views of the store for each read
//! (`views-computed-from-the-ledger`).

use std::collections::BTreeMap;

use serde::Serialize;

use crate::claim::{self, Kind};
use crate::judge::{Claim, State, Status, Version};

/// The outline a book is computed from: its name, the version shown, its
/// status, its title and the fingerprint of its body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Head {
    /// The outline's name.
    pub name: String,
    /// Its accepted version, or its newest where none is accepted.
    pub version: String,
    /// That version's status.
    pub status: &'static str,
    /// Its title.
    pub title: String,
    /// The fingerprint of its body, where it has one.
    pub body: Option<String>,
}

/// A narrative in the book, at its shown version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    /// The narrative's name.
    pub narrative: String,
    /// Its accepted version, or its newest where none is accepted.
    pub version: String,
    /// That version's title.
    pub title: String,
    /// That version's status.
    pub status: &'static str,
    /// The count of its open notes.
    pub open_notes: usize,
    /// Whether the outline holds it.
    pub held: bool,
}

/// A section: a context and the narratives that bind it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Section {
    /// The context's name.
    pub context: String,
    /// The narratives that bind it, in the book's order.
    pub narratives: Vec<Entry>,
}

/// A chapter of Part II: a repository, the narratives that bind it, and a
/// section for each context of that repository alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Chapter {
    /// The repository's name.
    pub repository: String,
    /// The narratives that bind the repository, in the book's order.
    pub narratives: Vec<Entry>,
    /// Its sections, in the book's order.
    pub sections: Vec<Section>,
}

/// Part I: the narratives that bind the whole system, then a section for
/// each context of two repositories or more.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct System {
    /// The narratives that bind the whole system, in the book's order.
    pub narratives: Vec<Entry>,
    /// Its sections, in the book's order.
    pub sections: Vec<Section>,
}

/// The design book of a store (views, "The parts of the design book").
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Book {
    /// The outline it is computed from; none where the store has none.
    pub outline: Option<Head>,
    /// Part I. The system.
    pub system: System,
    /// Part II. The repositories: a chapter for each.
    pub repositories: Vec<Chapter>,
}

/// The title of Part I.
pub const PART_I: &str = "Part I. The system";

/// The title of Part II.
pub const PART_II: &str = "Part II. The repositories";

/// A claim's shown version: its accepted version, or its newest where
/// none is accepted.
pub fn shown(claim: &Claim) -> Option<&Version> {
    claim
        .standing
        .as_deref()
        .and_then(|standing| claim.version(standing))
        .or_else(|| claim.versions.last())
}

/// The outline of the design book: the outline with a version of the
/// view `book` that is not rejected or withdrawn, the first by name.
pub fn book_outline(state: &State) -> Option<&Claim> {
    state.claims.values().find(|claim| {
        claim.kind == Kind::Outline
            && claim.versions.iter().any(|version| {
                !matches!(version.status, Status::Rejected | Status::Withdrawn)
                    && version.proposal.view.as_deref() == Some(claim::BOOK)
            })
    })
}

/// Where a narrative sits in the book.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    /// The whole system.
    System,
    /// A context of two repositories or more, in Part I.
    SystemSection(String),
    /// A repository.
    Repository(String),
    /// A context of one repository, in that repository's chapter.
    RepositorySection(String, String),
}

/// The level of `version`: the first repository it binds, else the first
/// context with its repositories, else the whole system.
fn level(state: &State, version: &Version) -> Level {
    if let Some(repository) = version.repositories.first() {
        return Level::Repository(repository.clone());
    }
    let Some(context) = crate::context::names(&version.contexts)
        .first()
        .map(|name| (*name).to_owned())
    else {
        return Level::System;
    };
    let repositories: Vec<String> = state
        .named(&context)
        .first()
        .and_then(|claim| shown(claim))
        .map(|version| version.repositories.clone())
        .unwrap_or_default();
    match repositories.as_slice() {
        [one] => Level::RepositorySection(one.clone(), context),
        _ => Level::SystemSection(context),
    }
}

/// The order of a group: by the outline's first narrative in it, then by
/// its name.
fn group_key(first: Option<usize>, name: &str) -> (bool, usize, String) {
    (first.is_none(), first.unwrap_or(0), name.to_owned())
}

/// The design book of `state`, computed from the outline `outline`, or
/// from the outline of the view `book` where none is named. `state`
/// holds the narratives, the contexts and the outlines of the store, and
/// every note (views).
pub fn book(state: &State, outline: Option<&str>) -> Book {
    let outline = match outline {
        Some(name) => state
            .named(name)
            .into_iter()
            .find(|claim| claim.kind == Kind::Outline),
        None => book_outline(state),
    };
    let head = outline.and_then(|claim| {
        let version = shown(claim)?;
        Some((claim, version))
    });
    let positions: BTreeMap<String, usize> = head
        .map(|(_, version)| {
            version
                .proposal
                .narratives
                .iter()
                .enumerate()
                .filter_map(|(at, edge)| {
                    edge.split_once('@').map(|(name, _)| (name.to_owned(), at))
                })
                .collect()
        })
        .unwrap_or_default();
    let mut placed: BTreeMap<Level, Vec<(Option<usize>, Entry)>> = BTreeMap::new();
    for claim in state
        .claims
        .values()
        .filter(|claim| claim.kind == Kind::Narrative)
    {
        let Some(version) = shown(claim) else {
            continue;
        };
        let position = positions.get(&claim.name).copied();
        let rejected = claim
            .versions
            .iter()
            .all(|version| matches!(version.status, Status::Rejected | Status::Withdrawn));
        if position.is_none() && rejected {
            continue;
        }
        let entry = Entry {
            narrative: claim.name.clone(),
            version: version.proposal.version.clone(),
            title: version.proposal.statement.clone(),
            status: version.status.as_str(),
            open_notes: state.open_notes(&claim.name).len(),
            held: position.is_some(),
        };
        placed
            .entry(level(state, version))
            .or_default()
            .push((position, entry));
    }
    // Inside a level, the narratives the outline holds in its order, then
    // the others by name.
    let mut first: BTreeMap<Level, Option<usize>> = BTreeMap::new();
    let mut levels: BTreeMap<Level, Vec<Entry>> = BTreeMap::new();
    for (level, mut entries) in placed {
        entries.sort_by_key(|(at, entry)| group_key(*at, &entry.narrative));
        first.insert(level.clone(), entries.iter().find_map(|(at, _)| *at));
        levels.insert(level, entries.into_iter().map(|(_, entry)| entry).collect());
    }
    let earliest = |a: Option<usize>, b: Option<usize>| match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    };
    let mut system = System::default();
    let mut system_sections: Vec<(Option<usize>, Section)> = Vec::new();
    let mut chapters: BTreeMap<String, (Option<usize>, Chapter)> = BTreeMap::new();
    for (level, narratives) in levels {
        let at = first.get(&level).copied().flatten();
        match level {
            Level::System => system.narratives = narratives,
            Level::SystemSection(context) => {
                system_sections.push((
                    at,
                    Section {
                        context,
                        narratives,
                    },
                ));
            }
            Level::Repository(repository) => {
                let (chapter_at, chapter) = chapters
                    .entry(repository.clone())
                    .or_insert_with(|| (None, empty_chapter(&repository)));
                *chapter_at = earliest(*chapter_at, at);
                chapter.narratives = narratives;
            }
            Level::RepositorySection(repository, context) => {
                let (chapter_at, chapter) = chapters
                    .entry(repository.clone())
                    .or_insert_with(|| (None, empty_chapter(&repository)));
                *chapter_at = earliest(*chapter_at, at);
                chapter.sections.push(Section {
                    context,
                    narratives,
                });
            }
        }
    }
    system.sections = in_order(system_sections, |section| &section.context);
    let chapters: Vec<(Option<usize>, Chapter)> = chapters
        .into_values()
        .map(|(at, mut chapter)| {
            let sections = std::mem::take(&mut chapter.sections)
                .into_iter()
                .map(|section| (first_held(&section.narratives, &positions), section))
                .collect();
            chapter.sections = in_order(sections, |section| &section.context);
            (at, chapter)
        })
        .collect();
    Book {
        outline: head.map(|(claim, version)| Head {
            name: claim.name.clone(),
            version: version.proposal.version.clone(),
            status: version.status.as_str(),
            title: version.proposal.statement.clone(),
            body: version.proposal.body.clone(),
        }),
        system,
        repositories: in_order(chapters, |chapter| &chapter.repository),
    }
}

fn empty_chapter(repository: &str) -> Chapter {
    Chapter {
        repository: repository.to_owned(),
        narratives: Vec::new(),
        sections: Vec::new(),
    }
}

/// The outline's first position among `entries`.
fn first_held(entries: &[Entry], positions: &BTreeMap<String, usize>) -> Option<usize> {
    entries
        .iter()
        .filter_map(|entry| positions.get(&entry.narrative).copied())
        .min()
}

/// `groups` by the outline's first narrative in each, then by name.
fn in_order<T>(mut groups: Vec<(Option<usize>, T)>, name: impl Fn(&T) -> &String) -> Vec<T> {
    groups.sort_by_key(|(a, x)| group_key(*a, name(x)));
    groups.into_iter().map(|(_, group)| group).collect()
}

impl Book {
    /// Each narrative in the book's order: Part I, then each chapter of
    /// Part II, each level before its sections.
    pub fn order(&self) -> Vec<&Entry> {
        let mut order: Vec<&Entry> = self.system.narratives.iter().collect();
        for section in &self.system.sections {
            order.extend(&section.narratives);
        }
        for chapter in &self.repositories {
            order.extend(&chapter.narratives);
            for section in &chapter.sections {
                order.extend(&section.narratives);
            }
        }
        order
    }

    /// Each narrative of the book that the outline does not hold, in the
    /// book's order.
    pub fn unheld(&self) -> Vec<&Entry> {
        self.order()
            .into_iter()
            .filter(|entry| !entry.held)
            .collect()
    }

    /// The book in words: its outline, then each part, chapter and
    /// section, and each narrative as `<name>@<version>` with its status.
    pub fn words(&self) -> String {
        let mut words = match &self.outline {
            Some(head) => format!(
                "book: {}, outline {}@{} {}\n",
                head.title, head.name, head.version, head.status
            ),
            None => "book: no outline of the view book; propose one with boom propose outline <name> \"<title>\" --view book --narrative <name>\n".to_owned(),
        };
        let entries = |words: &mut String, entries: &[Entry], indent: &str| {
            for entry in entries {
                words.push_str(&format!(
                    "{indent}{}@{} {}",
                    entry.narrative, entry.version, entry.status
                ));
                if !entry.held {
                    words.push_str(", not in the outline");
                }
                words.push('\n');
            }
        };
        words.push_str(PART_I);
        words.push('\n');
        entries(&mut words, &self.system.narratives, "  ");
        for section in &self.system.sections {
            words.push_str(&format!("  {}\n", section.context));
            entries(&mut words, &section.narratives, "    ");
        }
        words.push_str(PART_II);
        words.push('\n');
        for chapter in &self.repositories {
            words.push_str(&format!("  {}\n", chapter.repository));
            entries(&mut words, &chapter.narratives, "    ");
            for section in &chapter.sections {
                words.push_str(&format!("    {}\n", section.context));
                entries(&mut words, &section.narratives, "      ");
            }
        }
        let unheld = self.unheld();
        if !unheld.is_empty() {
            let names: Vec<&str> = unheld
                .iter()
                .map(|entry| entry.narrative.as_str())
                .collect();
            words.push_str(&format!("not held: {}\n", names.join(", ")));
        }
        words
    }
}

// The claims' views (claims-view): the levels of the book, each with the
// claims of some kinds that bind it.

/// A level of the claims' views: the whole system, a repository, or a
/// context under a repository of its globs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ClaimLevel {
    /// The whole system.
    System,
    /// A repository.
    Repository(String),
    /// A context, under one repository of its globs: the repository, then
    /// the context.
    Context(String, String),
}

impl ClaimLevel {
    /// Its id in a page and in the filter `level`: `system`,
    /// `<repository>` or `<repository>/<context>`.
    pub fn id(&self) -> String {
        match self {
            ClaimLevel::System => "system".to_owned(),
            ClaimLevel::Repository(repository) => repository.clone(),
            ClaimLevel::Context(repository, context) => format!("{repository}/{context}"),
        }
    }

    /// Its repository, where it has one.
    pub fn repository(&self) -> Option<&str> {
        match self {
            ClaimLevel::System => None,
            ClaimLevel::Repository(repository) | ClaimLevel::Context(repository, _) => {
                Some(repository)
            }
        }
    }
}

/// A claim and one of its versions.
pub type Held<'a> = (&'a Claim, &'a Version);

/// A level and the claims that bind it: its accepted versions, and its
/// proposed versions, each in the order of the claims' names.
#[derive(Debug, Clone)]
pub struct LevelClaims<'a> {
    /// The level.
    pub level: ClaimLevel,
    /// The accepted globs of a context's level; none for another.
    pub globs: Vec<String>,
    /// The accepted versions that bind it.
    pub accepted: Vec<Held<'a>>,
    /// The proposed versions that bind it.
    pub proposed: Vec<Held<'a>>,
}

/// The repositories of a context, from its accepted version, else its
/// newest, and the globs of its accepted version.
fn context_of(claim: &Claim) -> (Vec<String>, Vec<String>) {
    let repositories = shown(claim)
        .map(|version| version.repositories.clone())
        .unwrap_or_default();
    let globs = claim
        .standing
        .as_deref()
        .and_then(|standing| claim.version(standing))
        .map(|version| version.proposal.paths.clone())
        .unwrap_or_default();
    (repositories, globs)
}

/// The levels of the claims' views of `state`: the whole system, then
/// each repository of `repositories`, of a version or of a context, by
/// name, each followed by its contexts by name; each with the accepted
/// and the proposed versions of `kinds` that bind it. A version binds
/// each repository it names and each context it names, under each
/// repository of that context, and the whole system where it names
/// neither, or a context of no repository.
pub fn claim_levels<'a>(
    state: &'a State,
    kinds: &[Kind],
    repositories: &[String],
) -> Vec<LevelClaims<'a>> {
    let contexts: BTreeMap<&str, (Vec<String>, Vec<String>)> = state
        .claims
        .values()
        .filter(|claim| claim.kind == Kind::Context && shown(claim).is_some())
        .map(|claim| (claim.name.as_str(), context_of(claim)))
        .collect();
    let mut named: std::collections::BTreeSet<String> = repositories.iter().cloned().collect();
    for claim in state.claims.values() {
        for version in &claim.versions {
            named.extend(version.repositories.iter().cloned());
        }
    }
    for (found, _) in contexts.values() {
        named.extend(found.iter().cloned());
    }
    let mut levels = vec![LevelClaims {
        level: ClaimLevel::System,
        globs: Vec::new(),
        accepted: Vec::new(),
        proposed: Vec::new(),
    }];
    for repository in &named {
        levels.push(LevelClaims {
            level: ClaimLevel::Repository(repository.clone()),
            globs: Vec::new(),
            accepted: Vec::new(),
            proposed: Vec::new(),
        });
        for (context, (of, globs)) in &contexts {
            if of.contains(repository) {
                levels.push(LevelClaims {
                    level: ClaimLevel::Context(repository.clone(), (*context).to_owned()),
                    globs: globs.clone(),
                    accepted: Vec::new(),
                    proposed: Vec::new(),
                });
            }
        }
    }
    let at: BTreeMap<ClaimLevel, usize> = levels
        .iter()
        .enumerate()
        .map(|(place, level)| (level.level.clone(), place))
        .collect();
    let places =
        |version: &Version| -> Vec<usize> {
            let mut found: Vec<ClaimLevel> = version
                .repositories
                .iter()
                .map(|repository| ClaimLevel::Repository(repository.clone()))
                .collect();
            for context in crate::context::names(&version.contexts) {
                match contexts.get(context) {
                    Some((of, _)) if !of.is_empty() => found.extend(of.iter().map(|repository| {
                        ClaimLevel::Context(repository.clone(), context.to_owned())
                    })),
                    _ => found.push(ClaimLevel::System),
                }
            }
            if found.is_empty() {
                found.push(ClaimLevel::System);
            }
            let mut places: Vec<usize> = found
                .iter()
                .filter_map(|level| at.get(level).copied())
                .collect();
            places.sort_unstable();
            places.dedup();
            places
        };
    for claim in state
        .claims
        .values()
        .filter(|claim| kinds.contains(&claim.kind))
    {
        if let Some(version) = claim
            .standing
            .as_deref()
            .and_then(|standing| claim.version(standing))
        {
            for place in places(version) {
                levels[place].accepted.push((claim, version));
            }
        }
        for version in claim
            .versions
            .iter()
            .filter(|version| version.status == Status::Proposed)
        {
            for place in places(version) {
                levels[place].proposed.push((claim, version));
            }
        }
    }
    levels
}

/// A version that names a claim with `--source` or `--changes`.
#[derive(Debug, Clone)]
pub struct Naming<'a> {
    /// The claim whose version names it.
    pub claim: &'a Claim,
    /// That version, accepted or proposed.
    pub version: &'a Version,
    /// The edge: `source` or `changes`.
    pub edge: &'static str,
    /// The version of the named claim that the edge meant.
    pub meant: String,
}

/// Each accepted or proposed version of `state` that names the claim
/// `name` with `--source` or `--changes`, with the version meant, in the
/// order of the claims' names, then their versions (claims-view, "The
/// claims that name a claim").
pub fn naming<'a>(state: &'a State, name: &str) -> Vec<Naming<'a>> {
    let mut found = Vec::new();
    for claim in state.claims.values() {
        for version in claim
            .versions
            .iter()
            .filter(|version| matches!(version.status, Status::Accepted | Status::Proposed))
        {
            let edges = version
                .proposal
                .source
                .iter()
                .map(|edge| ("source", edge))
                .chain(
                    version
                        .proposal
                        .changes
                        .iter()
                        .map(|edge| ("changes", edge)),
                );
            for (edge, target) in edges {
                if let Some((named, meant)) = target.split_once('@') {
                    if named == name {
                        found.push(Naming {
                            claim,
                            version,
                            edge,
                            meant: meant.to_owned(),
                        });
                    }
                }
            }
        }
    }
    found
}
