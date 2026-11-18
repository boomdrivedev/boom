//! Reading a repository's records and rules: a folder of MADR decision
//! records and a rules page in the family's form, each read into a
//! [`Record`] in its file's own words (record-import; design 1 to 4;
//! record 0007).
//!
//! Lines are read as text, with no Markdown parser: a title, a status
//! and a bullet are each one line in the forms read. Everything is read
//! and checked before anything is written, and every refusal names its
//! file.

use std::path::Path;

use yaml_rust2::{Yaml, YamlLoader};

use crate::claim::{History, HistoryStatus, Kind, Proposal};
use crate::entry::{Form, Imported};
use crate::Failure;

/// What a record still needs before it is proposed: a name.
pub const NEEDS_NAME: &str = "a name";
/// What a record still needs before it is proposed: a scenario.
pub const NEEDS_SCENARIO: &str = "a scenario";

/// A record read from a repository's file: a decision record or a rule,
/// in its file's words, with what an agent drafts for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// Where in the import it is: an ADR's file name, or a rule's id.
    pub source: String,
    /// A decision for an ADR, an invariant for a rule.
    pub kind: Kind,
    /// Its name: the file's, or its draft's; none for a rule until its
    /// draft names it.
    pub name: Option<String>,
    /// Its statement: an ADR's title, a rule's words.
    pub statement: String,
    /// Its body: the text after an ADR's title, a rule's details.
    pub body: Option<String>,
    /// Where it came from.
    pub imported: Imported,
    /// Its history, where its file marks it superseded, deprecated or
    /// rejected; a superseded one's successor named once it is known.
    pub history: Option<History>,
    /// The source of a superseded record's successor, a record of the
    /// same folder.
    pub successor: Option<String>,
    /// Its scenarios, from its draft.
    pub scenarios: Vec<String>,
}

impl Record {
    /// What it still needs before it is proposed: a rule a name, and a
    /// record to stand a scenario; history neither.
    pub fn needs(&self) -> Vec<&'static str> {
        let mut needs = Vec::new();
        if self.name.is_none() {
            needs.push(NEEDS_NAME);
        }
        if self.history.is_none() && self.scenarios.is_empty() {
            needs.push(NEEDS_SCENARIO);
        }
        needs
    }

    /// The record as a refusal names it: an ADR by its file, a rule by
    /// its id and its page.
    pub fn named(&self) -> String {
        if self.source == self.imported.from {
            self.source.clone()
        } else {
            format!("{} of {}", self.source, self.imported.from)
        }
    }

    /// Whether its file left it proposed, so it waits to be accepted on
    /// its own (design 9).
    pub fn waits(&self) -> bool {
        self.imported
            .status
            .as_deref()
            .is_some_and(|status| first_word(status) == "proposed")
    }

    /// The version it is proposed as, its body named by its fingerprint.
    pub fn proposal(&self) -> Proposal {
        Proposal::new(
            self.kind,
            self.name.as_deref().unwrap_or_default(),
            &self.statement,
            self.scenarios.clone(),
            self.body
                .as_deref()
                .map(|body| crate::sha256_hex(body.as_bytes())),
            None,
            self.history.clone(),
        )
    }
}

/// A file an import read and did not take as a record, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    /// The file's name.
    pub file: String,
    /// Why it is not a record.
    pub why: String,
}

/// What an import read: its records, in order, and the files skipped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Read {
    /// The records.
    pub records: Vec<Record>,
    /// The files skipped.
    pub skipped: Vec<Skipped>,
}

impl Read {
    /// Names each superseded record's successor by the successor's name,
    /// as read or as its draft gives it.
    pub fn name_successors(&mut self) {
        let names: Vec<(String, Option<String>)> = self
            .records
            .iter()
            .map(|record| (record.source.clone(), record.name.clone()))
            .collect();
        for record in &mut self.records {
            let (Some(successor), Some(history)) = (&record.successor, &mut record.history) else {
                continue;
            };
            history.by = names
                .iter()
                .find(|(source, _)| source == successor)
                .and_then(|(_, name)| name.clone());
        }
    }
}

fn refused_in(file: &str, why: impl std::fmt::Display) -> Failure {
    Failure::refused(format!("{file}: {why}"))
}

fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// Reads a file as text, refused naming it where it cannot be read or is
/// not UTF-8.
fn read_text(path: &Path, named: &str) -> Result<String, Failure> {
    let bytes = std::fs::read(path)
        .map_err(|error| refused_in(named, format!("cannot be read: {error}")))?;
    String::from_utf8(bytes).map_err(|_| refused_in(named, "is not UTF-8 text"))
}

/// The leading run of letters of `text`, lowercase: a status's word.
fn first_word(text: &str) -> String {
    text.trim_start()
        .chars()
        .take_while(char::is_ascii_alphabetic)
        .collect::<String>()
        .to_ascii_lowercase()
}

/// The leading digits of a file's name and the rest after their hyphen,
/// where it begins with digits and a hyphen.
fn numbered(file: &str) -> Option<(&str, &str)> {
    let digits = file.len() - file.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    if digits == 0 {
        return None;
    }
    file[digits..]
        .strip_prefix('-')
        .map(|rest| (&file[..digits], rest))
}

/// A number kept as four digits.
fn four_digits(number: u64) -> String {
    format!("{number:04}")
}

/// A frontmatter's YAML, where the text opens with one: the YAML and the
/// line the rest of the text starts at.
fn frontmatter<'a>(file: &str, text: &'a str) -> Result<(Yaml, Vec<&'a str>), Failure> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.first().map(|line| line.trim_end()) != Some("---") {
        return Ok((Yaml::Null, lines));
    }
    let Some(close) = lines
        .iter()
        .skip(1)
        .position(|line| line.trim_end() == "---")
    else {
        return Err(refused_in(
            file,
            "its frontmatter opens with --- and never closes",
        ));
    };
    let yaml = lines[1..=close].join("\n");
    let mut documents = YamlLoader::load_from_str(&yaml)
        .map_err(|error| refused_in(file, format!("its frontmatter is not YAML: {error}")))?;
    let document = if documents.is_empty() {
        Yaml::Null
    } else {
        documents.swap_remove(0)
    };
    match document {
        Yaml::Hash(_) | Yaml::Null => Ok((document, lines[close + 2..].to_vec())),
        _ => Err(refused_in(file, "its frontmatter is not a map of fields")),
    }
}

/// A frontmatter field as text, where it is a string or a number.
fn field_text(yaml: &Yaml, field: &str) -> Option<String> {
    match &yaml[field] {
        Yaml::String(text) | Yaml::Real(text) => Some(text.clone()),
        Yaml::Integer(number) => Some(number.to_string()),
        Yaml::Boolean(value) => Some(value.to_string()),
        _ => None,
    }
}

/// A record's title and the text after it: the first line outside a
/// fenced block that opens `# `, and the lines after it, the blank lines
/// that open them left out.
fn title_and_body(lines: &[&str]) -> Option<(String, Option<String>)> {
    let mut fenced = false;
    for (at, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if let Some(title) = line.strip_prefix("# ") {
            let rest: Vec<&str> = lines[at + 1..]
                .iter()
                .copied()
                .skip_while(|line| line.trim().is_empty())
                .collect();
            let body = if rest.is_empty() {
                None
            } else {
                Some(format!("{}\n", rest.join("\n")))
            };
            return Some((title.trim().to_owned(), body));
        }
    }
    None
}

/// What a MADR file's frontmatter and name give, before the folder's
/// records are known.
struct Madr {
    record: Record,
    number: Option<u64>,
    /// A superseded record's status, read for its successor.
    superseded: Option<String>,
}

/// The statuses boom reads, as a refusal names them.
const STATUSES: &str = "accepted, proposed, superseded, deprecated or rejected";

fn read_madr(path: &Path, file: &str, repository: &str) -> Result<Option<Madr>, Failure> {
    let text = read_text(path, file)?;
    let (yaml, lines) = frontmatter(file, &text)?;
    let named = field_text(&yaml, "name");
    let from_file = numbered(file);
    if from_file.is_none() && named.is_none() {
        return Ok(None);
    }
    let (statement, body) = title_and_body(&lines)
        .ok_or_else(|| refused_in(file, "holds no title, a line opening `# `"))?;
    let stem = file.strip_suffix(".md").unwrap_or(file);
    let name = named.unwrap_or_else(|| numbered(stem).map_or(stem, |(_, rest)| rest).to_owned());
    let file_number = from_file.map(|(digits, _)| digits.parse::<u64>());
    let file_number = match file_number {
        Some(Ok(number)) => Some(number),
        Some(Err(_)) => return Err(refused_in(file, "its name's number is too long")),
        None => None,
    };
    let given = match &yaml["number"] {
        Yaml::Integer(number) => Some(
            u64::try_from(*number)
                .map_err(|_| refused_in(file, format!("its number {number} is not a record's")))?,
        ),
        Yaml::String(text) => Some(
            text.trim()
                .parse::<u64>()
                .map_err(|_| refused_in(file, format!("its number {text:?} is not a number")))?,
        ),
        Yaml::Null | Yaml::BadValue => None,
        _ => return Err(refused_in(file, "its number is not a number")),
    };
    let number = match (file_number, given) {
        (Some(ours), Some(theirs)) if ours != theirs => {
            return Err(refused_in(
                file,
                format!(
                    "its name gives the number {} and its frontmatter {}",
                    four_digits(ours),
                    four_digits(theirs)
                ),
            ))
        }
        (Some(number), _) | (None, Some(number)) => Some(number),
        (None, None) => None,
    };
    let status = field_text(&yaml, "status");
    let history = match status.as_deref().map(first_word).as_deref() {
        None | Some("accepted" | "proposed") => None,
        Some("superseded") => Some(HistoryStatus::Superseded),
        Some("deprecated") => Some(HistoryStatus::Deprecated),
        Some("rejected") => Some(HistoryStatus::Rejected),
        Some(_) => {
            return Err(refused_in(
                file,
                format!(
                    "its status {:?} is none boom reads: {STATUSES}",
                    status.unwrap_or_default()
                ),
            ))
        }
    };
    let deciders = match &yaml["decision-makers"] {
        Yaml::Array(items) => items
            .iter()
            .filter_map(|item| match item {
                Yaml::String(text) | Yaml::Real(text) => Some(text.clone()),
                Yaml::Integer(number) => Some(number.to_string()),
                _ => None,
            })
            .collect(),
        _ => field_text(&yaml, "decision-makers").into_iter().collect(),
    };
    let superseded =
        (history == Some(HistoryStatus::Superseded)).then(|| status.clone().unwrap_or_default());
    let record = Record {
        source: file.to_owned(),
        kind: Kind::Decision,
        name: Some(name),
        statement,
        body,
        imported: Imported {
            repository: repository.to_owned(),
            from: file.to_owned(),
            form: Form::Madr,
            legacy: number.map(four_digits),
            status,
            date: field_text(&yaml, "date"),
            deciders,
            section: None,
            drafted: true,
        },
        history: history.map(|status| History { status, by: None }),
        successor: None,
        scenarios: Vec::new(),
    };
    Ok(Some(Madr {
        record,
        number,
        superseded,
    }))
}

/// The file a status's Markdown link names, `](<file>)`, without its
/// folder or anchor.
fn linked_file(status: &str) -> Option<&str> {
    let start = status.find("](")? + 2;
    let target = &status[start..];
    let target = &target[..target.find(')')?];
    let target = target.split('#').next().unwrap_or_default();
    target.rsplit('/').next().filter(|file| !file.is_empty())
}

/// The first number in a status, `superseded by ADR-0003`.
fn first_number(status: &str) -> Option<u64> {
    let start = status.find(|c: char| c.is_ascii_digit())?;
    let digits: String = status[start..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

/// Reads a folder of MADR files: each `.md` file directly in it whose
/// name begins with digits and a hyphen, or whose frontmatter gives a
/// `name`, as a decision; every other file skipped (design 2).
pub fn read_adr(dir: &Path, repository: &str) -> Result<Read, Failure> {
    let folder = dir.display().to_string();
    let entries = std::fs::read_dir(dir)
        .map_err(|error| refused_in(&folder, format!("cannot be read: {error}")))?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|error| refused_in(&folder, format!("cannot be read: {error}")))?;
        paths.push(entry.path());
    }
    paths.sort();
    let mut read = Read::default();
    let mut madrs = Vec::new();
    for path in paths {
        let file = file_name(&path);
        if path.is_dir() {
            read.skipped.push(Skipped {
                file,
                why: "a folder: an import reads the files directly in its folder".to_owned(),
            });
            continue;
        }
        if !file.ends_with(".md") {
            read.skipped.push(Skipped {
                file,
                why: "not a .md file".to_owned(),
            });
            continue;
        }
        match read_madr(&path, &file, repository)? {
            Some(madr) => madrs.push(madr),
            None => read.skipped.push(Skipped {
                file,
                why: "its name begins with no number, and its frontmatter gives no name".to_owned(),
            }),
        }
    }
    let found: Vec<(String, Option<u64>)> = madrs
        .iter()
        .map(|madr| (madr.record.source.clone(), madr.number))
        .collect();
    for mut madr in madrs {
        if let Some(status) = &madr.superseded {
            let linked =
                linked_file(status).filter(|linked| found.iter().any(|(file, _)| file == linked));
            let successor = linked.map(str::to_owned).or_else(|| {
                let number = first_number(status)?;
                found
                    .iter()
                    .find(|(_, theirs)| *theirs == Some(number))
                    .map(|(file, _)| file.clone())
            });
            let Some(successor) = successor else {
                return Err(refused_in(
                    &madr.record.source,
                    format!("its status {status:?} names no record of the folder as its successor"),
                ));
            };
            madr.record.successor = Some(successor);
        }
        read.records.push(madr.record);
    }
    read.name_successors();
    Ok(read)
}

/// The lines of a page's `## <heading>` section, up to the next `## `.
fn section<'a>(lines: &[&'a str], heading: &str) -> Option<Vec<&'a str>> {
    let start = lines.iter().position(|line| line.trim_end() == heading)?;
    Some(
        lines[start + 1..]
            .iter()
            .copied()
            .take_while(|line| !line.starts_with("## "))
            .collect(),
    )
}

/// The bullets of `lines` that open with `opening(id)`: each id, the
/// bullet's first line after its opening, and the lines indented under
/// it, as written.
fn bullets<'a>(
    lines: &[&'a str],
    opening: impl Fn(&'a str) -> Option<(&'a str, &'a str)>,
) -> Vec<Bullet<'a>> {
    let mut found: Vec<Bullet<'a>> = Vec::new();
    let mut open = false;
    let mut heading: Option<&'a str> = None;
    for line in lines {
        if let Some(text) = line.strip_prefix("### ") {
            heading = Some(text.trim());
            open = false;
            continue;
        }
        if let Some((id, words)) = opening(line) {
            found.push(Bullet {
                id,
                words,
                lines: vec![line],
                heading,
            });
            open = true;
            continue;
        }
        let continues =
            open && !line.trim().is_empty() && line.starts_with(|c: char| c.is_whitespace());
        match found.last_mut() {
            Some(bullet) if continues => bullet.lines.push(line),
            _ => open = false,
        }
    }
    found
}

/// A bullet of a rules page.
struct Bullet<'a> {
    id: &'a str,
    words: &'a str,
    lines: Vec<&'a str>,
    heading: Option<&'a str>,
}

impl Bullet<'_> {
    /// Its words on one line: its first line's and its continuation
    /// lines', joined with one space.
    fn one_line(&self) -> String {
        std::iter::once(self.words.trim())
            .chain(self.lines[1..].iter().map(|line| line.trim()))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The bullet as written.
    fn as_written(&self) -> String {
        format!("{}\n", self.lines.join("\n"))
    }
}

/// A rule's bullet, `- **[<id>]** <words>`.
fn rule_opening(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("- **[")?;
    let close = rest.find("]**")?;
    let id = &rest[..close];
    let words = rest[close + 3..].strip_prefix(' ')?;
    (!id.is_empty() && !id.contains(char::is_whitespace)).then_some((id, words))
}

/// A rule's details, `- **<id>** <words>`.
fn details_opening(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("- **")?;
    if rest.starts_with('[') {
        return None;
    }
    let close = rest.find("**")?;
    let id = &rest[..close];
    let words = rest[close + 2..].strip_prefix(' ')?;
    (!id.is_empty() && !id.contains(char::is_whitespace)).then_some((id, words))
}

/// Reads a rules page: each bullet of its `## Rules` that opens
/// `**[<id>]**` an invariant, its words its statement and its details
/// its body, from the page's `## Details` or, where it has none,
/// `<stem>-details.md` beside it (design 3).
pub fn read_rules(page: &Path, repository: &str) -> Result<Read, Failure> {
    let file = file_name(page);
    let text = read_text(page, &file)?;
    let (_, lines) = frontmatter(&file, &text)?;
    let rules = section(&lines, "## Rules")
        .ok_or_else(|| refused_in(&file, "has no `## Rules` section"))?;
    let rules = bullets(&rules, rule_opening);
    if rules.is_empty() {
        return Err(refused_in(
            &file,
            "holds no rule: its `## Rules` has no bullet opening `**[<id>]**`",
        ));
    }
    for (at, rule) in rules.iter().enumerate() {
        if rules[..at].iter().any(|before| before.id == rule.id) {
            return Err(refused_in(
                &file,
                format!("gives the rule {} twice", rule.id),
            ));
        }
    }
    let beside_text;
    let details = match section(&lines, "## Details") {
        Some(details) => bullets(&details, details_opening),
        None => {
            let stem = file.strip_suffix(".md").unwrap_or(&file);
            let beside = page.with_file_name(format!("{stem}-details.md"));
            if beside.is_file() {
                beside_text = read_text(&beside, &file_name(&beside))?;
                let beside_lines: Vec<&str> = beside_text.lines().collect();
                bullets(&beside_lines, details_opening)
            } else {
                Vec::new()
            }
        }
    };
    let records = rules
        .iter()
        .map(|rule| {
            let body = details
                .iter()
                .find(|details| details.id == rule.id)
                .map(Bullet::as_written);
            Record {
                source: rule.id.to_owned(),
                kind: Kind::Invariant,
                name: None,
                statement: rule.one_line(),
                body,
                imported: Imported {
                    repository: repository.to_owned(),
                    from: file.clone(),
                    form: Form::Rules,
                    legacy: Some(rule.id.to_owned()),
                    status: None,
                    date: None,
                    deciders: Vec::new(),
                    section: rule.heading.map(str::to_owned),
                    drafted: true,
                },
                history: None,
                successor: None,
                scenarios: Vec::new(),
            }
        })
        .collect();
    Ok(Read {
        records,
        skipped: Vec::new(),
    })
}

/// The format a drafts file names.
pub const DRAFTS_FORMAT: &str = "boom-import-drafts/1";

/// What an agent drafted for one record: its scenarios and, where it
/// names it, its name.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    /// Its scenarios.
    #[serde(default)]
    pub scenarios: Vec<String>,
    /// Its name.
    pub name: Option<String>,
}

/// A drafts file: what an agent drafted for each record, by its source
/// (design 4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drafts {
    /// The file's name, as a refusal names it.
    pub file: String,
    /// Each record's draft, by its source.
    pub records: std::collections::BTreeMap<String, Draft>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DraftsFile {
    format: String,
    #[serde(default)]
    records: std::collections::BTreeMap<String, Draft>,
}

/// Reads a drafts file, refused naming it where it is not a
/// `boom-import-drafts/1` object.
pub fn read_drafts(path: &Path) -> Result<Drafts, Failure> {
    let file = path.display().to_string();
    let text = read_text(path, &file)?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|error| refused_in(&file, format!("is not JSON: {error}")))?;
    match value.get("format").and_then(serde_json::Value::as_str) {
        Some(DRAFTS_FORMAT) => {}
        Some(other) => {
            return Err(refused_in(
                &file,
                format!("its format is {other}, not {DRAFTS_FORMAT}"),
            ))
        }
        None => {
            return Err(refused_in(
                &file,
                format!("names no format, {DRAFTS_FORMAT}"),
            ))
        }
    }
    let drafts: DraftsFile = serde_json::from_value(value)
        .map_err(|error| refused_in(&file, format!("its form: {error}")))?;
    debug_assert_eq!(drafts.format, DRAFTS_FORMAT);
    Ok(Drafts {
        file,
        records: drafts.records,
    })
}

impl Drafts {
    /// Gives each record the scenarios and name its draft gives, and
    /// nothing else; refused naming a draft whose key names no record of
    /// the import.
    pub fn apply(self, read: &mut Read) -> Result<(), Failure> {
        for key in self.records.keys() {
            if !read.records.iter().any(|record| &record.source == key) {
                return Err(refused_in(
                    &self.file,
                    format!("its draft {key} names no record of this import"),
                ));
            }
        }
        for record in &mut read.records {
            let Some(draft) = self.records.get(&record.source) else {
                continue;
            };
            record.scenarios.clone_from(&draft.scenarios);
            if let Some(name) = &draft.name {
                record.name = Some(name.clone());
            }
        }
        read.name_successors();
        Ok(())
    }
}

/// Refuses two records of one import under one name, naming both files
/// (design 7).
pub fn check_names(records: &[Record]) -> Result<(), Failure> {
    for (at, record) in records.iter().enumerate() {
        let Some(name) = &record.name else { continue };
        if let Some(other) = records[..at]
            .iter()
            .find(|other| other.name.as_ref() == Some(name))
        {
            return Err(Failure::refused(format!(
                "two records of this import are named {name}, {} and {}: name one otherwise in its draft",
                other.named(),
                record.named()
            )));
        }
    }
    Ok(())
}
