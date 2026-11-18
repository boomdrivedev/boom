//! Claims: their kinds, their names, their statements and scenarios,
//! and a version as the fingerprint of its content (claim-versions;
//! design 5).

use serde::{Deserialize, Serialize};

/// The kind of a claim, fixed with its name at its first proposal.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Something that holds of the work at every moment.
    Invariant,
    /// A choice made, and why.
    Decision,
    /// Something taken as true and not checked.
    Assumption,
}

impl Kind {
    /// The kind's name as an entry writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Invariant => "invariant",
            Kind::Decision => "decision",
            Kind::Assumption => "assumption",
        }
    }
}

/// A version of a claim, as a `proposed` entry holds it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    /// The claim's name, kebab case.
    pub name: String,
    /// The claim's kind.
    pub kind: Kind,
    /// The version: the fingerprint of the rest.
    pub version: String,
    /// The version it amends, or none for a first version.
    pub base: Option<String>,
    /// The claim in one line.
    pub statement: String,
    /// Its scenarios, one at least.
    pub scenarios: Vec<String>,
    /// The fingerprint of its body, where it has one.
    pub body: Option<String>,
}

/// The content a version is the fingerprint of, in the order it is
/// written.
#[derive(Serialize)]
struct Content<'a> {
    kind: Kind,
    name: &'a str,
    statement: &'a str,
    scenarios: &'a [String],
    body: Option<&'a str>,
    base: Option<&'a str>,
}

impl Proposal {
    /// A version with its fingerprint computed from its content.
    pub fn new(
        kind: Kind,
        name: &str,
        statement: &str,
        scenarios: Vec<String>,
        body: Option<String>,
        base: Option<String>,
    ) -> Proposal {
        let mut proposal = Proposal {
            name: name.to_owned(),
            kind,
            version: String::new(),
            base,
            statement: statement.to_owned(),
            scenarios,
            body,
        };
        proposal.version = proposal.fingerprint();
        proposal
    }

    /// The first 12 hexadecimal characters of the SHA-256 of its kind,
    /// name, statement, scenarios, body's fingerprint and base.
    pub fn fingerprint(&self) -> String {
        let content = serde_json::to_string(&Content {
            kind: self.kind,
            name: &self.name,
            statement: &self.statement,
            scenarios: &self.scenarios,
            body: self.body.as_deref(),
            base: self.base.as_deref(),
        })
        .expect("a version's content is JSON");
        crate::sha256_hex(content.as_bytes())[..12].to_owned()
    }

    /// The rules of a version's form: its name, its statement, its
    /// scenarios, and its version the fingerprint of its content.
    pub fn check(&self) -> Result<(), String> {
        check_name(&self.name)?;
        check_statement(&self.statement)?;
        check_scenarios(&self.scenarios)?;
        if self.version != self.fingerprint() {
            return Err(format!(
                "its version {} is not the fingerprint of its content, {}",
                self.version,
                self.fingerprint()
            ));
        }
        Ok(())
    }
}

/// A name is kebab case: lowercase letters and digits in words joined by
/// single hyphens.
pub fn check_name(name: &str) -> Result<(), String> {
    let kebab = !name.is_empty()
        && name.split('-').all(|word| {
            !word.is_empty()
                && word
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        });
    if kebab {
        Ok(())
    } else {
        Err(format!(
            "a name is kebab case, lowercase letters and digits in words joined by single hyphens: {name:?} is not"
        ))
    }
}

/// A statement is one line of words.
pub fn check_statement(statement: &str) -> Result<(), String> {
    if statement.contains(['\n', '\r']) {
        Err("a statement is one line: this one holds a line break".to_owned())
    } else if statement.trim().is_empty() {
        Err("a statement is one line of words: this one is empty".to_owned())
    } else {
        Ok(())
    }
}

/// A claim needs a scenario, and no scenario is empty.
pub fn check_scenarios(scenarios: &[String]) -> Result<(), String> {
    if scenarios.is_empty() {
        Err("a claim needs a scenario: give one with --scenario".to_owned())
    } else if scenarios.iter().any(|scenario| scenario.trim().is_empty()) {
        Err("a claim needs a scenario: one given is empty".to_owned())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn money(statement: &str, base: Option<&str>) -> Proposal {
        Proposal::new(
            Kind::Invariant,
            "money-has-currency",
            statement,
            vec!["Given an amount, it names its currency.".to_owned()],
            None,
            base.map(str::to_owned),
        )
    }

    #[test]
    fn a_name_is_kebab_case() {
        for name in ["money-has-currency", "billing", "rule-2", "a1-b2-c3"] {
            assert_eq!(check_name(name), Ok(()), "{name}");
        }
        for name in [
            "Refund_Within",
            "refund_within",
            "Money",
            "-money",
            "money-",
            "money--currency",
            "",
            "money currency",
        ] {
            let refused = check_name(name).unwrap_err();
            assert!(
                refused.starts_with("a name is kebab case"),
                "{name}: {refused}"
            );
        }
    }

    #[test]
    fn a_statement_is_one_line() {
        assert_eq!(check_statement("Every amount names its currency."), Ok(()));
        for statement in ["Two\nlines.", "Two\r\nlines.", "Cr\rlines."] {
            let refused = check_statement(statement).unwrap_err();
            assert!(refused.starts_with("a statement is one line"), "{refused}");
        }
        assert!(check_statement("  ").is_err());
    }

    #[test]
    fn a_claim_needs_a_scenario() {
        let refused = check_scenarios(&[]).unwrap_err();
        assert!(refused.starts_with("a claim needs a scenario"), "{refused}");
        assert!(check_scenarios(&[String::new()]).is_err());
        assert_eq!(check_scenarios(&["Given x, then y.".to_owned()]), Ok(()));
        let mut proposal = money("Every amount names its currency.", None);
        proposal.scenarios.clear();
        assert!(proposal.check().is_err());
    }

    #[test]
    fn a_version_is_the_fingerprint_of_its_content() {
        let first = money("Every amount names its currency.", None);
        assert_eq!(first.version.len(), 12);
        assert!(first.version.chars().all(|c| c.is_ascii_hexdigit()));
        let expected = crate::sha256_hex(
            br#"{"kind":"invariant","name":"money-has-currency","statement":"Every amount names its currency.","scenarios":["Given an amount, it names its currency."],"body":null,"base":null}"#,
        );
        assert_eq!(first.version, expected[..12]);
        assert_eq!(first, money("Every amount names its currency.", None));
        assert_ne!(
            first.version,
            money("Every amount names one currency.", None).version
        );
        assert_eq!(first.check(), Ok(()));
        let mut forged = first.clone();
        forged.version = "000000000000".to_owned();
        assert!(forged.check().unwrap_err().contains("not the fingerprint"));
    }

    #[test]
    fn the_base_is_part_of_a_versions_content() {
        let words = "Every amount names its currency.";
        let first = money(words, None);
        let on_base = money(words, Some(&first.version));
        assert_ne!(first.version, on_base.version);
        assert_ne!(on_base.version, money(words, Some("aaaaaaaaaaaa")).version);
    }
}
