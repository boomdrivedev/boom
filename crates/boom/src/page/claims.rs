//! The views of the claims on `boom page`, beside the design book: the
//! rules, the decisions, the terms and every claim, by level, with the
//! proposed versions apart from the accepted ones, filters and a search
//! with no script, and the page of each claim (claims-view). Each is
//! computed from the views at the log's last entry, for each request
//! (`views-computed-from-the-ledger`).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use super::{
    actor_words, binds, body_text, escape, markdown, note_box, shell, time_words, Context, Shell,
    View, CLAIMS, DECISIONS, RULES, TERMS,
};
use crate::claim::{Kind, Origin};
use crate::judge::{Claim, State, Status, Version};
use crate::store;
use crate::view::{ClaimLevel, Held, LevelClaims};
use crate::Failure;

/// A view of the claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Of {
    /// Each invariant and assumption.
    Rules,
    /// Each decision.
    Decisions,
    /// Each term.
    Terms,
    /// Each claim of every kind but narrative and outline.
    Claims,
}

impl Of {
    /// The view `rest`, the path after `<base>/`, names.
    pub fn from_path(rest: &str) -> Option<Of> {
        match rest {
            "rules" => Some(Of::Rules),
            "decisions" => Some(Of::Decisions),
            "terms" => Some(Of::Terms),
            "claims" | "claims/" => Some(Of::Claims),
            _ => None,
        }
    }

    fn view(self) -> View {
        match self {
            Of::Rules => RULES,
            Of::Decisions => DECISIONS,
            Of::Terms => TERMS,
            Of::Claims => CLAIMS,
        }
    }

    /// Its kinds, in the order it gives them.
    fn kinds(self) -> &'static [Kind] {
        match self {
            Of::Rules => &[Kind::Invariant, Kind::Assumption],
            Of::Decisions => &[Kind::Decision],
            Of::Terms => &[Kind::Term],
            Of::Claims => &[
                Kind::Invariant,
                Kind::Decision,
                Kind::Assumption,
                Kind::Term,
                Kind::Context,
            ],
        }
    }

    /// The words under its title.
    fn about(self) -> &'static str {
        match self {
            Of::Rules => "Each accepted invariant and assumption, by the level that it binds: what an agent keeps and what it assumes.",
            Of::Decisions => "Each accepted decision, by the level that it binds, in the order that a person accepted them.",
            Of::Terms => "Each accepted term, by its repository: its word and its meaning.",
            Of::Claims => "Each accepted claim of every kind but narrative and outline, by the level that it binds, then by kind.",
        }
    }

    /// One of it, and many, in words.
    fn nouns(self) -> (&'static str, &'static str) {
        match self {
            Of::Rules => ("rule", "rules"),
            Of::Decisions => ("decision", "decisions"),
            Of::Terms => ("term", "terms"),
            Of::Claims => ("claim", "claims"),
        }
    }
}

/// Which statuses a view gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Statuses {
    /// The proposed versions apart, then the accepted ones.
    Both,
    /// The accepted versions alone.
    Accepted,
    /// The proposed versions alone.
    Proposed,
}

/// The filters of a view, from its query.
#[derive(Debug, Clone)]
struct Filters {
    kind: Option<Kind>,
    status: Statuses,
    repository: Option<String>,
    level: Option<String>,
    q: String,
}

impl Filters {
    /// The query that gives these filters again, `?...`, or nothing.
    fn query(&self) -> String {
        let mut pairs = Vec::new();
        if let Some(kind) = self.kind {
            pairs.push(format!("kind={}", kind.as_str()));
        }
        match self.status {
            Statuses::Both => {}
            Statuses::Accepted => pairs.push("status=accepted".to_owned()),
            Statuses::Proposed => pairs.push("status=proposed".to_owned()),
        }
        if let Some(repository) = &self.repository {
            pairs.push(format!("repository={}", encoded(repository)));
        }
        if let Some(level) = &self.level {
            pairs.push(format!("level={}", encoded(level)));
        }
        if !self.q.is_empty() {
            pairs.push(format!("q={}", encoded(&self.q)));
        }
        if pairs.is_empty() {
            String::new()
        } else {
            format!("?{}", pairs.join("&amp;"))
        }
    }
}

/// `text` with each byte that is not a letter, a digit or `-._~/` as `%XX`.
fn encoded(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte) {
            out.push(char::from(byte));
        } else {
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

/// The filters of `fields` for the view `of`, or the words of a 400: the
/// field and the values that it takes.
fn filters(
    of: Of,
    fields: &[(String, String)],
    repositories: &[String],
    levels: &[String],
) -> Result<Filters, String> {
    let field = |name: &str| {
        fields
            .iter()
            .find(|(field, _)| field == name)
            .map(|(_, value)| value.trim())
            .filter(|value| !value.is_empty())
    };
    let takes = |name: &str, values: Vec<String>| {
        format!(
            "The field {name} takes {}, or nothing for each.",
            values.join(", ")
        )
    };
    let kind = match field("kind") {
        None => None,
        Some(given) => Some(
            of.kinds()
                .iter()
                .copied()
                .find(|kind| kind.as_str() == given)
                .ok_or_else(|| {
                    takes(
                        "kind",
                        of.kinds()
                            .iter()
                            .map(|kind| kind.as_str().to_owned())
                            .collect(),
                    )
                })?,
        ),
    };
    let status = match field("status") {
        None | Some("both") => Statuses::Both,
        Some("accepted") => Statuses::Accepted,
        Some("proposed") => Statuses::Proposed,
        Some(_) => {
            return Err(takes(
                "status",
                vec!["both".into(), "accepted".into(), "proposed".into()],
            ))
        }
    };
    let repository = match field("repository") {
        None => None,
        Some(given) if repositories.iter().any(|name| name == given) => Some(given.to_owned()),
        Some(_) => return Err(takes("repository", repositories.to_vec())),
    };
    let level = match field("level") {
        None => None,
        Some(given) if levels.iter().any(|id| id == given) => Some(given.to_owned()),
        Some(_) => return Err(takes("level", levels.to_vec())),
    };
    Ok(Filters {
        kind,
        status,
        repository,
        level,
        q: field("q").unwrap_or_default().to_owned(),
    })
}

/// The repositories of the store: those registered, and each that a
/// version or a context names.
fn repositories_of(conn: &rusqlite::Connection) -> Result<Vec<String>, Failure> {
    Ok(store::repositories(conn)?
        .into_iter()
        .map(|repository| repository.name)
        .collect())
}

/// The title of a level.
fn level_title(level: &ClaimLevel) -> String {
    match level {
        ClaimLevel::System => "The whole system".to_owned(),
        ClaimLevel::Repository(repository) => repository.clone(),
        ClaimLevel::Context(_, context) => context.clone(),
    }
}

/// The page of the view `of`, with the query's `fields`: `Ok(Err(words))`
/// for a value that a field does not take.
pub fn view_page(
    conn: &rusqlite::Connection,
    of: Of,
    fields: &[(String, String)],
    context: &Context,
) -> Result<Result<String, String>, Failure> {
    let state = store::load_state(conn)?;
    let (entry, _) = store::head_entry(conn)?;
    let registered = repositories_of(conn)?;
    let mut levels = crate::view::claim_levels(&state, of.kinds(), &registered);
    if of == Of::Terms {
        levels.retain(|level| level.level != ClaimLevel::System);
    }
    let repositories: Vec<String> = levels
        .iter()
        .filter_map(|level| match &level.level {
            ClaimLevel::Repository(repository) => Some(repository.clone()),
            _ => None,
        })
        .collect();
    let ids: Vec<String> = levels.iter().map(|level| level.level.id()).collect();
    let filters = match filters(of, fields, &repositories, &ids) {
        Ok(filters) => filters,
        Err(words) => return Ok(Err(words)),
    };
    let levels = kept(conn, levels, &filters)?;
    let base = &context.base;
    let view = of.view();
    let path = format!("{base}{}", view.path);
    let query = filters.query();
    let mut main = format!(
        "<h1>{}</h1>\n<p class=\"hint\">{}</p>\n",
        view.words,
        of.about()
    );
    main.push_str(&filter_form(of, &filters, &path, &repositories, &ids));
    let accepted: BTreeSet<(&str, &str)> = levels
        .iter()
        .flat_map(|level| &level.accepted)
        .map(|(claim, version)| (claim.name.as_str(), version.proposal.version.as_str()))
        .collect();
    let proposed: Vec<(&str, &str)> = {
        let mut seen = BTreeSet::new();
        levels
            .iter()
            .flat_map(|level| &level.proposed)
            .map(|(claim, version)| (claim.name.as_str(), version.proposal.version.as_str()))
            .filter(|at| seen.insert(*at))
            .collect()
    };
    // The one command names the newest proposed version of each claim:
    // `boom accept` takes one version of a claim.
    let newest: Vec<(&str, &str)> = {
        let mut by_claim: BTreeMap<&str, (u64, &str)> = BTreeMap::new();
        for (claim, version) in levels.iter().flat_map(|level| &level.proposed) {
            let entry = by_claim
                .entry(claim.name.as_str())
                .or_insert((version.proposed_in, version.proposal.version.as_str()));
            if version.proposed_in > entry.0 {
                *entry = (version.proposed_in, version.proposal.version.as_str());
            }
        }
        proposed
            .iter()
            .copied()
            .filter(|(name, version)| {
                by_claim
                    .get(name)
                    .is_some_and(|(_, newest)| newest == version)
            })
            .collect()
    };
    let (one, many) = of.nouns();
    let counted = |count: usize| if count == 1 { one } else { many };
    let _ = writeln!(
        main,
        "<p class=\"meta count\">{} accepted {} · {} proposed {}, at entry {entry} of the store's log</p>",
        accepted.len(),
        counted(accepted.len()),
        proposed.len(),
        counted(proposed.len()),
    );
    if filters.status != Statuses::Accepted && !proposed.is_empty() {
        main.push_str(&proposed_html(conn, of, &levels, &newest, base)?);
    }
    if filters.status != Statuses::Proposed {
        main.push_str(&levels_html(conn, of, &levels, base)?);
    }
    let page = Shell {
        title: view.words,
        entry,
        base,
        view: Some(view),
        side: Some(levels_side(of, &levels, &path, &query, &filters)),
        query: "",
    };
    Ok(Ok(shell(&page, &main)))
}

/// The levels that `filters` keep, each with the versions that they keep.
fn kept<'a>(
    conn: &rusqlite::Connection,
    levels: Vec<LevelClaims<'a>>,
    filters: &Filters,
) -> Result<Vec<LevelClaims<'a>>, Failure> {
    let contexts = match &filters.repository {
        Some(repository) => store::contexts_in(conn, repository)?,
        None => Vec::new(),
    };
    let mut levels: Vec<LevelClaims<'a>> = levels
        .into_iter()
        .filter(|level| {
            filters
                .level
                .as_ref()
                .is_none_or(|id| level.level.id() == *id)
                && filters.repository.as_ref().is_none_or(|repository| {
                    level.level.repository().is_none_or(|own| own == repository)
                })
        })
        .collect();
    let keep = |(claim, version): &Held<'_>| {
        filters.kind.is_none_or(|kind| claim.kind == kind)
            && filters
                .repository
                .as_ref()
                .is_none_or(|repository| version.binds(repository, &contexts))
    };
    for level in &mut levels {
        level.accepted.retain(keep);
        level.proposed.retain(keep);
    }
    let words = crate::search::words(
        &filters
            .q
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>(),
    );
    if words.is_empty() {
        return Ok(levels);
    }
    // One row for each version kept, searched with boom search's matching.
    let mut held: Vec<Held<'a>> = Vec::new();
    let mut seen = BTreeSet::new();
    for (claim, version) in levels
        .iter()
        .flat_map(|level| level.accepted.iter().chain(&level.proposed))
    {
        if seen.insert((claim.name.clone(), version.proposal.version.clone())) {
            held.push((claim, version));
        }
    }
    let bodies = held
        .iter()
        .map(|(_, version)| match &version.proposal.body {
            Some(fingerprint) => Ok(String::from_utf8(store::blob(conn, fingerprint)?).ok()),
            None => Ok(None),
        })
        .collect::<Result<Vec<_>, Failure>>()?;
    let searched: Vec<crate::search::Claim<'_>> = held
        .iter()
        .zip(&bodies)
        .map(|((claim, version), body)| {
            let imported = version.imported.as_ref();
            crate::search::Claim {
                name: &claim.name,
                legacy: imported.and_then(|imported| imported.legacy.as_deref()),
                statement: &version.proposal.statement,
                section: imported.and_then(|imported| imported.section.as_deref()),
                scenarios: &version.proposal.scenarios,
                body: body.as_deref(),
            }
        })
        .collect();
    let found: BTreeSet<(String, String)> = crate::search::search(&searched, &words)?
        .into_iter()
        .map(|found| {
            let (claim, version) = held[found.place];
            (claim.name.clone(), version.proposal.version.clone())
        })
        .collect();
    let matched = |(claim, version): &Held<'_>| {
        found.contains(&(claim.name.clone(), version.proposal.version.clone()))
    };
    for level in &mut levels {
        level.accepted.retain(matched);
        level.proposed.retain(matched);
    }
    Ok(levels)
}

/// The form of a view's filters: a `GET` to its own path.
fn filter_form(
    of: Of,
    filters: &Filters,
    path: &str,
    repositories: &[String],
    levels: &[String],
) -> String {
    let select = |name: &str, all: &str, values: &[String], chosen: Option<&str>| {
        let mut html =
            format!("<label>{name} <select name=\"{name}\"><option value=\"\">{all}</option>");
        for value in values {
            let selected = if chosen == Some(value.as_str()) {
                " selected"
            } else {
                ""
            };
            let _ = write!(
                html,
                "<option value=\"{value}\"{selected}>{value}</option>",
                value = escape(value)
            );
        }
        html.push_str("</select></label>");
        html
    };
    let kinds: Vec<String> = of
        .kinds()
        .iter()
        .map(|kind| kind.as_str().to_owned())
        .collect();
    let status = match filters.status {
        Statuses::Both => "both",
        Statuses::Accepted => "accepted",
        Statuses::Proposed => "proposed",
    };
    let mut html = format!("<form class=\"filters\" method=\"get\" action=\"{path}\">");
    if kinds.len() > 1 {
        html.push_str(&select(
            "kind",
            "every kind",
            &kinds,
            filters.kind.map(Kind::as_str),
        ));
    }
    let statuses = ["both", "accepted", "proposed"].map(str::to_owned);
    let mut status_html = String::from("<label>status <select name=\"status\">");
    for value in &statuses {
        let selected = if value == status { " selected" } else { "" };
        let _ = write!(
            status_html,
            "<option value=\"{value}\"{selected}>{value}</option>"
        );
    }
    status_html.push_str("</select></label>");
    html.push_str(&status_html);
    html.push_str(&select(
        "repository",
        "every repository",
        repositories,
        filters.repository.as_deref(),
    ));
    html.push_str(&select(
        "level",
        "every level",
        levels,
        filters.level.as_deref(),
    ));
    let _ = write!(
        html,
        "<label>words <input type=\"search\" name=\"q\" value=\"{}\" placeholder=\"Words to find\"></label><button type=\"submit\">Show</button></form>\n",
        escape(&filters.q)
    );
    html
}

/// The sidebar of a view: each level, with its count of accepted and of
/// proposed versions, as a link to that level on the view.
fn levels_side(
    of: Of,
    levels: &[LevelClaims<'_>],
    path: &str,
    query: &str,
    filters: &Filters,
) -> String {
    let mut html = format!(
        "<a class=\"book-title\" href=\"{path}\">{}</a>\n",
        of.view().words
    );
    let link = |html: &mut String, level: &LevelClaims<'_>, class: &str| {
        let proposed = if level.proposed.is_empty() || filters.status == Statuses::Accepted {
            String::new()
        } else {
            format!(
                "<span class=\"unheld\">{} proposed</span>",
                level.proposed.len()
            )
        };
        let count = if filters.status == Statuses::Proposed {
            String::new()
        } else {
            format!(" <span class=\"count\">{}</span>", level.accepted.len())
        };
        let _ = writeln!(
            html,
            "<li{class}><a href=\"{path}{query}#{id}\">{}{count}{proposed}</a></li>",
            escape(&level_title(&level.level)),
            id = escape(&level.level.id()),
        );
    };
    let system: Vec<&LevelClaims<'_>> = levels
        .iter()
        .filter(|level| level.level == ClaimLevel::System)
        .collect();
    if !system.is_empty() {
        let _ = writeln!(html, "<p class=\"part\">{}</p>\n<ul>", crate::view::PART_I);
        for level in system {
            link(&mut html, level, "");
        }
        html.push_str("</ul>\n");
    }
    let _ = writeln!(html, "<p class=\"part\">{}</p>\n<ul>", crate::view::PART_II);
    for level in levels
        .iter()
        .filter(|level| level.level != ClaimLevel::System)
    {
        let class = if matches!(level.level, ClaimLevel::Context(..)) {
            " class=\"in\""
        } else {
            ""
        };
        link(&mut html, level, class);
    }
    html.push_str("</ul>\n");
    html
}

/// The command that accepts `targets`.
fn accept_command(targets: &[(&str, &str)]) -> String {
    let named: Vec<String> = targets
        .iter()
        .map(|(name, version)| format!("{}@{version}", escape(name)))
        .collect();
    format!("boom accept {} --key &lt;key&gt;", named.join(" "))
}

/// "Proposed, binding nothing": each proposed version of the view's
/// kinds, grouped by the level that it binds, each in a frame of its
/// own, with the command that accepts it, and at its head one command
/// that accepts each.
fn proposed_html(
    conn: &rusqlite::Connection,
    of: Of,
    levels: &[LevelClaims<'_>],
    proposed: &[(&str, &str)],
    base: &str,
) -> Result<String, Failure> {
    let mut html = format!(
        "<section class=\"waiting\" id=\"proposed\">\n<h2>Proposed, binding nothing</h2>\n<p class=\"hint\">A person accepts each with their key. To accept every one of them at once, at the newest version of each claim:</p>\n<pre class=\"command\">{}</pre>\n",
        accept_command(proposed)
    );
    for level in levels.iter().filter(|level| !level.proposed.is_empty()) {
        let _ = writeln!(
            html,
            "<h3 class=\"group\">{}</h3>",
            escape(&level.level.id().replace('/', " · "))
        );
        let mut sorted = level.proposed.clone();
        sort_held(of, &mut sorted);
        for held in &sorted {
            html.push_str(&item_html(conn, of, held, base, true)?);
        }
    }
    html.push_str("</section>\n");
    Ok(html)
}

/// The entry that accepted the first version of `claim`, where one did.
fn first_accepted(claim: &Claim) -> Option<u64> {
    claim
        .versions
        .iter()
        .filter(|version| version.decided_by.is_some() && version.reason.is_none())
        .filter_map(|version| version.decided_in)
        .min()
}

/// `held` in the view's order: decisions by the entry that accepted
/// their first version, the claims of every kind by kind, then each by
/// name.
fn sort_held(of: Of, held: &mut [Held<'_>]) {
    match of {
        Of::Decisions => held.sort_by_key(|(claim, _)| {
            (
                first_accepted(claim).unwrap_or(u64::MAX),
                claim.name.clone(),
            )
        }),
        _ => held.sort_by_key(|(claim, _)| {
            (
                of.kinds()
                    .iter()
                    .position(|kind| *kind == claim.kind)
                    .unwrap_or(usize::MAX),
                claim.name.clone(),
            )
        }),
    }
}

/// The date of the entry `seq`, `YYYY-MM-DD`, where its signed part
/// gives one.
fn entry_date(conn: &rusqlite::Connection, seq: u64) -> Result<Option<String>, Failure> {
    let Some(row) = store::row_at(conn, seq)? else {
        return Ok(None);
    };
    Ok(serde_json::from_str::<serde_json::Value>(&row.signed)
        .ok()
        .and_then(|signed| signed["at"].as_u64())
        .map(|at| time_words(at)[..10].to_owned()))
}

/// The item of a claim in a view: its kind, `<name>@<version>` as a link
/// to its page, and its statement, with its body folded; a proposed one
/// in a frame of its own, marked, with the command that accepts it.
fn item_html(
    conn: &rusqlite::Connection,
    of: Of,
    (claim, version): &Held<'_>,
    base: &str,
    proposed: bool,
) -> Result<String, Failure> {
    let name = escape(&claim.name);
    let at = format!("{name}@{}", version.proposal.version);
    let class = if proposed { "item proposed" } else { "item" };
    let mut html = format!("<div class=\"{class}\" data-claim=\"{name}\"><div class=\"meta\">");
    if proposed {
        html.push_str("<span class=\"mark\">proposed</span> ");
    }
    let _ = write!(
        html,
        "{} <a href=\"{base}/claims/{at}\"><code>{at}</code></a>",
        claim.kind.as_str()
    );
    if !proposed && of == Of::Decisions {
        if let Some(date) = version
            .decided_in
            .map(|seq| entry_date(conn, seq))
            .transpose()?
            .flatten()
        {
            let _ = write!(html, " · accepted {date}");
        }
        if claim
            .versions
            .last()
            .is_some_and(|newest| newest.status == Status::Proposed)
        {
            html.push_str(" · <span class=\"late\">a newer version is proposed</span>");
        }
    }
    html.push_str("</div>");
    if of == Of::Terms {
        let _ = write!(
            html,
            "<p><a class=\"word\" href=\"{base}/claims/{at}\">{name}</a>: {}</p>",
            escape(&version.proposal.statement)
        );
    } else {
        let _ = write!(html, "<p>{}</p>", escape(&version.proposal.statement));
    }
    if version.proposal.body.is_some() && of != Of::Terms {
        let body = body_text(conn, version.proposal.body.as_deref())?;
        let _ = write!(
            html,
            "<details><summary>details</summary>{}</details>",
            markdown(&body)
        );
    }
    if proposed {
        let _ = write!(
            html,
            "<pre class=\"command\">{}</pre>",
            accept_command(&[(&claim.name, &version.proposal.version)])
        );
    }
    html.push_str("</div>\n");
    Ok(html)
}

/// Each level with its accepted versions: Part I, the whole system; Part
/// II, each repository and its contexts, each context with its globs.
fn levels_html(
    conn: &rusqlite::Connection,
    of: Of,
    levels: &[LevelClaims<'_>],
    base: &str,
) -> Result<String, Failure> {
    let (_, many) = of.nouns();
    let mut html = String::new();
    let mut part_two = false;
    for level in levels {
        match &level.level {
            ClaimLevel::System => {
                let _ = writeln!(html, "<h2 id=\"system\">{}</h2>", crate::view::PART_I);
            }
            ClaimLevel::Repository(repository) => {
                if !part_two {
                    part_two = true;
                    let _ = writeln!(html, "<h2>{}</h2>", crate::view::PART_II);
                }
                let _ = writeln!(html, "<h3 id=\"{id}\">{id}</h3>", id = escape(repository));
            }
            ClaimLevel::Context(_, context) => {
                if !part_two {
                    part_two = true;
                    let _ = writeln!(html, "<h2>{}</h2>", crate::view::PART_II);
                }
                let _ = writeln!(
                    html,
                    "<h4 id=\"{}\" class=\"context\"><a href=\"{base}/claims/{name}\">{name}</a></h4>",
                    escape(&level.level.id()),
                    name = escape(context)
                );
                if !level.globs.is_empty() {
                    let globs: Vec<String> = level
                        .globs
                        .iter()
                        .map(|glob| format!("<code>{}</code>", escape(glob)))
                        .collect();
                    let _ = writeln!(html, "<p class=\"meta globs\">{}</p>", globs.join(" · "));
                }
            }
        }
        if level.accepted.is_empty() {
            let _ = writeln!(html, "<p class=\"hint\">No accepted {many} here.</p>");
            continue;
        }
        let mut sorted = level.accepted.clone();
        sort_held(of, &mut sorted);
        for held in &sorted {
            html.push_str(&item_html(conn, of, held, base, false)?);
        }
    }
    Ok(html)
}

/// The page of the claim `name` at `version`, or at its accepted
/// version, else its newest (claims-view, "A claim's page").
pub fn claim_page(
    conn: &rusqlite::Connection,
    name: &str,
    version: Option<&str>,
    context: &Context,
) -> Result<String, Failure> {
    let state = store::load_state(conn)?;
    let (entry, _) = store::head_entry(conn)?;
    let base = &context.base;
    let claim = state
        .named(name)
        .into_iter()
        .find(|claim| version.is_none_or(|version| claim.version(version).is_some()))
        .ok_or_else(|| match version {
            Some(version) if !state.named(name).is_empty() => {
                Failure::refused(format!("the claim {name} has no version {version}"))
            }
            _ => Failure::refused(format!("no claim has the name {name}")),
        })?;
    let shown = match version {
        Some(version) => claim.version(version).expect("the version was found"),
        None => crate::view::shown(claim)
            .ok_or_else(|| Failure::not_run(format!("the claim {name} has no version")))?,
    };
    let at = format!("{}@{}", escape(name), shown.proposal.version);
    let mut main = format!(
        "<h1>{}</h1>\n<p class=\"meta\">{} <code>{at}</code> · <span class=\"status\">{}</span> · proposed by {}",
        escape(&shown.proposal.statement),
        claim.kind.as_str(),
        shown.status.as_str(),
        escape(&actor_words(&shown.proposer)),
    );
    if let Some(by) = &shown.decided_by {
        let act = if shown.reason.is_some() {
            "rejected"
        } else {
            "accepted"
        };
        let _ = write!(main, " · {act} by {}", escape(by));
    }
    if let Some(amended) = &shown.proposal.base {
        let _ = write!(
            main,
            " · amends <a href=\"{base}/claims/{}@{amended}\">{amended}</a>",
            escape(name)
        );
    }
    let _ = writeln!(main, " · binds {}</p>", escape(&binds(shown)));
    // The versions that count: the accepted and the proposed ones.
    let counted: Vec<String> = claim
        .versions
        .iter()
        .rev()
        .filter(|other| matches!(other.status, Status::Accepted | Status::Proposed))
        .map(|other| {
            let v = &other.proposal.version;
            let here = if *v == shown.proposal.version {
                " class=\"here\""
            } else {
                ""
            };
            format!(
                "<a{here} href=\"{base}/claims/{}@{v}\">{v}</a> {}",
                escape(name),
                other.status.as_str()
            )
        })
        .collect();
    if !counted.is_empty() {
        let _ = writeln!(
            main,
            "<p class=\"versions\">versions: {}</p>",
            counted.join(" · ")
        );
    }
    if shown.status == Status::Proposed {
        let _ = writeln!(
            main,
            "<div class=\"waiting frame\"><p><span class=\"mark\">proposed, binds nothing</span> A person accepts it or rejects it in a terminal, with their key:</p><pre class=\"command\">boom accept {at} --key &lt;key&gt;</pre><pre class=\"command\">boom reject {at} --reason \"&lt;why&gt;\" --key &lt;key&gt;</pre></div>"
        );
    }
    if shown.proposal.body.is_some() {
        main.push_str("<div class=\"body\">");
        main.push_str(&markdown(&body_text(conn, shown.proposal.body.as_deref())?));
        main.push_str("</div>\n");
    }
    if !shown.proposal.replaces.is_empty() {
        let words: Vec<String> = shown
            .proposal
            .replaces
            .iter()
            .map(|word| format!("<code>{}</code>", escape(word)))
            .collect();
        let _ = writeln!(
            main,
            "<h2>Old words</h2>\n<p>It replaces {}.</p>",
            words.join(", ")
        );
    }
    if claim.kind == Kind::Context {
        main.push_str(&context_html(conn, &state, name, shown, base)?);
    }
    main.push_str(&edges_html(&state, "Sources", &shown.proposal.source, base));
    main.push_str(&edges_html(
        &state,
        "The claims it changes",
        &shown.proposal.changes,
        base,
    ));
    main.push_str(&naming_html(&state, claim, base));
    main.push_str(&origin_html(&shown.proposal.origin));
    main.push_str(&included_html(conn, &state, name, base)?);
    main.push_str(&history_html(claim));
    let registered = repositories_of(conn)?;
    let levels = crate::view::claim_levels(&state, Of::Claims.kinds(), &registered);
    let filters = Filters {
        kind: None,
        status: Statuses::Both,
        repository: None,
        level: None,
        q: String::new(),
    };
    let path = format!("{base}{}", CLAIMS.path);
    let page = Shell {
        title: &shown.proposal.statement,
        entry,
        base,
        view: Some(CLAIMS),
        side: Some(levels_side(Of::Claims, &levels, &path, "", &filters)),
        query: "",
    };
    Ok(shell(&page, &main))
}

/// A link to `name@meant`, marked out of date where another version of
/// it is accepted now.
fn edge_link(state: &State, name: &str, meant: &str, base: &str) -> String {
    let mut html = format!(
        "<a href=\"{base}/claims/{at}\"><code>{at}</code></a>",
        at = format!("{}@{}", escape(name), escape(meant))
    );
    let standing = state
        .named(name)
        .first()
        .and_then(|claim| claim.standing.clone());
    if let Some(now) = standing.filter(|now| now != meant) {
        let _ = write!(
            html,
            " · <span class=\"late\">out of date: accepted now {}</span>",
            escape(&now)
        );
    }
    html
}

/// The edges of a version to other claims, under `title`.
fn edges_html(state: &State, title: &str, edges: &[String], base: &str) -> String {
    if edges.is_empty() {
        return String::new();
    }
    let mut html = format!("<h2>{title}</h2>\n<ul class=\"edges\">\n");
    for edge in edges {
        if let Some((name, meant)) = edge.split_once('@') {
            let statement = state
                .holding(name, meant)
                .and_then(|claim| claim.version(meant))
                .map(|version| escape(&version.proposal.statement))
                .unwrap_or_default();
            let _ = writeln!(
                html,
                "<li>{} {statement}</li>",
                edge_link(state, name, meant, base)
            );
        }
    }
    html.push_str("</ul>\n");
    html
}

/// The claims whose accepted or proposed version names `claim`, each
/// with the version meant, marked out of date where it is not the
/// accepted one.
fn naming_html(state: &State, claim: &Claim, base: &str) -> String {
    let naming = crate::view::naming(state, &claim.name);
    if naming.is_empty() {
        return String::new();
    }
    let mut html =
        String::from("<h2>The claims that name it</h2>\n<ul class=\"edges\" id=\"named-by\">\n");
    for found in naming {
        let how = if found.edge == "source" {
            "rests on it"
        } else {
            "changes it"
        };
        let name = escape(&found.claim.name);
        let mut late = String::new();
        if claim
            .standing
            .as_deref()
            .is_some_and(|now| now != found.meant)
        {
            let _ = write!(
                late,
                " · <span class=\"late\">out of date: accepted now {}</span>",
                claim.standing.as_deref().map(escape).unwrap_or_default()
            );
        }
        let _ = writeln!(
            html,
            "<li data-claim=\"{name}\">{kind} <a href=\"{base}/claims/{name}@{v}\"><code>{name}@{v}</code></a> {status} · {how}, on <code>@{meant}</code>{late}</li>",
            kind = found.claim.kind.as_str(),
            v = found.version.proposal.version,
            status = found.version.status.as_str(),
            meant = escape(&found.meant),
        );
    }
    html.push_str("</ul>\n");
    html
}

/// The origin of a version: a kept file by its name, its fingerprint
/// and the command that writes it, and a commit as it is. boom serves no
/// kept file's bytes.
fn origin_html(origins: &[Origin]) -> String {
    if origins.is_empty() {
        return String::new();
    }
    let mut html = String::from("<h2>Origin</h2>\n<ul class=\"edges\">\n");
    for origin in origins {
        match origin {
            Origin::File { name, fingerprint } => {
                let _ = writeln!(
                    html,
                    "<li>{} <code>{fingerprint}</code><br><code>boom show --artifact {fingerprint}</code></li>",
                    escape(name)
                );
            }
            Origin::Git {
                repository,
                commit,
                path,
            } => {
                let _ = writeln!(
                    html,
                    "<li><code>git:{}@{commit}:{}</code></li>",
                    escape(repository),
                    escape(path)
                );
            }
        }
    }
    html.push_str("</ul>\n");
    html
}

/// Each narrative of the book that includes `name`, as a link to the
/// paragraph that names it, and the notes on that paragraph.
fn included_html(
    conn: &rusqlite::Connection,
    state: &State,
    name: &str,
    base: &str,
) -> Result<String, Failure> {
    let book = crate::view::book(state, None);
    let mut html = String::new();
    for entry in book.order() {
        let Some(version) = state
            .named(&entry.narrative)
            .first()
            .and_then(|claim| claim.version(&entry.version))
        else {
            continue;
        };
        let Some(meant) = version
            .proposal
            .includes
            .iter()
            .filter_map(|edge| edge.split_once('@'))
            .find(|(included, _)| *included == name)
            .map(|(_, meant)| meant)
        else {
            continue;
        };
        let body = body_text(conn, version.proposal.body.as_deref())?;
        let paragraph = crate::narrative::paragraphs(&body)
            .into_iter()
            .find(|paragraph| paragraph.names(&body, name))
            .map(|paragraph| paragraph.number);
        let narrative = escape(&entry.narrative);
        let at = paragraph.map(|n| format!("#p{n}")).unwrap_or_default();
        let _ = writeln!(
            html,
            "<li data-narrative=\"{narrative}\"><a href=\"{base}/n/{narrative}{at}\">{}</a> · <code>{narrative}@{}</code> includes <code>@{}</code>{}</li>",
            escape(&entry.title),
            entry.version,
            escape(meant),
            paragraph.map(|n| format!(" at ¶{n}")).unwrap_or_default()
        );
        if let Some(n) = paragraph {
            for note in state
                .notes_of(&entry.narrative)
                .into_iter()
                .filter(|note| note.version == entry.version && note.paragraph == n)
            {
                let _ = writeln!(
                    html,
                    "<li class=\"note-of\"><a href=\"{base}/n/{narrative}#p{n}\">¶{n} of {}</a>{}</li>",
                    escape(&entry.title),
                    note_box(state, note, None)
                );
            }
        }
    }
    if html.is_empty() {
        return Ok(String::new());
    }
    Ok(format!(
        "<h2>Narratives that include it</h2>\n<ul class=\"edges\">\n{html}</ul>\n"
    ))
}

/// A context's globs, and each accepted claim that binds it, by kind.
fn context_html(
    conn: &rusqlite::Connection,
    state: &State,
    name: &str,
    shown: &Version,
    base: &str,
) -> Result<String, Failure> {
    let mut html = String::from("<h2>Globs</h2>\n");
    if shown.proposal.paths.is_empty() {
        html.push_str("<p class=\"hint\">It owns no glob.</p>\n");
    } else {
        html.push_str("<ul class=\"edges globs\">\n");
        for glob in &shown.proposal.paths {
            let _ = writeln!(html, "<li><code>{}</code></li>", escape(glob));
        }
        html.push_str("</ul>\n");
    }
    let bound = store::bound_to(conn, name)?;
    let mut by_kind: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for found in &bound {
        let Some(claim) = state.holding(&found.name, &found.version) else {
            continue;
        };
        let statement = claim
            .version(&found.version)
            .map(|version| escape(&version.proposal.statement))
            .unwrap_or_default();
        let rank = Of::Claims
            .kinds()
            .iter()
            .position(|kind| *kind == claim.kind)
            .unwrap_or(usize::MAX);
        by_kind.entry(rank).or_default().push(format!(
            "<li data-claim=\"{n}\">{} <a href=\"{base}/claims/{n}@{v}\"><code>{n}@{v}</code></a> {statement}</li>",
            claim.kind.as_str(),
            n = escape(&found.name),
            v = found.version,
        ));
    }
    html.push_str("<h2>The accepted claims that bind it</h2>\n");
    if by_kind.is_empty() {
        html.push_str("<p class=\"hint\">No accepted claim binds it.</p>\n");
    } else {
        html.push_str("<ul class=\"edges\">\n");
        for items in by_kind.values() {
            for item in items {
                html.push_str(item);
                html.push('\n');
            }
        }
        html.push_str("</ul>\n");
    }
    Ok(html)
}

/// The entry that closed `version` of `claim`: its rejection; for a
/// version that was accepted, the accept of the next version accepted;
/// for a draft, the proposal of the version that superseded it.
fn closed_in(claim: &Claim, at: usize) -> Option<u64> {
    let version = &claim.versions[at];
    if version.reason.is_some() {
        return version.decided_in;
    }
    let later = &claim.versions[at + 1..];
    if version.decided_by.is_some() {
        later
            .iter()
            .filter(|other| other.decided_by.is_some() && other.reason.is_none())
            .filter_map(|other| other.decided_in)
            .min()
    } else {
        later.first().map(|other| other.proposed_in)
    }
}

/// "History": each version that is neither proposed nor accepted, with
/// its status, the entry that closed it, the person and why.
fn history_html(claim: &Claim) -> String {
    let closed: Vec<String> = claim
        .versions
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, version)| !matches!(version.status, Status::Accepted | Status::Proposed))
        .map(|(at, version)| {
            let mut line = format!(
                "<li data-version=\"{v}\"><code>{v}</code> {}",
                version.status.as_str(),
                v = version.proposal.version
            );
            if let Some(seq) = closed_in(claim, at) {
                let _ = write!(line, " in entry {seq}");
            }
            if let Some(by) = &version.decided_by {
                let _ = write!(
                    line,
                    " · {} by {}",
                    if version.reason.is_some() {
                        "rejected"
                    } else {
                        "accepted"
                    },
                    escape(by)
                );
            }
            if let Some(why) = &version.reason {
                let _ = write!(line, " · why: {}", escape(why));
            }
            line.push_str("</li>");
            line
        })
        .collect();
    if closed.is_empty() {
        return String::new();
    }
    format!(
        "<details class=\"history\"><summary><h2>History</h2></summary>\n<ul class=\"edges\">\n{}\n</ul></details>\n",
        closed.join("\n")
    )
}
