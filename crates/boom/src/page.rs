//! `boom page`: the page where a person reads a narrative with the claims
//! that it includes, and writes a note on a paragraph (narrative-page).
//!
//! Boom serves the page on `127.0.0.1`, each path under a token of 128
//! random bits that only the line boom prints holds. Each response is
//! computed from the views at the log's last entry, and no file holds
//! the page (`views-computed-from-the-ledger`). The page holds its own
//! style, loads nothing from another host and holds no script: a note is
//! a plain HTML form, and each response refuses each script with its
//! policy.

use std::fmt::Write as _;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use pulldown_cmark::{CowStr, Event, Tag, TagEnd};

use crate::claim::Kind;
use crate::cli::Env;
use crate::entry::{Act, Actor, Entry};
use crate::judge::{self, Claim, Proof, State, Verdict, Version};
use crate::narrative::{self, Paragraph};
use crate::store::{self, Scope, Store};
use crate::Failure;

/// The policy of each response: nothing from another host, no script,
/// the style the page holds, and forms sent to the page alone.
pub const POLICY: &str = "default-src 'none'; style-src 'unsafe-inline'; form-action 'self'";

/// The largest note a form sends, in bytes.
const LARGEST_FORM: u64 = 64 * 1024;

/// What `boom page` was asked to serve.
#[derive(Debug, Clone)]
pub struct Options {
    /// The narrative, `<name>` or `<name>@<version>`; none for the list.
    pub target: Option<String>,
    /// The port; 0 for a free one.
    pub port: u16,
    /// Whether to open the URL in the browser.
    pub open: bool,
    /// How long boom serves with no request.
    pub idle: Duration,
    /// The person of the page, `--person`.
    pub person: Option<String>,
}

/// A note written: its name, the person, the paragraph's fingerprint and
/// the entry that holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    /// The note's name.
    pub note: String,
    /// The person who wrote it.
    pub person: String,
    /// The fingerprint of its paragraph.
    pub fingerprint: String,
    /// The entry that holds it, or the one it repeats.
    pub entry: u64,
}

/// The person who writes a note: `given`, registered, or the one person
/// the store registers.
fn person_of(state: &State, given: Option<&str>) -> Result<String, Failure> {
    match given {
        Some(name) if state.persons.contains_key(name) => Ok(name.to_owned()),
        Some(name) => Err(Failure::refused(format!(
            "{name} is not registered in this store: --person names a person it registers"
        ))),
        None => match state.persons.keys().collect::<Vec<_>>().as_slice() {
            [one] => Ok((*one).clone()),
            all => Err(Failure::refused(format!(
                "the store registers {} persons: --person names the person who writes the note",
                all.len()
            ))),
        },
    }
}

/// Writes a person's note on paragraph `paragraph` of `name@version`, the
/// entry that `boom note` and the page append: the person's name and no
/// signature (narrative-notes).
pub fn write_note(
    store: &mut Store,
    name: &str,
    version: &str,
    paragraph: u64,
    words: &str,
    person: Option<&str>,
    env: &Env,
) -> Result<Written, Failure> {
    let state = store.state_for(&Scope::names([name]))?;
    let person = person_of(&state, person)?;
    let Some(claim) = state.holding(name, version) else {
        return Err(Failure::refused(if state.named(name).is_empty() {
            format!("no claim has the name {name}")
        } else {
            format!("the claim {name} has no version {version}")
        }));
    };
    if claim.kind != Kind::Narrative {
        return Err(Failure::refused(format!(
            "a note is on a narrative: {name} is {} {}",
            crate::claim::article(claim.kind),
            claim.kind.as_str()
        )));
    }
    let body = claim
        .version(version)
        .and_then(|found| found.proposal.body.clone())
        .unwrap_or_default();
    let paragraphs = state.paragraphs.get(&body).ok_or_else(|| {
        Failure::not_run(format!(
            "the store does not hold the body of {name}@{version}"
        ))
    })?;
    if paragraph == 0 {
        return Err(Failure::refused(
            "a note names a paragraph by its number, from 1",
        ));
    }
    let Some(fingerprint) = usize::try_from(paragraph - 1)
        .ok()
        .and_then(|at| paragraphs.get(at))
    else {
        return Err(Failure::refused(format!(
            "{name}@{version} has {}: it has no paragraph {paragraph}",
            judge::paragraphs_words(paragraphs.len())
        )));
    };
    let words = words.replace("\r\n", "\n");
    if words.trim().is_empty() {
        return Err(Failure::refused("a note holds words: this one is empty"));
    }
    let entry = Entry {
        at: env.clock.now(),
        actor: Actor::Person(person.clone()),
        act: Act::Noted {
            name: name.to_owned(),
            version: version.to_owned(),
            paragraph,
            fingerprint: fingerprint.clone(),
            words: words.clone(),
        },
    };
    let signed = entry.signed_part();
    let note = narrative::note_name(name, version, paragraph, &words, &person);
    let written = Written {
        note,
        person,
        fingerprint: fingerprint.clone(),
        entry: 0,
    };
    match judge::judge(&state, &entry, &signed, Proof::Unsigned) {
        Verdict::Counts => {}
        Verdict::Refused(why) => return Err(Failure::Refused(why)),
        Verdict::Repeats(seq) => {
            return Ok(Written {
                entry: seq,
                ..written
            })
        }
    }
    let (appended, _) = store.append(&signed, None, &[])?;
    Ok(Written {
        entry: appended.seq(),
        ..written
    })
}

// The HTML

/// Where the page's paths start and who writes its notes.
#[derive(Debug, Clone)]
pub struct Context {
    /// The start of each path, `/<token>`.
    pub base: String,
    /// The person of the page; none where boom knows none, and the page
    /// then gives no form.
    pub person: Option<String>,
}

/// `text` with `&`, `<`, `>`, `"` and `'` written as entities.
pub fn escape(text: &str) -> String {
    let mut written = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => written.push_str("&amp;"),
            '<' => written.push_str("&lt;"),
            '>' => written.push_str("&gt;"),
            '"' => written.push_str("&quot;"),
            '\'' => written.push_str("&#39;"),
            c => written.push(c),
        }
    }
    written
}

/// The HTML of Markdown events: each raw HTML as text, and each image as
/// its words, so a body loads nothing and runs nothing.
fn html_of(events: &[Event<'static>]) -> String {
    let mut image = 0usize;
    let safe = events.iter().cloned().filter_map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Some(Event::Text(raw)),
        Event::Start(Tag::Image { .. }) => {
            image += 1;
            Some(Event::Text(CowStr::Borrowed("[image: ")))
        }
        Event::End(TagEnd::Image) => {
            image = image.saturating_sub(1);
            Some(Event::Text(CowStr::Borrowed("]")))
        }
        Event::Start(Tag::HtmlBlock) | Event::End(TagEnd::HtmlBlock) => None,
        event => Some(event),
    });
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, safe);
    html
}

/// The HTML of a Markdown text, with [`html_of`]'s care.
fn markdown(text: &str) -> String {
    let events: Vec<Event<'static>> =
        pulldown_cmark::Parser::new_ext(text, pulldown_cmark::Options::ENABLE_TABLES)
            .map(Event::into_static)
            .collect();
    html_of(&events)
}

/// A time in Unix seconds as `YYYY-MM-DD HH:MM`, in UTC.
pub fn time_words(at: u64) -> String {
    let days = i64::try_from(at / 86_400).unwrap_or(0);
    let seconds = at % 86_400;
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02} UTC",
        seconds / 3600,
        seconds % 3600 / 60
    )
}

fn actor_words(actor: &Actor) -> String {
    match actor {
        Actor::Person(name) => name.clone(),
        Actor::Agent { provenance } => format!("an agent ({provenance})"),
    }
}

/// The whole document around `main`, with its title and its style.
fn document(title: &str, entry: u64, base: &str, main: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <meta name=\"referrer\" content=\"no-referrer\">\n\
         <title>{} · Boom</title>\n<style>{STYLE}</style>\n</head>\n<body>\n\
         <header class=\"bar\"><a href=\"{base}/\">Boom · narratives</a><span>entry {entry}</span></header>\n\
         <main>\n{main}</main>\n</body>\n</html>\n",
        escape(title)
    )
}

/// The page's style: one column for reading, the number of each
/// paragraph in its margin, and a box for each claim and each note.
const STYLE: &str = "
:root{--ink:#1d1d1f;--soft:#5c5c66;--line:#dcdce2;--paper:#fbfbf8;--card:#fff;--claim:#eef4ff;--claim-edge:#4b6fd6;--note:#fff6e0;--note-edge:#d29a16;--done:#e9f6ec;--done-edge:#3f9a5a;--link:#2f55c4}
@media (prefers-color-scheme: dark){:root{--ink:#e9e9ee;--soft:#a4a4b0;--line:#34343c;--paper:#141418;--card:#1c1c22;--claim:#1b2236;--claim-edge:#7d9bf0;--note:#2e2716;--note-edge:#e0b13e;--done:#18291d;--done-edge:#5cc27b;--link:#8fa9f5}}
*{box-sizing:border-box}
body{margin:0;background:var(--paper);color:var(--ink);font:17px/1.6 ui-serif,Georgia,'Iowan Old Style',serif}
a{color:var(--link)}
code,pre,.mono{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:.86em}
pre{background:var(--card);border:1px solid var(--line);border-radius:6px;padding:.7rem .9rem;overflow-x:auto}
.bar{display:flex;justify-content:space-between;gap:1rem;padding:.6rem 1rem;border-bottom:1px solid var(--line);font:13px/1.4 system-ui,sans-serif;color:var(--soft)}
.bar a{color:var(--soft);text-decoration:none;font-weight:600}
main{max-width:46rem;margin:0 auto;padding:1.5rem 1rem 4rem}
h1{font:600 2rem/1.2 system-ui,sans-serif;margin:.4rem 0 .6rem}
h2{font:600 1.05rem/1.3 system-ui,sans-serif;margin:2.2rem 0 .6rem;padding-top:1rem;border-top:1px solid var(--line)}
.meta,.versions,.hint{font:13px/1.55 system-ui,sans-serif;color:var(--soft)}
.versions{margin:.4rem 0 1.4rem}
.versions .here{font-weight:600;color:var(--ink)}
.status{display:inline-block;padding:0 .4rem;border-radius:4px;border:1px solid var(--line);font:12px/1.5 system-ui,sans-serif}
.p{position:relative;padding:.1rem 0 .1rem 3rem;border-radius:6px;scroll-margin-top:1rem}
.p:target{background:var(--card);box-shadow:0 0 0 1px var(--line)}
.p>.n{position:absolute;left:0;top:.35rem;width:2.5rem;text-align:right;font:12px/1.6 system-ui,sans-serif;color:var(--soft);text-decoration:none}
.p>.text>:first-child{margin-top:.3rem}
.p>.text>:last-child{margin-bottom:.3rem}
details.write{font:13px/1.4 system-ui,sans-serif;margin:0 0 .6rem}
details.write>summary{cursor:pointer;color:var(--soft);list-style:none;display:inline-block}
details.write>summary::before{content:'+ note'}
details.write[open]>summary::before{content:'− note'}
details.write textarea{display:block;width:100%;min-height:5rem;margin:.4rem 0;font:15px/1.5 system-ui,sans-serif;padding:.5rem;border:1px solid var(--line);border-radius:6px;background:var(--card);color:var(--ink)}
details.write button{font:600 13px system-ui,sans-serif;padding:.35rem .8rem;border-radius:6px;border:1px solid var(--note-edge);background:var(--note);color:var(--ink);cursor:pointer}
.box{margin:.4rem 0 .9rem;padding:.55rem .8rem;border-left:3px solid;border-radius:0 6px 6px 0;font:14px/1.5 system-ui,sans-serif}
.box .head{font-size:12px;color:var(--soft);margin-bottom:.2rem}
.box p{margin:.2rem 0}
.claim{background:var(--claim);border-color:var(--claim-edge)}
.claim details>summary{cursor:pointer;color:var(--soft);font-size:12px}
.note{background:var(--note);border-color:var(--note-edge)}
.note.answered{background:var(--done);border-color:var(--done-edge)}
.late{color:#b4452c}
ul.list{list-style:none;padding:0}
ul.list li{padding:.6rem 0;border-bottom:1px solid var(--line)}
ul.list .title{font:600 1.05rem system-ui,sans-serif}
.quote{border-left:2px solid var(--line);padding-left:.6rem;color:var(--soft)}
";

/// The binding of a version in words.
fn binds(version: &Version) -> String {
    crate::context::binds_words(&version.repositories, &version.contexts)
}

/// The page of the version `version` of the narrative `name`, or of its
/// newest where none is given, from the views on `conn`
/// (narrative-page, "What the page gives").
pub fn narrative_page(
    conn: &rusqlite::Connection,
    name: &str,
    version: Option<&str>,
    context: &Context,
) -> Result<String, Failure> {
    let first = store::load_state_for(conn, &Scope::names([name]))?;
    let claim = narrative_claim(&first, name)?;
    let shown = match version {
        Some(version) => claim.version(version).ok_or_else(|| {
            Failure::refused(format!("the narrative {name} has no version {version}"))
        })?,
        None => claim
            .versions
            .last()
            .ok_or_else(|| Failure::not_run(format!("the narrative {name} has no version")))?,
    };
    // The claims it includes are read with it.
    let names = std::iter::once(name.to_owned()).chain(
        shown
            .proposal
            .includes
            .iter()
            .filter_map(|edge| edge.split_once('@'))
            .map(|(name, _)| name.to_owned()),
    );
    let state = store::load_state_for(conn, &Scope::names(names))?;
    let claim = narrative_claim(&state, name)?;
    let (entry, _) = store::head_entry(conn)?;
    let version = shown.proposal.version.clone();
    let shown = claim.version(&version).expect("the version was found");
    let body = body_text(conn, shown.proposal.body.as_deref())?;
    let paragraphs = narrative::paragraphs(&body);
    let base = &context.base;
    let at = format!("{name}@{version}");
    let mut main = String::new();

    // The head: the title, the version, its status, its proposer, its
    // base, what it binds, and each version as a link.
    let _ = write!(
        main,
        "<h1>{}</h1>\n<p class=\"meta\">narrative <code>{}</code> · <span class=\"status\">{}</span> · proposed by {}",
        escape(&shown.proposal.statement),
        escape(&at),
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
            " · amends <a href=\"{base}/n/{name}@{amended}\">{amended}</a>"
        );
    }
    let _ = writeln!(main, " · binds {}</p>", escape(&binds(shown)));
    main.push_str("<p class=\"versions\">versions: ");
    let links: Vec<String> = claim
        .versions
        .iter()
        .rev()
        .map(|other| {
            let v = &other.proposal.version;
            let here = if *v == version { " class=\"here\"" } else { "" };
            format!(
                "<a{here} href=\"{base}/n/{name}@{v}\">{v}</a> {}",
                other.status.as_str()
            )
        })
        .collect();
    main.push_str(&links.join(" · "));
    main.push_str("</p>\n");
    if context.person.is_none() {
        let _ = writeln!(
            main,
            "<p class=\"hint\">To write a note, start the page with the person who writes it: <code>boom page {} --person &lt;name&gt;</code></p>",
            escape(name)
        );
    }

    // Each included claim, after the first paragraph that names it.
    let includes: Vec<(&str, &str)> = shown
        .proposal
        .includes
        .iter()
        .filter_map(|edge| edge.split_once('@'))
        .collect();
    let placed: Vec<Option<u64>> = includes
        .iter()
        .map(|(included, _)| {
            paragraphs
                .iter()
                .find(|paragraph| paragraph.names(&body, included))
                .map(|paragraph| paragraph.number)
        })
        .collect();
    let notes: Vec<&judge::Note> = state
        .notes_of(name)
        .into_iter()
        .filter(|note| note.version == version)
        .collect();

    main.push_str("<article>\n");
    for paragraph in &paragraphs {
        let n = paragraph.number;
        let _ = write!(
            main,
            "<section class=\"p\" id=\"p{n}\"><a class=\"n\" href=\"#p{n}\">¶{n}</a><div class=\"text\">{}</div>",
            html_of(&paragraph.events)
        );
        if let Some(person) = &context.person {
            let _ = write!(
                main,
                "<details class=\"write\"><summary title=\"Write a note on ¶{n} as {person}\"></summary>\
                 <form method=\"post\" action=\"{base}/n/{at}/note\">\
                 <input type=\"hidden\" name=\"paragraph\" value=\"{n}\">\
                 <textarea name=\"words\" required placeholder=\"A note on ¶{n}, as {person}\"></textarea>\
                 <button type=\"submit\">Send note</button></form></details>",
                person = escape(person),
                at = escape(&at),
            );
        }
        for ((included, meant), _) in includes
            .iter()
            .zip(&placed)
            .filter(|(_, place)| **place == Some(n))
        {
            main.push_str(&claim_box(conn, &state, included, meant)?);
        }
        for note in notes.iter().filter(|note| note.paragraph == n) {
            main.push_str(&note_box(&state, note, None));
        }
        main.push_str("</section>\n");
    }
    main.push_str("</article>\n");

    // The end: the claims no paragraph names, the notes of the earlier
    // versions, and the command line of the accept.
    let unplaced: Vec<&(&str, &str)> = includes
        .iter()
        .zip(&placed)
        .filter(|(_, place)| place.is_none())
        .map(|(include, _)| include)
        .collect();
    if !unplaced.is_empty() {
        main.push_str("<h2>Claims it includes that no paragraph names</h2>\n");
        for (included, meant) in unplaced {
            main.push_str(&claim_box(conn, &state, included, meant)?);
        }
    }
    let earlier: Vec<&judge::Note> = state
        .notes_of(name)
        .into_iter()
        .filter(|note| note.version != version)
        .collect();
    if !earlier.is_empty() {
        main.push_str("<h2>Notes on the other versions, and their answers</h2>\n");
        for note in earlier {
            let words = claim
                .version(&note.version)
                .map(|other| body_text(conn, other.proposal.body.as_deref()))
                .transpose()?
                .and_then(|body| {
                    narrative::paragraphs(&body)
                        .into_iter()
                        .find(|paragraph| paragraph.number == note.paragraph)
                        .map(|paragraph: Paragraph| paragraph.words())
                });
            main.push_str(&note_box(&state, note, words.as_deref()));
        }
    }
    if shown.status == judge::Status::Proposed {
        let _ = writeln!(
            main,
            "<h2>To accept</h2>\n<p class=\"hint\">In your terminal, with your key:</p>\n<pre>boom accept {} --key &lt;key&gt;</pre>",
            escape(&at)
        );
    }
    Ok(document(&shown.proposal.statement, entry, base, &main))
}

/// The claim `name` of `state`, a narrative.
fn narrative_claim<'a>(state: &'a State, name: &str) -> Result<&'a Claim, Failure> {
    let claim = state
        .named(name)
        .into_iter()
        .next()
        .ok_or_else(|| Failure::refused(format!("no claim has the name {name}")))?;
    if claim.kind != Kind::Narrative {
        return Err(Failure::refused(format!(
            "boom page gives a narrative: {name} is {} {}",
            crate::claim::article(claim.kind),
            claim.kind.as_str()
        )));
    }
    Ok(claim)
}

fn body_text(conn: &rusqlite::Connection, fingerprint: Option<&str>) -> Result<String, Failure> {
    let Some(fingerprint) = fingerprint else {
        return Ok(String::new());
    };
    String::from_utf8(store::blob(conn, fingerprint)?)
        .map_err(|_| Failure::not_run(format!("the blob {fingerprint} is not text")))
}

/// The box of an included claim: its kind, `<name>@<version>`, its
/// status, its statement and its body, marked out of date where another
/// version of it is accepted.
fn claim_box(
    conn: &rusqlite::Connection,
    state: &State,
    name: &str,
    meant: &str,
) -> Result<String, Failure> {
    let Some((claim, version)) = state
        .holding(name, meant)
        .and_then(|claim| Some((claim, claim.version(meant)?)))
    else {
        return Ok(format!(
            "<div class=\"box claim\"><div class=\"head\">{}@{} is not in the store</div></div>\n",
            escape(name),
            escape(meant)
        ));
    };
    let mut html = format!(
        "<div class=\"box claim\"><div class=\"head\">{} <code>{}@{}</code> · {}",
        claim.kind.as_str(),
        escape(name),
        escape(meant),
        version.status.as_str()
    );
    if let Some(accepted) = claim.standing.as_deref().filter(|now| *now != meant) {
        let _ = write!(
            html,
            " · <span class=\"late\">out of date: accepted now {}</span>",
            escape(accepted)
        );
    }
    let _ = write!(html, "</div><p>{}</p>", escape(&version.proposal.statement));
    if version.proposal.body.is_some() {
        let body = body_text(conn, version.proposal.body.as_deref())?;
        let _ = write!(
            html,
            "<details><summary>details</summary>{}</details>",
            markdown(&body)
        );
    }
    html.push_str("</div>\n");
    Ok(html)
}

/// The box of a note: its name, its person, its time, its words, and
/// `open` or its answer; for a note on another version, its version, its
/// paragraph and that paragraph's words.
fn note_box(state: &State, note: &judge::Note, paragraph: Option<&str>) -> String {
    let answer = state.answer_of(note);
    let class = if answer.is_some() {
        "box note answered"
    } else {
        "box note"
    };
    let mut html = format!(
        "<div class=\"{class}\" id=\"note-{}\"><div class=\"head\">note {} · {} · {}",
        note.name,
        note.name,
        escape(&note.person),
        time_words(note.at)
    );
    if paragraph.is_some() {
        let _ = write!(html, " · on ¶{} of {}", note.paragraph, note.version);
    }
    match answer {
        Some(_) => html.push_str(" · answered</div>"),
        None => html.push_str(" · open</div>"),
    }
    if let Some(words) = paragraph {
        let _ = write!(html, "<p class=\"quote\">{}</p>", escape(words));
    }
    let _ = write!(html, "<p>{}</p>", escape(&note.words).replace('\n', "<br>"));
    if let Some((by, words)) = answer {
        let _ = write!(
            html,
            "<p><strong>answered by {}:</strong> {}</p>",
            by.proposal.version,
            escape(words)
        );
    }
    html.push_str("</div>\n");
    html
}

/// The list of the narratives of the store, by what each binds, each
/// with its status and its count of open notes (narrative-page, "The
/// list of narratives").
pub fn list_page(conn: &rusqlite::Connection, context: &Context) -> Result<String, Failure> {
    let state = store::narratives(conn)?;
    let (entry, _) = store::head_entry(conn)?;
    let base = &context.base;
    let mut groups: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for claim in state
        .claims
        .values()
        .filter(|claim| claim.kind == Kind::Narrative)
    {
        let Some(newest) = claim.versions.last() else {
            continue;
        };
        let shown = claim
            .standing
            .as_deref()
            .and_then(|standing| claim.version(standing))
            .unwrap_or(newest);
        let open = state.open_notes(&claim.name).len();
        let open_words = match open {
            0 => "no open note".to_owned(),
            1 => "1 open note".to_owned(),
            n => format!("{n} open notes"),
        };
        let item = format!(
            "<li><a class=\"title\" href=\"{base}/n/{name}\">{title}</a><div class=\"meta\"><code>{name}@{version}</code> · <span class=\"status\">{status}</span> · <span class=\"open\" data-open=\"{open}\">{open_words}</span></div></li>\n",
            name = escape(&claim.name),
            title = escape(&shown.proposal.statement),
            version = shown.proposal.version,
            status = shown.status.as_str(),
        );
        let repositories = if shown.repositories.is_empty() && shown.contexts.is_empty() {
            vec!["the whole system".to_owned()]
        } else if shown.repositories.is_empty() {
            crate::context::names(&shown.contexts)
                .into_iter()
                .map(|name| format!("the context {name}"))
                .collect()
        } else {
            shown.repositories.clone()
        };
        for repository in repositories {
            groups.entry(repository).or_default().push(item.clone());
        }
    }
    let mut main = String::from("<h1>Narratives</h1>\n");
    if groups.is_empty() {
        main.push_str("<p class=\"hint\">This store holds no narrative: an agent proposes one with <code>boom propose narrative &lt;name&gt; \"&lt;title&gt;\" --body &lt;file&gt;</code>.</p>\n");
    }
    for (repository, items) in groups {
        let _ = writeln!(
            main,
            "<h2>{}</h2>\n<ul class=\"list\">\n{}</ul>",
            escape(&repository),
            items.concat()
        );
    }
    Ok(document("Narratives", entry, base, &main))
}

// The server

/// 128 random bits from the random source of the operating system, as 32
/// hexadecimal characters.
fn token() -> Result<String, Failure> {
    let mut bits = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(&mut bits))
        .map_err(|error| {
            Failure::not_run(format!(
                "boom page could not read the random source of the operating system: {error}"
            ))
        })?;
    Ok(crate::hex(&bits))
}

/// A form's fields, `application/x-www-form-urlencoded`.
fn form(body: &str) -> Vec<(String, String)> {
    body.split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            (decoded(name), decoded(value))
        })
        .collect()
}

/// `text` with each `+` a space and each `%XX` its byte.
fn decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'+' => out.push(b' '),
            b'%' => {
                let byte = bytes
                    .get(at + 1..at + 3)
                    .filter(|hex| hex.iter().all(u8::is_ascii_hexdigit))
                    .and_then(|hex| std::str::from_utf8(hex).ok())
                    .and_then(|hex| u8::from_str_radix(hex, 16).ok());
                if let Some(byte) = byte {
                    out.push(byte);
                    at += 3;
                    continue;
                }
                out.push(b'%');
            }
            byte => out.push(byte),
        }
        at += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A response: its status, its HTML or none, and a place to go to.
struct Reply {
    status: u16,
    html: String,
    location: Option<String>,
}

impl Reply {
    fn page(html: String) -> Reply {
        Reply {
            status: 200,
            html,
            location: None,
        }
    }

    fn words(status: u16, words: &str) -> Reply {
        Reply {
            status,
            html: format!(
                "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><title>Boom</title></head><body><p>{}</p></body></html>\n",
                escape(words)
            ),
            location: None,
        }
    }
}

/// The server of one run of `boom page`.
struct Server {
    path: PathBuf,
    port: u16,
    context: Context,
    env: Env,
}

impl Server {
    /// The answer to one request: a page computed from the views at the
    /// log's last entry, a note written, or a refusal.
    fn answer(&self, method: &str, url: &str, headers: &[(String, String)], body: &[u8]) -> Reply {
        let base = &self.context.base;
        let path = url.split(['?', '#']).next().unwrap_or_default();
        let Some(rest) = path.strip_prefix(base.as_str()) else {
            return Reply::words(404, "Not found.");
        };
        if !rest.is_empty() && !rest.starts_with('/') {
            return Reply::words(404, "Not found.");
        }
        let header = |name: &str| {
            headers
                .iter()
                .find(|(field, _)| field.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.as_str())
        };
        if header("host") != Some(format!("127.0.0.1:{}", self.port).as_str()) {
            return Reply::words(403, "This page answers only at 127.0.0.1.");
        }
        let rest = rest.trim_start_matches('/');
        match (method, rest.strip_prefix("n/")) {
            ("GET" | "HEAD", None) if rest.is_empty() => {
                self.read(|conn| list_page(conn, &self.context))
            }
            ("GET" | "HEAD", Some(target)) if !target.contains('/') => {
                let (name, version) = match target.split_once('@') {
                    Some((name, version)) => (name, Some(version)),
                    None => (target, None),
                };
                self.read(|conn| narrative_page(conn, name, version, &self.context))
            }
            ("POST", Some(target)) => {
                if header("origin") != Some(format!("http://127.0.0.1:{}", self.port).as_str()) {
                    return Reply::words(403, "A note is sent from this page alone.");
                }
                let Some(target) = target.strip_suffix("/note") else {
                    return Reply::words(404, "Not found.");
                };
                self.note(target, body)
            }
            _ => Reply::words(404, "Not found."),
        }
    }

    fn read(&self, page: impl FnOnce(&rusqlite::Connection) -> Result<String, Failure>) -> Reply {
        match Store::open(&self.path).and_then(|mut store| store.read(page)) {
            Ok(html) => Reply::page(html),
            Err(Failure::Refused(why)) => Reply::words(404, &why),
            Err(Failure::NotRun(why)) => Reply::words(500, &why),
        }
    }

    fn note(&self, target: &str, body: &[u8]) -> Reply {
        let Some(person) = self.context.person.as_deref() else {
            return Reply::words(
                403,
                "boom knows no person of this page: start it with --person <name>",
            );
        };
        let Some((name, version)) = target.split_once('@') else {
            return Reply::words(404, "A note is on a version, <name>@<version>.");
        };
        let fields = form(&String::from_utf8_lossy(body));
        let field = |wanted: &str| {
            fields
                .iter()
                .find(|(name, _)| name == wanted)
                .map(|(_, value)| value.as_str())
        };
        let Some(paragraph) = field("paragraph").and_then(|n| n.parse::<u64>().ok()) else {
            return Reply::words(400, "A note names its paragraph by its number.");
        };
        let words = field("words").unwrap_or_default();
        let written = Store::open(&self.path).and_then(|mut store| {
            write_note(
                &mut store,
                name,
                version,
                paragraph,
                words,
                Some(person),
                &self.env,
            )
        });
        match written {
            Ok(_) => Reply {
                status: 303,
                html: String::new(),
                location: Some(format!(
                    "{}/n/{name}@{version}#p{paragraph}",
                    self.context.base
                )),
            },
            Err(Failure::Refused(why)) => {
                Reply::words(400, &format!("The note was not written: {why}"))
            }
            Err(Failure::NotRun(why)) => {
                Reply::words(500, &format!("The note was not written: {why}"))
            }
        }
    }
}

/// The headers of each response.
fn headers(reply: &Reply) -> Vec<tiny_http::Header> {
    let mut fields = vec![
        ("Content-Type", "text/html; charset=utf-8".to_owned()),
        ("Content-Security-Policy", POLICY.to_owned()),
        ("Cache-Control", "no-store".to_owned()),
        ("Referrer-Policy", "no-referrer".to_owned()),
        ("X-Content-Type-Options", "nosniff".to_owned()),
    ];
    if let Some(location) = &reply.location {
        fields.push(("Location", location.clone()));
    }
    fields
        .into_iter()
        .filter_map(|(name, value)| tiny_http::Header::from_bytes(name, value).ok())
        .collect()
}

/// `boom page`: checks what it serves, binds `127.0.0.1`, prints its
/// line, opens the URL unless asked not to, and serves until `idle`
/// passes with no request (narrative-page).
pub fn serve(path: &Path, options: &Options, env: &Env) -> Result<(), Failure> {
    let mut store = Store::open(path)?;
    let name = options
        .target
        .as_deref()
        .map(|target| target.split_once('@').map_or(target, |(name, _)| name));
    let state = store.state_for(&Scope::names(name))?;
    let shown = match &options.target {
        Some(target) => {
            let (name, version) = match target.split_once('@') {
                Some((name, version)) => (name, Some(version)),
                None => (target.as_str(), None),
            };
            let claim = narrative_claim(&state, name)?;
            let version = match version {
                Some(version) => claim
                    .version(version)
                    .ok_or_else(|| {
                        Failure::refused(format!("the narrative {name} has no version {version}"))
                    })?
                    .proposal
                    .version
                    .clone(),
                None => claim
                    .versions
                    .last()
                    .map(|version| version.proposal.version.clone())
                    .unwrap_or_default(),
            };
            format!("{name}@{version}")
        }
        None => "the narratives of the store".to_owned(),
    };
    let person = match options.person.as_deref() {
        Some(given) => Some(person_of(&state, Some(given))?),
        None => person_of(&state, None).ok(),
    };
    drop(store);
    let server = tiny_http::Server::http(("127.0.0.1", options.port)).map_err(|error| {
        Failure::not_run(format!(
            "boom page could not serve on 127.0.0.1:{}: {error}",
            options.port
        ))
    })?;
    let port = server
        .server_addr()
        .to_ip()
        .map(|address| address.port())
        .ok_or_else(|| Failure::not_run("boom page serves on no port of 127.0.0.1"))?;
    let base = format!("/{}", token()?);
    let first = match &options.target {
        Some(target) => format!("{base}/n/{target}"),
        None => format!("{base}/"),
    };
    let url = format!("http://127.0.0.1:{port}{first}");
    let mut stdout = std::io::stdout();
    let _ = writeln!(stdout, "Boom: page of {shown} at {url}");
    let _ = stdout.flush();
    if options.open {
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        let opened = std::process::Command::new(opener)
            .arg(&url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if !opened.is_ok_and(|status| status.success()) {
            eprintln!("boom: {opener} did not open the page: open the URL above in a browser");
        }
    }
    let served = Server {
        path: path.to_owned(),
        port,
        context: Context { base, person },
        env: env.clone(),
    };
    let mut last = Instant::now();
    loop {
        let left = options.idle.saturating_sub(last.elapsed());
        if left.is_zero() {
            eprintln!(
                "boom page stopped: {} minutes passed with no request",
                options.idle.as_secs() / 60
            );
            return Ok(());
        }
        let Some(mut request) = server
            .recv_timeout(left)
            .map_err(|error| Failure::not_run(format!("boom page could not serve: {error}")))?
        else {
            continue;
        };
        last = Instant::now();
        let method = request.method().as_str().to_owned();
        let url = request.url().to_owned();
        let fields: Vec<(String, String)> = request
            .headers()
            .iter()
            .map(|header| {
                (
                    header.field.as_str().as_str().to_owned(),
                    header.value.as_str().to_owned(),
                )
            })
            .collect();
        let mut body = Vec::new();
        if method == "POST" {
            let _ = request
                .as_reader()
                .take(LARGEST_FORM)
                .read_to_end(&mut body);
        }
        let reply = served.answer(&method, &url, &fields, &body);
        let response =
            tiny_http::Response::from_string(reply.html.clone()).with_status_code(reply.status);
        let mut response = response;
        for header in headers(&reply) {
            response.add_header(header);
        }
        let _ = request.respond(response);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_form_is_decoded() {
        assert_eq!(
            form("paragraph=3&words=Say+what+a+drive+is%2E%0A%C3%A9"),
            [
                ("paragraph".to_owned(), "3".to_owned()),
                ("words".to_owned(), "Say what a drive is.\né".to_owned())
            ]
        );
        assert_eq!(decoded("100%"), "100%");
        assert_eq!(decoded("%zz"), "%zz");
    }

    #[test]
    fn a_time_in_words() {
        assert_eq!(time_words(0), "1970-01-01 00:00 UTC");
        assert_eq!(time_words(1_763_424_000), "2025-11-18 00:00 UTC");
    }

    #[test]
    fn raw_html_is_text() {
        let html = markdown(
            "<script>alert(1)</script>\n\nA <b>bold</b> word ![x](https://example.com/a.png).\n",
        );
        assert!(!html.contains("<script>"), "{html}");
        assert!(
            html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
            "{html}"
        );
        assert!(!html.contains("example.com"), "{html}");
    }
}
