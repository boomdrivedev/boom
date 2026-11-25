//! `boom page`: the page where a person reads a narrative with the claims
//! that it includes, and writes a note on a paragraph, and the page of the
//! design book, which links to the page of each narrative
//! (narrative-page; views).
//!
//! Boom serves the page on `127.0.0.1`, each path under a token of 128
//! random bits that only the line boom prints holds. Each response is
//! computed from the views at the log's last entry, and no file holds
//! the page (`views-computed-from-the-ledger`). The page holds its own
//! style and loads nothing from another host. A note is a plain HTML
//! form. Its one script is Mermaid's, which boom holds in its binary and
//! serves under the token, loaded only by a page that draws a diagram;
//! each response refuses each other script with its policy.

use std::borrow::Cow;
use std::fmt::Write as _;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

use crate::claim::Kind;
use crate::cli::Env;
use crate::entry::{Act, Actor, Entry};
use crate::judge::{self, Claim, Proof, State, Verdict, Version};
use crate::narrative::{self, Paragraph};
use crate::store::{self, Scope, Store};
use crate::view::Book;
use crate::Failure;

mod claims;
mod waiting;

/// The policy of each response: nothing from another host, no script
/// but the page's own origin's, which serves Mermaid's alone, the style
/// the page holds, and forms sent to the page alone.
pub const POLICY: &str =
    "default-src 'none'; script-src 'self'; style-src 'unsafe-inline'; form-action 'self'";

/// The referrer policy of each response and of each page: a request to
/// another host gets no URL of the page, so no token leaves it, and the
/// page's own form keeps its origin, which a `POST` of a note must name.
/// Under `no-referrer` a browser sends `Origin: null` with that form,
/// which boom refuses.
pub const REFERRER_POLICY: &str = "same-origin";

/// The version of Mermaid that boom holds: the package `mermaid` at this
/// version gives `crates/boom/assets/mermaid/` (narrative-page, "boom
/// serves Mermaid's script from itself").
pub const MERMAID_VERSION: &str = "11.17.2";

/// The SHA-256 of [`MERMAID_SCRIPT`], which a test holds it to.
pub const MERMAID_SHA256: &str = "581ed7d74bd9048d0e3a91363927d72ef22942d7722546b27f7cc29e35390eb8";

/// Mermaid's script, `dist/mermaid.min.js` of its package, byte for byte.
pub const MERMAID_SCRIPT: &[u8] = include_bytes!("../assets/mermaid/mermaid.min.js");

/// Mermaid's license, `LICENSE` of its package, byte for byte.
pub const MERMAID_LICENSE: &[u8] = include_bytes!("../assets/mermaid/LICENSE");

/// The mark of a diagram in the page's HTML. Only [`html_of`] writes it:
/// a body's raw HTML is text, so a page that holds it draws a diagram.
const DIAGRAM: &str = "<pre class=\"mermaid\">";

/// The path of Mermaid's script under the token.
fn script_path() -> String {
    format!("mermaid-{MERMAID_VERSION}.min.js")
}

/// The path of Mermaid's license under the token.
fn license_path() -> String {
    format!("mermaid-{MERMAID_VERSION}.LICENSE")
}

/// The largest note a form sends, in bytes.
const LARGEST_FORM: u64 = 64 * 1024;

/// What `boom page` was asked to serve.
#[derive(Debug, Clone)]
pub struct Options {
    /// The narrative, `<name>` or `<name>@<version>`; none for a view.
    pub target: Option<String>,
    /// The outline whose view is served; with no target and no outline,
    /// the design book.
    pub view: Option<String>,
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
        rules: crate::judge::RULES,
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

/// Whether a fenced code block's info string names Mermaid's language.
fn is_diagram(kind: &CodeBlockKind<'_>) -> bool {
    matches!(kind, CodeBlockKind::Fenced(info) if info.split_whitespace().next() == Some("mermaid"))
}

/// The HTML of Markdown events: each raw HTML as text, each image as its
/// words, and each `mermaid` code block as [`DIAGRAM`] with its source as
/// text, so a body loads nothing and runs nothing.
fn html_of(events: &[Event<'static>]) -> String {
    let mut image = 0usize;
    let mut diagram = false;
    let safe = events.iter().cloned().filter_map(|event| match event {
        Event::Start(Tag::CodeBlock(kind)) if is_diagram(&kind) => {
            diagram = true;
            Some(Event::Html(CowStr::Borrowed(DIAGRAM)))
        }
        Event::End(TagEnd::CodeBlock) if diagram => {
            diagram = false;
            Some(Event::Html(CowStr::Borrowed("</pre>\n")))
        }
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

/// A view of `boom page`: its words in the bar, and the path of its
/// first page under the token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct View {
    /// Its words in the bar.
    pub words: &'static str,
    /// The path of its first page, after `<base>`.
    pub path: &'static str,
}

/// The book: `<base>/`, `<base>/v/<outline>` and `<base>/n/<name>`.
pub const BOOK: View = View {
    words: "The book",
    path: "/",
};

/// The rules: each accepted invariant and assumption, by level
/// (claims-view).
pub const RULES: View = View {
    words: "The rules",
    path: "/rules",
};

/// The decisions: each accepted decision, by level, in the order accepted
/// (claims-view).
pub const DECISIONS: View = View {
    words: "The decisions",
    path: "/decisions",
};

/// The terms: each accepted term, by repository (claims-view).
pub const TERMS: View = View {
    words: "The terms",
    path: "/terms",
};

/// The claims: `<base>/claims/` and `<base>/claims/<name>[@<version>]`.
pub const CLAIMS: View = View {
    words: "The claims",
    path: "/claims/",
};

/// What waits on the person: `<base>/waiting`, each batch with its one
/// command (narrative-page, "The view \"Waiting on you\"").
pub const WAITING: View = View {
    words: "Waiting on you",
    path: "/waiting",
};

/// Each view that boom serves, in the bar's order (narrative-page, "One
/// shell for each view"; claims-view, "The views in the bar"). A later
/// view adds itself here and its routes.
pub const VIEWS: &[View] = &[WAITING, BOOK, RULES, DECISIONS, TERMS, CLAIMS];

/// The number of batches that wait on a person, which the bar gives.
fn waiting_count(conn: &rusqlite::Connection) -> Result<usize, Failure> {
    Ok(crate::batch::waiting(conn)?.len())
}

/// What the shell of a page holds besides its `<main>`.
struct Shell<'a> {
    /// The page's title.
    title: &'a str,
    /// The log's last entry.
    entry: u64,
    /// The start of each path.
    base: &'a str,
    /// The view of the page; none for the page of a search.
    view: Option<View>,
    /// The HTML of the view's sidebar; none where the view gives none.
    side: Option<String>,
    /// The words of the search, given back in its field.
    query: &'a str,
    /// The number of batches that wait on a person.
    waiting: usize,
}

/// The whole document around `main`: the bar with each view, the search
/// form and the log's entry; the control that folds the sidebar and the
/// sidebar, where the view gives one; `<main>`; and Mermaid's script and
/// license where `main` draws a diagram (narrative-page, "One shell for
/// each view").
fn shell(page: &Shell<'_>, main: &str) -> String {
    let base = page.base;
    let mermaid = if main.contains(DIAGRAM) {
        format!(
            "<footer class=\"hint\">Diagrams drawn by Mermaid {MERMAID_VERSION}, under its <a href=\"{base}/{}\">license</a>.</footer>\n\
             <script src=\"{base}/{}\"></script>\n",
            license_path(),
            script_path()
        )
    } else {
        String::new()
    };
    let views: String = VIEWS
        .iter()
        .map(|view| {
            let current = if page.view == Some(*view) {
                " aria-current=\"page\""
            } else {
                ""
            };
            let count = if *view == WAITING && page.waiting > 0 {
                format!("<span class=\"waits\">{}</span>", page.waiting)
            } else {
                String::new()
            };
            format!(
                "<a href=\"{base}{}\"{current}>{}{count}</a>",
                view.path, view.words
            )
        })
        .collect();
    let (control, side) = match &page.side {
        Some(side) => (
            "<input type=\"checkbox\" id=\"contents\" class=\"fold\">\
             <label for=\"contents\" class=\"fold\">Contents</label>\n",
            format!("<nav class=\"side\" aria-label=\"Contents\">\n{side}</nav>\n"),
        ),
        None => ("", String::new()),
    };
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <meta name=\"referrer\" content=\"{REFERRER_POLICY}\">\n\
         <title>{title} · Boom</title>\n<style>{STYLE}</style>\n</head>\n<body>\n\
         <header class=\"bar\"><span class=\"name\">Boom</span><nav class=\"views\" aria-label=\"Views\">{views}</nav>\
         <form class=\"search\" role=\"search\" method=\"get\" action=\"{base}/search\">\
         <input type=\"search\" name=\"q\" value=\"{query}\" placeholder=\"Search the book and the claims\" aria-label=\"Search the book and the claims\">\
         <button type=\"submit\">Search</button></form>\
         <span class=\"entry\">entry {entry}</span></header>\n\
         {control}<div class=\"page\">\n{side}<main>\n{main}</main>\n</div>\n{mermaid}</body>\n</html>\n",
        title = escape(page.title),
        query = escape(page.query),
        entry = page.entry,
    )
}

/// The page's style: the bar, the sidebar beside the page from 60rem
/// and folded behind its control under it, one column for reading, the
/// number of each paragraph in its margin, and a box for each claim and
/// each note.
const STYLE: &str = "
:root{--ink:#1d1d1f;--soft:#5c5c66;--line:#dcdce2;--paper:#fbfbf8;--card:#fff;--claim:#eef4ff;--claim-edge:#4b6fd6;--note:#fff6e0;--note-edge:#d29a16;--done:#e9f6ec;--done-edge:#3f9a5a;--link:#2f55c4}
@media (prefers-color-scheme: dark){:root{--ink:#e9e9ee;--soft:#a4a4b0;--line:#34343c;--paper:#141418;--card:#1c1c22;--claim:#1b2236;--claim-edge:#7d9bf0;--note:#2e2716;--note-edge:#e0b13e;--done:#18291d;--done-edge:#5cc27b;--link:#8fa9f5}}
*{box-sizing:border-box}
body{margin:0;background:var(--paper);color:var(--ink);font:17px/1.6 ui-serif,Georgia,'Iowan Old Style',serif}
a{color:var(--link)}
code,pre,.mono{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:.86em}
pre{background:var(--card);border:1px solid var(--line);border-radius:6px;padding:.7rem .9rem;overflow-x:auto}
.bar{display:flex;flex-wrap:wrap;align-items:center;gap:.5rem 1.2rem;padding:.6rem 1rem;border-bottom:1px solid var(--line);font:13px/1.4 system-ui,sans-serif;color:var(--soft)}
.bar .name{font-weight:700;color:var(--ink)}
.bar .views{display:flex;flex-wrap:wrap;gap:.2rem 1rem}
.bar .views a{white-space:nowrap}
.bar .views a{color:var(--soft);text-decoration:none;font-weight:600;padding:.15rem 0;border-bottom:2px solid transparent}
.bar .views a[aria-current]{color:var(--ink);border-bottom-color:var(--claim-edge)}
.bar .entry{margin-left:auto}
.bar form{display:flex;gap:.4rem;order:3;flex:1 1 100%}
.bar input[type=search]{flex:1;min-width:0;font:14px/1.4 system-ui,sans-serif;padding:.3rem .55rem;border:1px solid var(--line);border-radius:6px;background:var(--card);color:var(--ink)}
.bar button{font:600 13px system-ui,sans-serif;padding:.3rem .7rem;border:1px solid var(--line);border-radius:6px;background:var(--card);color:var(--ink);cursor:pointer}
input.fold{position:absolute;opacity:0;width:1px;height:1px}
label.fold{display:inline-block;margin:.7rem 1rem 0;padding:.25rem .7rem;border:1px solid var(--line);border-radius:6px;background:var(--card);font:600 13px/1.4 system-ui,sans-serif;color:var(--ink);cursor:pointer}
label.fold::before{content:'☰ '}
input.fold:focus-visible+label.fold{outline:2px solid var(--link);outline-offset:2px}
nav.side{display:none;padding:.8rem 1rem 1.2rem;border-bottom:1px solid var(--line);font:14px/1.45 system-ui,sans-serif}
input.fold:checked~.page>nav.side{display:block}
nav.side .book-title{display:block;font-weight:700;color:var(--ink);text-decoration:none;margin:0 0 .6rem}
nav.side .part{margin:1rem 0 .3rem;font-size:12px;font-weight:700;letter-spacing:.02em;text-transform:uppercase;color:var(--soft)}
nav.side .chapter{margin:.7rem 0 .2rem;font-weight:600;color:var(--ink)}
nav.side .section{margin:.5rem 0 .15rem .7rem;font-size:13px;color:var(--soft)}
nav.side ul{list-style:none;margin:0;padding:0}
nav.side li{margin:0}
nav.side li a{display:block;padding:.2rem .5rem;border-radius:5px;color:var(--ink);text-decoration:none}
nav.side .in li a,nav.side li.in>a{padding-left:1.2rem}
nav.side li a:hover{background:var(--card)}
nav.side li a[aria-current]{background:var(--claim);box-shadow:inset 3px 0 0 var(--claim-edge);font-weight:600}
nav.side .unheld{display:block;font-size:11px}
nav.side li a .n{float:right;margin-left:.5rem;color:var(--soft);font-size:12px}
main{max-width:46rem;margin:0 auto;padding:1.5rem 1rem 4rem;min-width:0}
@media (min-width:60rem){
.bar{flex-wrap:nowrap}
.bar form{order:0;flex:0 1 24rem;margin-left:auto}
.bar .entry{margin-left:0}
input.fold,label.fold{display:none}
.page{display:grid;grid-template-columns:17rem minmax(0,1fr);align-items:start}
nav.side{display:block;position:sticky;top:0;max-height:100vh;overflow-y:auto;border-bottom:0;border-right:1px solid var(--line);padding:1.2rem 1rem 2rem}
}
h1{font:600 2rem/1.2 system-ui,sans-serif;margin:.4rem 0 .6rem}
h2{font:600 1.05rem/1.3 system-ui,sans-serif;margin:2.2rem 0 .6rem;padding-top:1rem;border-top:1px solid var(--line)}
.meta,.versions,.hint{font:13px/1.55 system-ui,sans-serif;color:var(--soft)}
.versions{margin:.4rem 0 1.4rem}
.versions .here{font-weight:600;color:var(--ink)}
.status{display:inline-block;padding:0 .4rem;border-radius:4px;border:1px solid var(--line);font:12px/1.5 system-ui,sans-serif}
.p{position:relative;padding:.1rem 0 .1rem 3rem;border-radius:6px;scroll-margin-top:1rem}
.p:target{background:var(--card);box-shadow:0 0 0 1px var(--line)}
.p>.n{position:absolute;left:0;top:.35rem;width:2.5rem;text-align:right;font:12px/1.6 system-ui,sans-serif;color:var(--soft);text-decoration:none}
.p>.text{overflow-x:auto}
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
.unheld{color:#b4452c;font:12px/1.5 system-ui,sans-serif}
h3{font:600 .98rem/1.3 system-ui,sans-serif;margin:1.6rem 0 .3rem}
h4{font:600 .9rem/1.3 system-ui,sans-serif;margin:1.2rem 0 .2rem;color:var(--soft)}
nav.book{display:flex;justify-content:space-between;gap:1rem;margin:2.4rem 0 0;padding-top:1rem;border-top:1px solid var(--line);font:13px/1.5 system-ui,sans-serif}
.quote{border-left:2px solid var(--line);padding-left:.6rem;color:var(--soft)}
.found .quote{margin:.3rem 0 0;font:14px/1.5 system-ui,sans-serif;display:-webkit-box;-webkit-line-clamp:3;-webkit-box-orient:vertical;overflow:hidden}
.box .head a{color:inherit}
form.filters{display:flex;flex-wrap:wrap;gap:.5rem .8rem;align-items:end;margin:1rem 0;padding:.7rem .8rem;border:1px solid var(--line);border-radius:8px;background:var(--card);font:13px/1.4 system-ui,sans-serif;color:var(--soft)}
form.filters label{display:flex;flex-direction:column;gap:.2rem}
form.filters select,form.filters input{font:14px/1.3 system-ui,sans-serif;padding:.25rem .4rem;border:1px solid var(--line);border-radius:5px;background:var(--paper);color:var(--ink)}
form.filters button{font:600 13px system-ui,sans-serif;padding:.35rem .8rem;border:1px solid var(--claim-edge);border-radius:6px;background:var(--claim);color:var(--ink);cursor:pointer}
.count{display:inline-block;min-width:1.4rem;margin-left:.3rem;padding:0 .35rem;border-radius:9px;background:var(--card);border:1px solid var(--line);font-size:11px;text-align:center;color:var(--soft)}
.item{margin:.5rem 0 .8rem;padding:.55rem .8rem;border-left:3px solid var(--claim-edge);border-radius:0 6px 6px 0;background:var(--claim);font:14px/1.5 system-ui,sans-serif}
.item p{margin:.2rem 0}
.item details>summary{cursor:pointer;color:var(--soft);font-size:12px}
.item.proposed,.waiting.frame{background:transparent;border:1.5px dashed var(--note-edge);border-radius:6px}
.mark{display:inline-block;padding:0 .4rem;border-radius:4px;background:var(--note);border:1px solid var(--note-edge);font:600 11px/1.6 system-ui,sans-serif;text-transform:uppercase;letter-spacing:.03em;color:var(--ink)}
.waiting.frame{padding:.6rem .9rem;margin:1rem 0;font:14px/1.5 system-ui,sans-serif}
section.waiting{margin:1.4rem 0 2rem;padding:.2rem 1rem 1rem;border:1px solid var(--note-edge);border-radius:8px;background:color-mix(in srgb,var(--note) 45%,transparent)}
section.waiting h2{border-top:0;margin-top:.8rem}
pre.command{margin:.4rem 0;font-size:12.5px;white-space:pre-wrap;word-break:break-all}
h3.group{font-size:.9rem;color:var(--soft)}
ul.edges{padding-left:1.1rem;font:14px/1.55 system-ui,sans-serif}
ul.edges li{margin:.3rem 0}
ul.edges li.note-of{list-style:none}
details.history>summary{cursor:pointer;list-style:none}
details.history>summary h2{display:inline-block}
.meta.globs{margin-top:-.1rem}
.p table,.box table{border-collapse:collapse;margin:.5rem 0;font:15px/1.45 system-ui,sans-serif}
.p th,.p td,.box th,.box td{border:1px solid var(--line);padding:.35rem .7rem;text-align:left;vertical-align:top}
.p th,.box th{background:var(--card);font-weight:600}
pre.mermaid{background:#fff;color:#1d1d1f;border-color:#dcdce2}
pre.mermaid[data-processed]{font:inherit;text-align:center}
pre.mermaid svg{max-width:100%;height:auto}
footer.hint{max-width:46rem;margin:0 auto;padding:0 1rem 2rem}
.bar .views .waits{display:inline-block;margin-left:.3rem;padding:0 .35rem;border-radius:9px;background:var(--note-edge);color:#fff;font-size:11px}
.how,.total{margin:.4rem 0 1rem;padding:.6rem .8rem;border-radius:6px;border:1px solid var(--line);background:var(--card);font:14px/1.5 system-ui,sans-serif}
.how p{margin:0}.how ul{margin:.3rem 0 0;padding-left:1.2rem}
.total{background:var(--done);border-color:var(--done-edge)}
section.wait{margin:1rem 0;padding:.9rem 1rem;border:1.5px dashed var(--claim-edge);border-radius:8px;background:var(--card);scroll-margin-top:1rem}
section.wait.ruling{border-color:var(--note-edge)}
section.wait.revision{border-color:var(--soft)}
section.wait .hd{margin:0;font:13px/1.4 system-ui,sans-serif;color:var(--soft)}
.shape{display:inline-block;margin-right:.4rem;padding:0 .4rem;border-radius:4px;font:700 11px/1.6 system-ui,sans-serif;letter-spacing:.06em;text-transform:uppercase;color:#fff;background:var(--claim-edge)}
.ruling .shape{background:var(--note-edge)}
.revision .shape{background:var(--soft)}
section.wait h3{margin:.35rem 0 .2rem;font:600 1.15rem/1.3 system-ui,sans-serif}
section.wait p{margin:.25rem 0}
section.wait .counts{font:600 14px/1.5 system-ui,sans-serif}
section.wait .read{font:14px/1.5 system-ui,sans-serif}
.cmdl{font:12px/1.4 system-ui,sans-serif;color:var(--soft);margin-top:.5rem}
details.takes{font:14px/1.5 system-ui,sans-serif;margin:.3rem 0}
details.takes>summary{cursor:pointer;color:var(--soft);font-size:13px}
.diff p{margin:.15rem 0;padding:.2rem .5rem;border-radius:4px}
.diff .o{background:color-mix(in srgb,#b4452c 12%,transparent)}
.diff .w{background:var(--claim)}
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
        let act = shown.decided_word();
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
            main.push_str(&claim_box(conn, base, &state, included, meant)?);
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
            main.push_str(&claim_box(conn, base, &state, included, meant)?);
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
        // What its accept takes: the narrative and each proposed version
        // it includes, or the closed versions that refuse it
        // (narrative-page, "A narrative's page lists what its accept
        // takes").
        let takes = crate::batch::of_narrative(&state, name, shown);
        main.push_str("<h2>To accept</h2>\n");
        if takes.closed.is_empty() {
            let _ = writeln!(
                main,
                "<p class=\"hint\">In your terminal, with your key:</p>\n<pre>boom accept {} --key &lt;key&gt;</pre>\n<p class=\"hint\">This accept takes {} in one entry:</p>\n<ul class=\"edges takes\">",
                escape(&at),
                if takes.versions.len() == 1 {
                    "1 version".to_owned()
                } else {
                    format!("{} versions", takes.versions.len())
                }
            );
            for named in &takes.versions {
                let _ = writeln!(
                    main,
                    "<li>{} <code>{}</code></li>",
                    named.kind.as_str(),
                    escape(&named.target())
                );
            }
            main.push_str("</ul>\n");
        } else {
            let _ = writeln!(
                main,
                "<p class=\"unheld\">Boom refuses its accept: {}. Its author proposes a new version that names the current ones.</p>",
                escape(&crate::batch::closed_words(&takes.closed))
            );
        }
    }
    let (_, book) = book_of(conn, None)?;
    main.push_str(&neighbors(&book, name, base));
    let page = Shell {
        title: &shown.proposal.statement,
        entry,
        base,
        view: Some(BOOK),
        side: Some(book_side(&book, base, BOOK.path, Some(name))),
        query: "",
        waiting: waiting_count(conn)?,
    };
    Ok(shell(&page, &main))
}

/// The links of a narrative that the design book gives: to the book, and
/// to the narratives before it and after it in the book's order; nothing
/// for a narrative that the book does not give (narrative-page, "A
/// narrative's place in the book").
fn neighbors(book: &Book, name: &str, base: &str) -> String {
    let order = book.order();
    let Some(at) = order.iter().position(|entry| entry.narrative == name) else {
        return String::new();
    };
    let link = |entry: &crate::view::Entry, rel: &str, words: &str| {
        format!(
            "<a rel=\"{rel}\" href=\"{base}/n/{}\">{words} {}</a>",
            escape(&entry.narrative),
            escape(&entry.title)
        )
    };
    let before = at
        .checked_sub(1)
        .and_then(|before| order.get(before))
        .map(|entry| link(entry, "prev", "←"))
        .unwrap_or_default();
    let after = order
        .get(at + 1)
        .map(|entry| link(entry, "next", "→"))
        .unwrap_or_default();
    format!(
        "<nav class=\"book\"><span>{before}</span><a rel=\"contents\" href=\"{base}/\">the design book</a><span>{after}</span></nav>\n"
    )
}

/// The design book of the store at the log's last entry, from the
/// outline `outline`, or from the outline of the view `book` where none
/// is named, computed once for a request.
fn book_of(conn: &rusqlite::Connection, outline: Option<&str>) -> Result<(State, Book), Failure> {
    let state = store::view_state(conn)?;
    if let Some(name) = outline {
        outline_claim(&state, name)?;
    }
    let book = crate::view::book(&state, outline);
    Ok((state, book))
}

/// The book's contents in the sidebar: its title, Part I with its
/// narratives and a section for each of its contexts, then Part II with
/// a chapter for each repository and a section for each of its
/// contexts. Each narrative is a link to its page with its title, in the
/// book's order; `current` has `aria-current="page"`, and each narrative
/// that the outline does not hold is marked (narrative-page, "The book's
/// contents in the sidebar", "The sidebar marks the page's narrative").
fn book_side(book: &Book, base: &str, home: &str, current: Option<&str>) -> String {
    let entries = |html: &mut String, entries: &[crate::view::Entry], class: &str| {
        if entries.is_empty() {
            return;
        }
        let _ = writeln!(html, "<ul{class}>");
        for entry in entries {
            let here = if current == Some(entry.narrative.as_str()) {
                " aria-current=\"page\""
            } else {
                ""
            };
            let unheld = if entry.held {
                ""
            } else {
                "<span class=\"unheld\">not in the outline</span>"
            };
            let _ = writeln!(
                html,
                "<li><a href=\"{base}/n/{}\"{here}>{}{unheld}</a></li>",
                escape(&entry.narrative),
                escape(&entry.title)
            );
        }
        html.push_str("</ul>\n");
    };
    let title = book
        .outline
        .as_ref()
        .map_or("The design book", |head| head.title.as_str());
    let mut html = format!(
        "<a class=\"book-title\" href=\"{base}{home}\">{}</a>\n",
        escape(title)
    );
    let _ = writeln!(html, "<p class=\"part\">{}</p>", crate::view::PART_I);
    entries(&mut html, &book.system.narratives, "");
    for section in &book.system.sections {
        let _ = writeln!(
            html,
            "<p class=\"section\">{}</p>",
            escape(&section.context)
        );
        entries(&mut html, &section.narratives, " class=\"in\"");
    }
    let _ = writeln!(html, "<p class=\"part\">{}</p>", crate::view::PART_II);
    for chapter in &book.repositories {
        let _ = writeln!(
            html,
            "<p class=\"chapter\">{}</p>",
            escape(&chapter.repository)
        );
        entries(&mut html, &chapter.narratives, "");
        for section in &chapter.sections {
            let _ = writeln!(
                html,
                "<p class=\"section\">{}</p>",
                escape(&section.context)
            );
            entries(&mut html, &section.narratives, " class=\"in\"");
        }
    }
    html
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

/// The box of an included claim: its kind, `<name>@<version>` as a link
/// to its page in the claims view at the version meant, its status, its
/// statement and its body, marked out of date where another version of
/// it is accepted (narrative-page, "An included claim links to the
/// claims view").
fn claim_box(
    conn: &rusqlite::Connection,
    base: &str,
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
        "<div class=\"box claim\"><div class=\"head\">{} <a href=\"{base}/claims/{at}\"><code>{at}</code></a> · {}",
        claim.kind.as_str(),
        version.status.as_str(),
        at = format!("{}@{}", escape(name), escape(meant)),
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

/// The count of open notes in words.
fn open_words(open: usize) -> String {
    match open {
        0 => "no open note".to_owned(),
        1 => "1 open note".to_owned(),
        n => format!("{n} open notes"),
    }
}

/// The entries of a level of the book, each a link to its narrative's
/// page with its title, `<name>@<version>`, status, open notes and mark.
fn entries_html(entries: &[crate::view::Entry], base: &str) -> String {
    if entries.is_empty() {
        return String::new();
    }
    let mut html = String::from("<ul class=\"list\">\n");
    for entry in entries {
        let unheld = if entry.held {
            String::new()
        } else {
            " · <span class=\"unheld\">not in the outline</span>".to_owned()
        };
        let _ = writeln!(
            html,
            "<li data-narrative=\"{name}\"><a class=\"title\" href=\"{base}/n/{name}\">{title}</a><div class=\"meta\"><code>{name}@{version}</code> · <span class=\"status\">{status}</span> · <span class=\"open\" data-open=\"{open}\">{open_words}</span>{unheld}</div></li>",
            name = escape(&entry.narrative),
            title = escape(&entry.title),
            version = entry.version,
            status = entry.status,
            open = entry.open_notes,
            open_words = open_words(entry.open_notes),
        );
    }
    html.push_str("</ul>\n");
    html
}

/// The page of the design book, computed from the outline `outline`, or
/// from the outline of the view `book` where none is named: the outline's
/// head, each part, chapter and section as a heading, and each narrative
/// as a link to its page (narrative-page, "The design book on the page").
pub fn book_page(
    conn: &rusqlite::Connection,
    outline: Option<&str>,
    context: &Context,
) -> Result<String, Failure> {
    let (state, book) = book_of(conn, outline)?;
    let (entry, _) = store::head_entry(conn)?;
    let base = &context.base;
    let mut main = String::new();
    let title = match &book.outline {
        Some(head) => {
            let count = state
                .named(&head.name)
                .first()
                .and_then(|claim| claim.version(&head.version))
                .map_or(0, |version| version.proposal.narratives.len());
            let _ = writeln!(
                main,
                "<h1>{}</h1>\n<p class=\"meta\">outline <code>{}@{}</code> · <span class=\"status\">{}</span> · {count} narratives</p>",
                escape(&head.title),
                escape(&head.name),
                head.version,
                head.status
            );
            if head.body.is_some() {
                main.push_str(&markdown(&body_text(conn, head.body.as_deref())?));
            }
            head.title.clone()
        }
        None => {
            main.push_str("<h1>The design book</h1>\n<p class=\"hint\">No outline orders this book, so each narrative is given by name, marked as not in the outline. An agent proposes the outline with <code>boom propose outline &lt;name&gt; \"&lt;title&gt;\" --view book --narrative &lt;name&gt; ...</code>, and a person accepts it.</p>\n");
            "The design book".to_owned()
        }
    };
    let _ = writeln!(main, "<h2>{}</h2>", crate::view::PART_I);
    main.push_str(&entries_html(&book.system.narratives, base));
    for section in &book.system.sections {
        let _ = writeln!(main, "<h3>{}</h3>", escape(&section.context));
        main.push_str(&entries_html(&section.narratives, base));
    }
    if book.system.narratives.is_empty() && book.system.sections.is_empty() {
        main.push_str("<p class=\"hint\">No narrative binds the whole system.</p>\n");
    }
    let _ = writeln!(main, "<h2>{}</h2>", crate::view::PART_II);
    for chapter in &book.repositories {
        let _ = writeln!(main, "<h3>{}</h3>", escape(&chapter.repository));
        main.push_str(&entries_html(&chapter.narratives, base));
        for section in &chapter.sections {
            let _ = writeln!(main, "<h4>{}</h4>", escape(&section.context));
            main.push_str(&entries_html(&section.narratives, base));
        }
    }
    if book.repositories.is_empty() {
        main.push_str("<p class=\"hint\">No narrative binds a repository.</p>\n");
    }
    let home = outline.map_or_else(|| BOOK.path.to_owned(), |name| format!("/v/{name}"));
    let page = Shell {
        title: &title,
        entry,
        base,
        view: Some(BOOK),
        side: Some(book_side(&book, base, &home, None)),
        query: "",
        waiting: waiting_count(conn)?,
    };
    Ok(shell(&page, &main))
}

/// The page of the search `query`: the narratives of the book whose
/// paragraphs, at the version that the book gives, hold each word, each
/// linked to its first paragraph that holds them all; then each other
/// claim that `boom search --all` gives, each linked to its page in the
/// claims view (narrative-page, "A search from the page", "A narrative
/// found links to its paragraph").
pub fn search_page(
    conn: &rusqlite::Connection,
    query: &str,
    context: &Context,
) -> Result<String, Failure> {
    let (state, book) = book_of(conn, None)?;
    let (entry, _) = store::head_entry(conn)?;
    let base = &context.base;
    let given: Vec<String> = query.split_whitespace().map(str::to_owned).collect();
    let words = crate::search::words(&given);
    let mut main = String::from("<h1>Search</h1>\n");
    if words.is_empty() {
        let _ = writeln!(
            main,
            "<p class=\"hint\">{}.</p>",
            escape(crate::search::NO_WORD)
        );
    } else {
        let _ = writeln!(
            main,
            "<p class=\"meta\">“{}”, across the book and every repository, at entry {entry} of the store's log</p>",
            escape(&words.join(" "))
        );
        main.push_str(&found_narratives(conn, &state, &book, &words, base)?);
        main.push_str(&found_claims(conn, &words, base)?);
    }
    let page = Shell {
        title: "Search",
        entry,
        base,
        view: None,
        side: Some(book_side(&book, base, BOOK.path, None)),
        query,
        waiting: waiting_count(conn)?,
    };
    Ok(shell(&page, &main))
}

/// The narratives of `book` whose paragraphs hold each of `words`, in
/// the book's order, each with a link to the first paragraph that holds
/// them all and that paragraph's words.
fn found_narratives(
    conn: &rusqlite::Connection,
    state: &State,
    book: &Book,
    words: &[String],
    base: &str,
) -> Result<String, Failure> {
    let order = book.order();
    let mut documents = Vec::with_capacity(order.len());
    for entry in &order {
        let body = state
            .named(&entry.narrative)
            .first()
            .and_then(|claim| claim.version(&entry.version))
            .and_then(|version| version.proposal.body.clone());
        let body = body_text(conn, body.as_deref())?;
        documents.push(
            narrative::paragraphs(&body)
                .iter()
                .map(Paragraph::words)
                .collect::<Vec<_>>(),
        );
    }
    let found = crate::search::paragraphs(&documents, words)?;
    let mut html = format!(
        "<h2>Narratives of the book</h2>\n<p class=\"hint\">{}</p>\n",
        count_words(
            found.len(),
            "narrative holds",
            "narratives hold",
            "each word"
        )
    );
    if found.is_empty() {
        return Ok(html);
    }
    html.push_str("<ul class=\"list found\">\n");
    for (place, held) in found {
        let entry = order[place];
        let name = escape(&entry.narrative);
        let (at, excerpt) = match held {
            crate::search::Held::Paragraph(n) => (
                format!("#p{}", n + 1),
                Some(format!("¶{} {}", n + 1, documents[place][n])),
            ),
            crate::search::Held::Across(excerpt) => (String::new(), excerpt),
        };
        let _ = write!(
            html,
            "<li data-narrative=\"{name}\"><a class=\"title\" href=\"{base}/n/{name}{at}\">{}</a><div class=\"meta\">narrative <code>{name}@{}</code> · <span class=\"status\">{}</span></div>",
            escape(&entry.title),
            entry.version,
            entry.status,
        );
        if let Some(excerpt) = excerpt {
            let _ = write!(html, "<p class=\"quote\">{}</p>", escape(&excerpt));
        }
        html.push_str("</li>\n");
    }
    html.push_str("</ul>\n");
    Ok(html)
}

/// Each claim that `boom search --all` gives for `words`, in its order,
/// but the narratives, which [`found_narratives`] gives: its kind,
/// `<name>@<version>`, status, statement and excerpt, as a link to its
/// page in the claims view.
fn found_claims(
    conn: &rusqlite::Connection,
    words: &[String],
    base: &str,
) -> Result<String, Failure> {
    let found: Vec<crate::search::Standing> = crate::search::standing(conn, None, &[], words)?
        .into_iter()
        .filter(|found| found.claim.kind != Kind::Narrative)
        .collect();
    let mut html = format!(
        "<h2>Claims that stand</h2>\n<p class=\"hint\">{}</p>\n",
        count_words(found.len(), "claim holds", "claims hold", "each word")
    );
    if found.is_empty() {
        return Ok(html);
    }
    html.push_str("<ul class=\"list found\">\n");
    for found in &found {
        let at = format!(
            "{}@{}",
            escape(&found.claim.name),
            found.version.proposal.version
        );
        let _ = write!(
            html,
            "<li data-claim=\"{name}\"><div class=\"meta\">{kind} <a href=\"{base}/claims/{at}\"><code>{at}</code></a> · <span class=\"status\">{status}</span> · binds {binds}</div><a class=\"title\" href=\"{base}/claims/{at}\">{statement}</a>",
            name = escape(&found.claim.name),
            kind = found.claim.kind.as_str(),
            status = judge::Status::Accepted.as_str(),
            binds = escape(&binds(&found.version)),
            statement = escape(&found.version.proposal.statement),
        );
        if let Some(excerpt) = &found.excerpt {
            let _ = write!(html, "<p class=\"quote\">{}</p>", escape(excerpt));
        }
        html.push_str("</li>\n");
    }
    html.push_str("</ul>\n");
    Ok(html)
}

/// A count in words: "No narrative holds each word", "1 claim holds
/// each word", "3 claims hold each word".
fn count_words(count: usize, one: &str, many: &str, what: &str) -> String {
    match count {
        0 => format!("No {one} {what}."),
        1 => format!("1 {one} {what}."),
        n => format!("{n} {many} {what}."),
    }
}

/// The claim `name` of `state`, an outline.
fn outline_claim<'a>(state: &'a State, name: &str) -> Result<&'a Claim, Failure> {
    let claim = state
        .named(name)
        .into_iter()
        .next()
        .ok_or_else(|| Failure::refused(format!("no claim has the name {name}")))?;
    if claim.kind != Kind::Outline {
        return Err(Failure::refused(format!(
            "--view names an outline: {name} is {} {}",
            crate::claim::article(claim.kind),
            claim.kind.as_str()
        )));
    }
    Ok(claim)
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

/// The query of `url`, after its `?` and before its `#`.
fn query_of(url: &str) -> &str {
    url.split_once('?')
        .map(|(_, query)| query.split('#').next().unwrap_or_default())
        .unwrap_or_default()
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

/// A response: its status, its type, its bytes, and a place to go to.
struct Reply {
    status: u16,
    content_type: &'static str,
    body: Cow<'static, [u8]>,
    location: Option<String>,
}

impl Reply {
    fn page(html: String) -> Reply {
        Reply {
            status: 200,
            content_type: "text/html; charset=utf-8",
            body: Cow::Owned(html.into_bytes()),
            location: None,
        }
    }

    fn words(status: u16, words: &str) -> Reply {
        Reply {
            status,
            ..Reply::page(format!(
                "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><title>Boom</title></head><body><p>{}</p></body></html>\n",
                escape(words)
            ))
        }
    }

    /// A file that boom holds in its binary.
    fn held(content_type: &'static str, bytes: &'static [u8]) -> Reply {
        Reply {
            status: 200,
            content_type,
            body: Cow::Borrowed(bytes),
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
            ("GET" | "HEAD", None) if rest == script_path() => {
                Reply::held("text/javascript; charset=utf-8", MERMAID_SCRIPT)
            }
            ("GET" | "HEAD", None) if rest == license_path() => {
                Reply::held("text/plain; charset=utf-8", MERMAID_LICENSE)
            }
            ("GET" | "HEAD", None) if rest.is_empty() => {
                self.read(|conn| book_page(conn, None, &self.context))
            }
            ("GET" | "HEAD", None) if rest == "waiting" => {
                self.read(|conn| waiting::waiting_page(conn, &self.context))
            }
            ("GET" | "HEAD", None) if rest == "search" => {
                let words = form(query_of(url))
                    .into_iter()
                    .find(|(name, _)| name == "q")
                    .map(|(_, words)| words)
                    .unwrap_or_default();
                self.read(|conn| search_page(conn, &words, &self.context))
            }
            ("GET" | "HEAD", None) if claims::Of::from_path(rest).is_some() => {
                let of = claims::Of::from_path(rest).expect("a view of the claims");
                let fields = form(query_of(url));
                let answered = Store::open(&self.path).and_then(|mut store| {
                    store.read(|conn| claims::view_page(conn, of, &fields, &self.context))
                });
                match answered {
                    Ok(Ok(html)) => Reply::page(html),
                    Ok(Err(words)) => Reply::words(400, &words),
                    Err(Failure::Refused(why)) => Reply::words(404, &why),
                    Err(Failure::NotRun(why)) => Reply::words(500, &why),
                }
            }
            ("GET" | "HEAD", None)
                if rest.len() > "claims/".len()
                    && rest.starts_with("claims/")
                    && !rest["claims/".len()..].contains('/') =>
            {
                let target = &rest["claims/".len()..];
                let (name, version) = match target.split_once('@') {
                    Some((name, version)) => (name, Some(version)),
                    None => (target, None),
                };
                self.read(|conn| claims::claim_page(conn, name, version, &self.context))
            }
            ("GET" | "HEAD", None) if rest.starts_with("v/") && !rest[2..].contains('/') => {
                self.read(|conn| book_page(conn, Some(&rest[2..]), &self.context))
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
                location: Some(format!(
                    "{}/n/{name}@{version}#p{paragraph}",
                    self.context.base
                )),
                ..Reply::page(String::new())
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
        ("Content-Type", reply.content_type.to_owned()),
        ("Content-Security-Policy", POLICY.to_owned()),
        ("Cache-Control", "no-store".to_owned()),
        ("Referrer-Policy", REFERRER_POLICY.to_owned()),
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
        .map(|target| target.split_once('@').map_or(target, |(name, _)| name))
        .or(options.view.as_deref());
    let state = store.state_for(&Scope::names(name))?;
    let shown = match (&options.target, &options.view) {
        (Some(_), Some(_)) => {
            return Err(Failure::refused(
                "boom page takes a narrative or --view <outline>, and not both",
            ))
        }
        (None, Some(view)) => {
            let claim = outline_claim(&state, view)?;
            let version = crate::view::shown(claim)
                .map(|version| version.proposal.version.clone())
                .unwrap_or_default();
            format!("{view}@{version}")
        }
        (Some(target), None) => {
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
        (None, None) => "the design book".to_owned(),
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
    let first = match (&options.target, &options.view) {
        (Some(target), _) => format!("{base}/n/{target}"),
        (None, Some(view)) => format!("{base}/v/{view}"),
        (None, None) => format!("{base}/"),
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
        let length = reply.body.len();
        let mut response = tiny_http::Response::new(
            tiny_http::StatusCode(reply.status),
            Vec::new(),
            std::io::Cursor::new(reply.body.clone()),
            Some(length),
            None,
        )
        // Each length is known: the response gives it, never chunks.
        .with_chunked_threshold(usize::MAX);
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

    /// narrative-page, "The pinned script".
    #[test]
    fn the_pinned_script() {
        use sha2::Digest as _;
        assert_eq!(
            crate::hex(&sha2::Sha256::digest(MERMAID_SCRIPT)),
            MERMAID_SHA256
        );
        let script = String::from_utf8_lossy(MERMAID_SCRIPT);
        assert!(
            script.contains(&format!("\"{MERMAID_VERSION}\"")),
            "the script names no version {MERMAID_VERSION}"
        );
        assert!(String::from_utf8_lossy(MERMAID_LICENSE).starts_with("The MIT License"));
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
