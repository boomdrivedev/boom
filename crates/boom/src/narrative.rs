//! A narrative's body: its paragraphs, each with its number, kind and
//! fingerprint, and the name of a note on one of them (narratives;
//! narrative-notes).
//!
//! Boom reads a body as CommonMark with tables. A paragraph is one block
//! at the top of the body, a heading, a paragraph of text, a code block,
//! a block quote, a table or a block of HTML, or one item of a list at
//! the top of the body, with its nested lists. A thematic break is no
//! paragraph. A version never changes, so its numbers never change.

use std::ops::Range;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// One paragraph of a body.
#[derive(Debug, Clone)]
pub struct Paragraph {
    /// Its number, from 1, in the order of the body.
    pub number: u64,
    /// Its kind: `heading`, `text`, `code`, `quote`, `table`, `html` or
    /// `item`.
    pub kind: &'static str,
    /// Its bytes in the body, with no line break at their end.
    pub range: Range<usize>,
    /// The first 12 hexadecimal characters of the SHA-256 of its bytes.
    pub fingerprint: String,
    /// Its events, for the page to write: an item's are inside a list of
    /// one item, numbered as it is numbered in the body.
    pub events: Vec<Event<'static>>,
}

impl Paragraph {
    /// Its bytes in `body`.
    pub fn source<'a>(&self, body: &'a str) -> &'a str {
        &body[self.range.clone()]
    }

    /// Its words as a person reads them: its text and its code, each run
    /// of white space one space.
    pub fn words(&self) -> String {
        let mut text = String::new();
        for event in &self.events {
            match event {
                Event::Text(words)
                | Event::Code(words)
                | Event::Html(words)
                | Event::InlineHtml(words) => text.push_str(words),
                Event::SoftBreak | Event::HardBreak | Event::End(_) => text.push(' '),
                _ => {}
            }
        }
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Whether its bytes in `body` name `name` as a whole word: no letter,
    /// digit or hyphen before it or after it, as the terms check matches a
    /// hyphenated word.
    pub fn names(&self, body: &str, name: &str) -> bool {
        let source = self.source(body);
        let part = |c: char| c.is_alphanumeric() || c == '-';
        source.match_indices(name).any(|(at, _)| {
            let before = source[..at].chars().next_back();
            let after = source[at + name.len()..].chars().next();
            !before.is_some_and(part) && !after.is_some_and(part)
        })
    }
}

/// The paragraphs of `body`, numbered from 1.
pub fn paragraphs(body: &str) -> Vec<Paragraph> {
    let mut found = Vec::new();
    let mut depth = 0usize;
    // The start of the list at the top that is open, and its next number.
    let mut list: Option<Option<u64>> = None;
    // The paragraph that is open: its kind, its start and its events.
    let mut open: Option<(&'static str, usize, Vec<Event<'static>>)> = None;
    for (event, range) in Parser::new_ext(body, Options::ENABLE_TABLES).into_offset_iter() {
        match &event {
            Event::Start(tag) => {
                let at_top = depth == 0;
                let in_list = depth == 1 && list.is_some();
                depth += 1;
                if open.is_some() {
                    push(&mut open, event);
                    continue;
                }
                if at_top {
                    if let Tag::List(start) = tag {
                        list = Some(*start);
                        continue;
                    }
                    open = Some((kind(tag), range.start, vec![event.into_static()]));
                } else if in_list && matches!(tag, Tag::Item) {
                    let start = list.flatten();
                    let wrap = Event::Start(Tag::List(start));
                    open = Some(("item", range.start, vec![wrap, event.into_static()]));
                }
            }
            Event::End(end) => {
                depth = depth.saturating_sub(1);
                let closes = depth == 0 || (depth == 1 && list.is_some() && *end == TagEnd::Item);
                if let (true, Some(_)) = (closes, &open) {
                    if matches!(end, TagEnd::List(_)) && depth == 0 {
                        list = None;
                        continue;
                    }
                    let (kind, start, mut events) = open.take().expect("a paragraph is open");
                    events.push(event.into_static());
                    if kind == "item" {
                        let ordered = list.flatten().is_some();
                        events.push(Event::End(TagEnd::List(ordered)));
                        if let Some(Some(next)) = list.as_mut() {
                            *next += 1;
                        }
                    }
                    let mut end = range.end;
                    while end > start && matches!(body.as_bytes()[end - 1], b'\n' | b'\r') {
                        end -= 1;
                    }
                    let bytes = &body.as_bytes()[start..end];
                    found.push(Paragraph {
                        number: found.len() as u64 + 1,
                        kind,
                        range: start..end,
                        fingerprint: crate::sha256_hex(bytes)[..12].to_owned(),
                        events,
                    });
                } else if open.is_some() {
                    push(&mut open, event);
                } else if depth == 0 && matches!(end, TagEnd::List(_)) {
                    list = None;
                }
            }
            // A thematic break at the top is no paragraph.
            Event::Rule if open.is_none() => {}
            _ => push(&mut open, event),
        }
    }
    found
}

fn push(open: &mut Option<(&'static str, usize, Vec<Event<'static>>)>, event: Event<'_>) {
    if let Some((_, _, events)) = open {
        events.push(event.into_static());
    }
}

/// The kind of a paragraph that starts with `tag` at the top of a body.
fn kind(tag: &Tag<'_>) -> &'static str {
    match tag {
        Tag::Heading { .. } => "heading",
        Tag::CodeBlock(_) => "code",
        Tag::BlockQuote(_) => "quote",
        Tag::Table(_) => "table",
        Tag::HtmlBlock => "html",
        Tag::Item | Tag::List(_) => "item",
        _ => "text",
    }
}

/// The name of a note: the first 12 hexadecimal characters of the SHA-256
/// of the narrative's name, the version, the paragraph's number, the words
/// and the person, as one JSON array.
pub fn note_name(name: &str, version: &str, paragraph: u64, words: &str, person: &str) -> String {
    let content = serde_json::json!([name, version, paragraph, words, person]).to_string();
    crate::sha256_hex(content.as_bytes())[..12].to_owned()
}

/// The fingerprints of the paragraphs of `body`, in order.
pub fn fingerprints(body: &str) -> Vec<String> {
    paragraphs(body)
        .into_iter()
        .map(|paragraph| paragraph.fingerprint)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(body: &str) -> Vec<&'static str> {
        paragraphs(body).iter().map(|p| p.kind).collect()
    }

    #[test]
    fn each_block_at_the_top_is_a_paragraph() {
        let body = "# Title\n\nSome text\non two lines.\n\n```\ncode\n```\n\n> quoted\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n<div>raw</div>\n";
        assert_eq!(
            kinds(body),
            ["heading", "text", "code", "quote", "table", "html"]
        );
        let found = paragraphs(body);
        assert_eq!(found[1].source(body), "Some text\non two lines.");
        assert_eq!(found[1].words(), "Some text on two lines.");
        assert_eq!(found[0].source(body), "# Title");
        assert_eq!(
            found[1].fingerprint,
            crate::sha256_hex(b"Some text\non two lines.")[..12]
        );
    }

    #[test]
    fn each_item_of_a_list_at_the_top_is_a_paragraph() {
        let body =
            "Intro.\n\n1. one\n2. two\n   - nested a\n   - nested b\n3. three\n\n---\n\nEnd.\n";
        let found = paragraphs(body);
        assert_eq!(
            found.iter().map(|p| p.kind).collect::<Vec<_>>(),
            ["text", "item", "item", "item", "text"]
        );
        assert_eq!(
            found[2].source(body),
            "2. two\n   - nested a\n   - nested b"
        );
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, found[3].events.clone().into_iter());
        assert!(html.starts_with("<ol start=\"3\">"), "{html}");
        assert_eq!(
            found.iter().map(|p| p.number).collect::<Vec<_>>(),
            [1, 2, 3, 4, 5]
        );
    }

    #[test]
    fn a_paragraph_names_a_claim_as_a_whole_word() {
        let body = "It rests on `status-read-from-events`.\n\nNot status-read-from-events-too.\n";
        let found = paragraphs(body);
        assert!(found[0].names(body, "status-read-from-events"));
        assert!(!found[1].names(body, "status-read-from-events"));
    }

    #[test]
    fn a_notes_name_is_the_fingerprint_of_what_it_names() {
        let one = note_name("boom-end-to-end", "aaaaaaaaaaaa", 3, "Say it.", "chuck");
        assert_eq!(one.len(), 12);
        assert_eq!(
            one,
            note_name("boom-end-to-end", "aaaaaaaaaaaa", 3, "Say it.", "chuck")
        );
        assert_ne!(
            one,
            note_name("boom-end-to-end", "aaaaaaaaaaaa", 4, "Say it.", "chuck")
        );
        assert_ne!(
            one,
            note_name("boom-end-to-end", "aaaaaaaaaaaa", 3, "Say it.", "ada")
        );
    }
}
