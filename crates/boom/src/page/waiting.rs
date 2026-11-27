//! The view "Waiting on you" of `boom page`: each batch that waits on a
//! person, with its shape, its title, why it exists, its counts, what to
//! read and its one command (narrative-page, "The view \"Waiting on
//! you\""). Each command is the one the person runs: with no `--key`
//! where Boom's settings hold their key. The person picks batches with
//! a form that sends `GET` to the view, and the view gives the one
//! command for those it picks (narrative-page, "The view \"Waiting on
//! you\" gives the person's own command"). It is computed from the views
//! at the log's last entry, for each request
//! (`views-computed-from-the-ledger`).

use std::fmt::Write as _;

use super::{actor_words, escape, shell, Context, Shell, WAITING};
use crate::batch::{self, Batch, Named, Shape};
use crate::settings::Signing;
use crate::store;
use crate::Failure;

/// The form the person picks batches with: its `id`, which each box
/// names.
const PICK: &str = "pick";

/// The words of a batch's head, as its anchor on the page and its value
/// in the form.
fn anchor(batch: &Batch) -> String {
    format!("{}-{}", batch.shape.as_str(), batch.head.name)
}

/// The link that reads a version: a narrative's page, else the claim's.
fn read_link(base: &str, named: &Named) -> String {
    let path = if named.kind == crate::claim::Kind::Narrative {
        "n"
    } else {
        "claims"
    };
    format!(
        "<a href=\"{base}/{path}/{target}\"><code>{target}</code></a>",
        target = escape(&named.target())
    )
}

/// Each version that a batch takes, folded: its kind and a link to it,
/// for a revision with its old words and its new.
fn versions_html(
    conn: &rusqlite::Connection,
    batch: &Batch,
    base: &str,
) -> Result<String, Failure> {
    let mut html = format!(
        "<details class=\"takes\"><summary>The {} that this sign takes</summary>\n<ul class=\"edges\">\n",
        if batch.takes.versions.len() == 1 {
            "1 version".to_owned()
        } else {
            format!("{} versions", batch.takes.versions.len())
        }
    );
    for named in &batch.takes.versions {
        let _ = write!(
            html,
            "<li>{} {}",
            named.kind.as_str(),
            read_link(base, named)
        );
        let before = match (&batch.shape, &named.base) {
            (Shape::Revision, Some(base_version)) => {
                store::meant(conn, &named.name, base_version)?.map(|(statement, _)| statement)
            }
            _ => None,
        };
        match before {
            Some(before) => {
                let _ = write!(
                    html,
                    "<div class=\"diff\"><p class=\"o\"><del>{}</del></p><p class=\"w\">{}</p></div>",
                    escape(&before),
                    escape(&named.statement)
                );
            }
            None => {
                let _ = write!(html, "<br>{}", escape(&named.statement));
            }
        }
        html.push_str("</li>\n");
    }
    html.push_str("</ul></details>\n");
    Ok(html)
}

/// One batch on the page, its box checked where `picked`.
fn batch_html(
    conn: &rusqlite::Connection,
    batch: &Batch,
    context: &Context,
    picked: bool,
) -> Result<String, Failure> {
    let base = &context.base;
    let shape = batch.shape.as_str();
    let binds = crate::context::binds_words(&batch.repositories, &[]);
    let mut html = format!(
        "<section class=\"wait {shape}\" id=\"{id}\">\n<p class=\"hd\"><span class=\"shape\">{shape}</span> proposed by {by} · binds {binds}</p>\n<h3>{title}</h3>\n",
        id = escape(&anchor(batch)),
        by = escape(&actor_words(&batch.proposer)),
        binds = escape(&binds),
        title = escape(&batch.title),
    );
    let why = match batch.shape {
        Shape::Narrative => format!(
            "The narrative {}, with each proposed claim it includes, each at its paragraph.",
            read_link(base, &batch.head)
        ),
        Shape::Revision => format!(
            "Each proposed version whose cause is {}, and the cause{}.",
            read_link(base, &batch.head),
            if batch.head.status == crate::judge::Status::Proposed {
                ""
            } else {
                ", accepted already"
            }
        ),
        Shape::Ruling => format!(
            "One {} that no waiting narrative includes and that names no cause: {}.",
            batch.head.kind.as_str(),
            read_link(base, &batch.head)
        ),
    };
    let _ = writeln!(
        html,
        "<p class=\"counts\">{}</p>\n<p class=\"read\">{why}</p>",
        escape(&batch.counts())
    );
    html.push_str(&versions_html(conn, batch, base)?);
    match batch.command(&context.signing) {
        Some(command) => {
            let _ = writeln!(
                html,
                "<p class=\"cmdl\">In your terminal, with your key: Boom lists each version before the key signs.</p>\n<pre class=\"command\">{}</pre>\n<label class=\"pick\"><input type=\"checkbox\" form=\"{PICK}\" name=\"{PICK}\" value=\"{}\"{}> Pick this batch</label>",
                escape(&command),
                escape(&anchor(batch)),
                if picked { " checked" } else { "" }
            );
        }
        None => {
            let _ = writeln!(
                html,
                "<p class=\"unheld\">Boom refuses this accept: {}. Its author proposes a new version that names the current ones.</p>",
                escape(&batch::closed_words(&batch.takes.closed))
            );
        }
    }
    html.push_str("</section>\n");
    Ok(html)
}

/// The sidebar: each batch, with the number of versions it takes.
fn side(batches: &[Batch]) -> String {
    let mut html = String::from("<p class=\"part\">Waiting on you</p>\n<ul>\n");
    for batch in batches {
        let _ = writeln!(
            html,
            "<li><a href=\"#{}\">{}<span class=\"n\">{}</span></a></li>",
            escape(&anchor(batch)),
            escape(&batch.head.name),
            batch.takes.versions.len()
        );
    }
    html.push_str("</ul>\n");
    html
}

/// How the page says the key is named: from Boom's settings, or with
/// `--key`, and the one line of the settings that names it.
fn signing_html(context: &Context) -> String {
    let file = context.settings.as_ref().map_or_else(
        || "Boom's settings".to_owned(),
        |file| format!("<code>{}</code>", escape(&file.display().to_string())),
    );
    match &context.signing {
        Signing::Settings { person } => format!(
            "<p class=\"hint\">Each command signs with the key of {} that {file} names: it takes no <code>--key</code>.</p>\n",
            escape(person)
        ),
        Signing::Given => format!(
            "<p class=\"hint\">Each command takes your key with <code>--key</code>. One line in {file} names your key, and the program that gives its passphrase where it has one, and then no command takes <code>--key</code>:</p>\n<pre class=\"command\">key = {{ path = \"~/.ssh/&lt;your key&gt;\", askpass = \"&lt;program&gt;\" }}</pre>\n"
        ),
    }
}

/// The page `<base>/waiting`, with the batches that `fields` pick.
pub fn waiting_page(
    conn: &rusqlite::Connection,
    fields: &[(String, String)],
    context: &Context,
) -> Result<String, Failure> {
    let batches = batch::waiting(conn)?;
    let wanted: Vec<&str> = fields
        .iter()
        .filter(|(name, _)| name == PICK)
        .map(|(_, value)| value.as_str())
        .collect();
    let picked: Vec<&Batch> = batches
        .iter()
        .filter(|batch| !batch.refused() && wanted.contains(&anchor(batch).as_str()))
        .collect();
    let (entry, _) = store::head_entry(conn)?;
    let base = &context.base;
    let mut main = String::from("<h1>Waiting on you</h1>\n");
    if batches.is_empty() {
        main.push_str("<p>Nothing waits on you: no version is proposed.</p>\n");
    } else {
        let mut versions: Vec<(String, String)> = Vec::new();
        for batch in batches.iter().filter(|batch| !batch.refused()) {
            for named in &batch.takes.versions {
                let pair = (named.name.clone(), named.version.clone());
                if !versions.contains(&pair) {
                    versions.push(pair);
                }
            }
        }
        let _ = writeln!(
            main,
            "<p class=\"hint\">{} {}, {} {}. Each batch is one command, which your key signs one time. Nothing here binds work until you accept it.</p>",
            batches.len(),
            if batches.len() == 1 { "batch" } else { "batches" },
            versions.len(),
            if versions.len() == 1 { "version" } else { "versions" },
        );
        main.push_str("<div class=\"how\"><p>Boom computes each batch from the log's links, and no agent makes one:</p><ul><li>a narrative: it and the proposed versions it includes</li><li>a revision: the proposed versions that name one cause, and the cause</li><li>a ruling: one proposed version that no waiting narrative includes and that names no cause</li></ul></div>\n");
        main.push_str(&signing_html(context));
        if let Some(command) = batch::accept_command(picked.iter().copied(), &context.signing) {
            let _ = writeln!(
                main,
                "<div class=\"total yours\" id=\"picked\"><p>The {} you picked, in one sign:</p><pre class=\"command\">{}</pre></div>",
                if picked.len() == 1 {
                    "1 batch".to_owned()
                } else {
                    format!("{} batches", picked.len())
                },
                escape(&command)
            );
        }
        if let Some(command) =
            batch::accept_command(&batches, &context.signing).filter(|_| batches.len() > 1)
        {
            let _ = writeln!(
                main,
                "<div class=\"total\"><p>Each batch that Boom does not refuse, the {}, in one sign:</p><pre class=\"command\">{}</pre><p>Boom lists each version before the key signs, and <code>boom accept --waiting</code> lists the batches.</p></div>",
                if versions.len() == 1 {
                    "1 version".to_owned()
                } else {
                    format!("{} versions", versions.len())
                },
                escape(&command)
            );
        }
        let _ = writeln!(
            main,
            "<form id=\"{PICK}\" class=\"pick\" method=\"get\" action=\"{base}/waiting#picked\"><p>Pick the batches to accept below, then <button type=\"submit\">Give one command for the batches I pick</button></p></form>"
        );
        for batch in &batches {
            let picked = picked.iter().any(|one| std::ptr::eq(*one, batch));
            main.push_str(&batch_html(conn, batch, context, picked)?);
        }
        let _ = writeln!(
            main,
            "<p class=\"pick\"><button type=\"submit\" form=\"{PICK}\">Give one command for the batches I pick</button></p>"
        );
    }
    let page = Shell {
        title: "Waiting on you",
        entry,
        base,
        view: Some(WAITING),
        side: (!batches.is_empty()).then(|| side(&batches)),
        query: "",
        waiting: batches.len(),
    };
    Ok(shell(&page, &main))
}
