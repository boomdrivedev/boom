//! A search of the claims that stand by their words (claim-search;
//! record 0009): the words an agent gives, each searched as written,
//! never as a query language (design 2), in an FTS5 table filled in
//! memory for one answer and dropped with it, so no index is kept
//! (design 3; core.2), and ranked by where the words fall (design 5).

use rusqlite::Connection;

use crate::Failure;

/// The table one search fills, a claim a row: its columns in the order
/// `RANK`'s weights name them. `unicode61` splits a name's hyphens and
/// a legacy id's dots and folds case and accents; `porter` takes each
/// word to its stem.
const TABLE: &str = "CREATE VIRTUAL TABLE found USING fts5(
    name, legacy, statement, section, scenarios, body,
    tokenize = 'porter unicode61 remove_diacritics 2'
)";

/// The matches, best first: FTS5's `bm25` with a weight per column, a
/// name or a legacy id 10, a statement or a section 5, scenarios 2, a
/// body 1, lower better, equal matches in their place; each with the
/// sixteen tokens of its body where the words fall thickest.
const RANK: &str = "SELECT rowid, snippet(found, 5, '', '', '…', 16)
    FROM found WHERE found MATCH ?1
    ORDER BY bm25(found, 10.0, 10.0, 5.0, 5.0, 2.0, 1.0), rowid";

/// The words of a claim a search reads.
#[derive(Debug, Clone, Copy, Default)]
pub struct Claim<'a> {
    /// Its name.
    pub name: &'a str,
    /// An imported record's number or a rule's id, where it has one.
    pub legacy: Option<&'a str>,
    /// Its statement.
    pub statement: &'a str,
    /// An imported rule's `###` heading, where it has one.
    pub section: Option<&'a str>,
    /// Its scenarios.
    pub scenarios: &'a [String],
    /// Its body's text, where it has one that is text.
    pub body: Option<&'a str>,
}

/// A claim the words match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// Its place among the claims searched.
    pub place: usize,
    /// The passage of its body where the words fall, on one line; none
    /// where it has no body text.
    pub excerpt: Option<String>,
}

/// Searches `claims`, in their scope's order, for every one of `words`
/// (none of them empty of letters and digits, as [`words`] gives them),
/// in a table filled in memory and dropped with the answer: every claim
/// matched, best first.
pub fn search(claims: &[Claim<'_>], words: &[String]) -> Result<Vec<Found>, Failure> {
    let failed =
        |error: rusqlite::Error| Failure::not_run(format!("the search could not run: {error}"));
    let conn = Connection::open_in_memory().map_err(failed)?;
    conn.execute(TABLE, []).map_err(failed)?;
    {
        let mut insert = conn
            .prepare(
                "INSERT INTO found (rowid, name, legacy, statement, section, scenarios, body)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )
            .map_err(failed)?;
        for (place, claim) in claims.iter().enumerate() {
            let rowid = i64::try_from(place).expect("a claim's place fits a rowid");
            insert
                .execute(rusqlite::params![
                    rowid,
                    claim.name,
                    claim.legacy.unwrap_or_default(),
                    claim.statement,
                    claim.section.unwrap_or_default(),
                    claim.scenarios.join("\n"),
                    claim.body.unwrap_or_default(),
                ])
                .map_err(failed)?;
        }
    }
    let mut rank = conn.prepare(RANK).map_err(failed)?;
    let rows = rank
        .query_map([query(words)], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(failed)?;
    let mut found = Vec::new();
    for row in rows {
        let (rowid, snippet) = row.map_err(failed)?;
        let place = usize::try_from(rowid).expect("a rowid is a claim's place");
        let excerpt = Some(one_line(&snippet))
            .filter(|excerpt| !excerpt.is_empty() && claims[place].body.is_some());
        found.push(Found { place, excerpt });
    }
    Ok(found)
}

/// The words of `boom search`'s refusal of a search with no word that
/// holds a letter or a digit, which the page gives too.
pub const NO_WORD: &str = "boom search needs a word holding a letter or a digit";

/// Where a document's paragraphs hold the words searched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Held {
    /// The first paragraph that holds each word, its place from 0.
    Paragraph(usize),
    /// No one paragraph holds each word, and the paragraphs together do,
    /// with the passage of the whole where the words fall.
    Across(Option<String>),
}

/// Searches `documents`, each a list of paragraphs' words, for every one
/// of `words`, with [`search`]'s matching: each paragraph a row, then
/// each document a row. Gives each document found, in the order given,
/// with the first paragraph that holds each word, or [`Held::Across`].
pub fn paragraphs(
    documents: &[Vec<String>],
    words: &[String],
) -> Result<Vec<(usize, Held)>, Failure> {
    let mut rows = Vec::new();
    let mut of = Vec::new();
    for (document, paragraphs) in documents.iter().enumerate() {
        for (place, text) in paragraphs.iter().enumerate() {
            rows.push(Claim {
                body: Some(text),
                ..Claim::default()
            });
            of.push((document, place));
        }
    }
    let mut first: Vec<Option<usize>> = vec![None; documents.len()];
    for found in search(&rows, words)? {
        let (document, place) = of[found.place];
        first[document] = Some(first[document].map_or(place, |earlier| earlier.min(place)));
    }
    let wholes: Vec<String> = documents
        .iter()
        .map(|paragraphs| paragraphs.join("\n\n"))
        .collect();
    let rows: Vec<Claim<'_>> = wholes
        .iter()
        .map(|whole| Claim {
            body: Some(whole),
            ..Claim::default()
        })
        .collect();
    let mut found: Vec<(usize, Held)> = search(&rows, words)?
        .into_iter()
        .map(|found| {
            let held = match first[found.place] {
                Some(place) => Held::Paragraph(place),
                None => Held::Across(found.excerpt),
            };
            (found.place, held)
        })
        .collect();
    found.sort_by_key(|(document, _)| *document);
    Ok(found)
}

/// A claim that stands, found by a search: the claim, its accepted
/// version, its imported number or id, and the passage of its body.
#[derive(Debug, Clone)]
pub struct Standing {
    /// The claim.
    pub claim: crate::judge::Claim,
    /// Its accepted version.
    pub version: crate::judge::Version,
    /// Its imported record's number or rule's id, where it has one.
    pub legacy: Option<String>,
    /// The passage of its body where the words fall.
    pub excerpt: Option<String>,
}

/// The claims that stand for `repository` and its `contexts`, or every
/// claim that stands where none is named, whose words hold every one of
/// `words`, best first: `boom search`'s answer, read from the views on
/// `conn` (claim-search).
pub fn standing(
    conn: &Connection,
    repository: Option<&str>,
    contexts: &[String],
    words: &[String],
) -> Result<Vec<Standing>, Failure> {
    let accepted = crate::store::accepted_for(conn, repository, contexts)?;
    // A body that is not text is searched as holding no word; one whose
    // bytes no longer match its fingerprint stops the search.
    let bodies = accepted
        .iter()
        .map(|(_, version)| match &version.proposal.body {
            Some(fingerprint) => Ok(String::from_utf8(crate::store::blob(conn, fingerprint)?).ok()),
            None => Ok(None),
        })
        .collect::<Result<Vec<_>, Failure>>()?;
    let searched: Vec<Claim<'_>> = accepted
        .iter()
        .zip(&bodies)
        .map(|((claim, version), body)| {
            let imported = version.imported.as_ref();
            Claim {
                name: &claim.name,
                legacy: imported.and_then(|imported| imported.legacy.as_deref()),
                statement: &version.proposal.statement,
                section: imported.and_then(|imported| imported.section.as_deref()),
                scenarios: &version.proposal.scenarios,
                body: body.as_deref(),
            }
        })
        .collect();
    let found = search(&searched, words)?;
    Ok(found
        .into_iter()
        .map(|found| {
            let (claim, version) = &accepted[found.place];
            Standing {
                claim: claim.clone(),
                version: version.clone(),
                legacy: searched[found.place].legacy.map(str::to_owned),
                excerpt: found.excerpt,
            }
        })
        .collect())
}

/// `text` with its runs of whitespace made one space, and none at
/// either end.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The words searched: those given, less each holding no letter or
/// digit, in the order given.
pub fn words(given: &[String]) -> Vec<String> {
    given
        .iter()
        .filter(|word| word.chars().any(char::is_alphanumeric))
        .cloned()
        .collect()
}

/// The FTS5 query of `words`: each written as a string, its `"`
/// doubled, so FTS5 reads no operator in it, and the strings joined by
/// spaces, which FTS5 reads as every one of them.
pub fn query(words: &[String]) -> String {
    words
        .iter()
        .map(|word| format!("\"{}\"", word.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn given(words: &[&str]) -> Vec<String> {
        words.iter().map(|&word| word.to_owned()).collect()
    }

    #[test]
    fn a_word_is_written_as_a_string() {
        assert_eq!(query(&given(&["currency"])), "\"currency\"");
        assert_eq!(query(&given(&["OR"])), "\"OR\"");
        assert_eq!(query(&given(&["currency*"])), "\"currency*\"");
        assert_eq!(query(&given(&["core.2"])), "\"core.2\"");
        assert_eq!(query(&given(&["(NEAR"])), "\"(NEAR\"");
    }

    #[test]
    fn a_quote_in_a_word_is_doubled() {
        assert_eq!(query(&given(&["\"amount"])), "\"\"\"amount\"");
        assert_eq!(query(&given(&["say\"so\""])), "\"say\"\"so\"\"\"");
    }

    #[test]
    fn a_word_with_no_letter_or_digit_is_dropped() {
        assert_eq!(
            words(&given(&["***", "amount", "-", "\"\"", "naïve", "2", "…"])),
            given(&["amount", "naïve", "2"])
        );
        assert!(words(&given(&["***", "--", "()"])).is_empty());
    }

    #[test]
    fn the_words_are_joined_so_each_must_be_found() {
        assert_eq!(
            query(&given(&["amount", "currency", "core.2"])),
            "\"amount\" \"currency\" \"core.2\""
        );
    }

    const SCENARIOS: &[String] = &[];

    fn claim<'a>(name: &'a str, statement: &'a str) -> Claim<'a> {
        Claim {
            name,
            statement,
            scenarios: SCENARIOS,
            ..Claim::default()
        }
    }

    /// The places `words` finds among `claims`, best first.
    fn places(claims: &[Claim<'_>], words: &[&str]) -> Vec<usize> {
        search(claims, &given(words))
            .unwrap()
            .iter()
            .map(|found| found.place)
            .collect()
    }

    #[test]
    fn a_stem_finds_another_form() {
        let claims = [
            claim("money-has-currency", "Every amount names its currency."),
            claim("orders-are-paid-once", "An order is paid once."),
        ];
        assert_eq!(places(&claims, &["amounts", "currencies"]), [0]);
        assert_eq!(places(&claims, &["ordered", "ONCE"]), [1]);
    }

    #[test]
    fn case_and_accents_are_aside() {
        let claims = [
            claim("dates-name-their-zone", "A naive date names its zone."),
            claim("money-has-currency", "Every amount names its currency."),
        ];
        assert_eq!(places(&claims, &["NAÏVE"]), [0]);
        assert_eq!(places(&claims, &["Naïve", "ZONE"]), [0]);
        assert_eq!(places(&claims, &["CURRENCY"]), [1]);
    }

    #[test]
    fn a_word_with_punctuation_is_found_as_its_parts_in_order() {
        let claims = [
            Claim {
                legacy: Some("core.1"),
                ..claim("the-log-is-appended", "An event is appended.")
            },
            Claim {
                legacy: Some("core.2"),
                ..claim("views-are-never-stored", "A view is computed.")
            },
            Claim {
                body: Some("The 2 core rules bind every change."),
                ..claim("the-rules-bind", "The rules bind.")
            },
        ];
        assert_eq!(places(&claims, &["core.2"]), [1]);
        assert_eq!(places(&claims, &["core"]), [0, 1, 2]);
        assert_eq!(places(&claims, &["2-core"]), [2]);
    }

    #[test]
    fn a_name_ranks_above_a_body() {
        let claims = [
            Claim {
                body: Some("An order is paid once, and a refund is paid against it."),
                ..claim("orders-are-paid-once", "An order is paid once.")
            },
            claim(
                "refund-never-exceeds-payment",
                "It never exceeds the payment.",
            ),
        ];
        assert_eq!(places(&claims, &["refund"]), [1, 0]);
    }

    #[test]
    fn equal_matches_keep_their_place() {
        let claims = [
            claim("gamma-kept", "Gamma is kept."),
            claim("alpha-kept", "Alpha is kept."),
            claim("beta-kept", "Beta is kept."),
        ];
        assert_eq!(places(&claims, &["kept"]), [0, 1, 2]);
    }

    fn documents(given: &[&[&str]]) -> Vec<Vec<String>> {
        given
            .iter()
            .map(|paragraphs| paragraphs.iter().map(|&text| text.to_owned()).collect())
            .collect()
    }

    #[test]
    fn the_first_paragraph_that_holds_each_word() {
        let documents = documents(&[
            &["Nothing here.", "A bolt lands.", "Bolts land on green."],
            &["No word of it."],
            &["The bolt waits.", "A bolt is held."],
        ]);
        assert_eq!(
            paragraphs(&documents, &given(&["BOLTS"])).unwrap(),
            [(0, Held::Paragraph(1)), (2, Held::Paragraph(0))]
        );
        assert_eq!(
            paragraphs(&documents, &given(&["bolt", "green"])).unwrap(),
            [(0, Held::Paragraph(2))]
        );
    }

    #[test]
    fn words_in_two_paragraphs() {
        let documents = documents(&[&["A bolt.", "Then", "The landing comes."]]);
        let found = paragraphs(&documents, &given(&["bolt", "landing"])).unwrap();
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(matches!(found[0], (0, Held::Across(_))), "{found:?}");
        assert!(paragraphs(&documents, &given(&["bolt", "absent"]))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn an_excerpt_is_the_bodys_passage_on_one_line() {
        let body = "# Amounts in cents\n\nAmounts are kept as integers of the smallest unit,\n\
                    \tthe cent,   so no rounding ever loses a cent between two\n\
                    ledgers that add the same payments in another order, and every\n\
                    report sums to the same total whatever adds it up first.\n";
        let claims = [
            Claim {
                body: Some(body),
                ..claim("amounts-in-cents", "Amounts are kept in integer cents.")
            },
            claim("rounding-is-banned", "Nothing is rounded."),
            Claim {
                body: Some(" \n "),
                ..claim("rounding-waits", "Rounding waits.")
            },
        ];
        let found = search(&claims, &given(&["rounding"])).unwrap();
        let excerpt = |place| {
            found
                .iter()
                .find(|found| found.place == place)
                .unwrap()
                .excerpt
                .clone()
        };
        let passage = excerpt(0).unwrap();
        assert!(passage.contains("rounding"), "{passage}");
        assert!(passage.contains('…'), "{passage}");
        assert!(!passage.contains(char::is_control), "{passage:?}");
        assert!(!passage.contains("  "), "{passage:?}");
        assert!(!passage.contains("# Amounts"), "{passage}");
        assert_eq!(passage, passage.trim(), "{passage:?}");
        assert_eq!(excerpt(1), None);
        assert_eq!(excerpt(2), None);
    }
}
