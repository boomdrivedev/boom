//! Contexts (context-claims): a context is a claim that owns path globs,
//! each glob in one repository, `<repository>:<pattern>`. A claim binds
//! a context, and `boom query --path` gives the claims for a file from
//! the globs of the accepted contexts.
//!
//! A pattern has the form that the `paths:` line of a Claude Code rules
//! file reads. `globset` compiles it with `literal_separator`, so `*`
//! does not cross a `/`. Boom matches the text of a path and reads no
//! repository.

use globset::{GlobBuilder, GlobMatcher};
use rusqlite::Connection;

use crate::{repository, store, Failure};

/// A glob or a path, `whole`, split at its first `:`: its repository,
/// and the rest, its `part`.
fn split<'a>(whole: &str, part: &str, given: &'a str) -> Result<(&'a str, &'a str), String> {
    let form = format!("a {whole} is `<repository>:<{part}>`");
    let (name, rest) = given
        .split_once(':')
        .ok_or_else(|| format!("{form}: {given} names no repository"))?;
    repository::check_name(name).map_err(|why| format!("{form}: {given}: {why}"))?;
    if rest.is_empty() {
        return Err(format!("{form}: {given} names no {part}"));
    }
    if rest.starts_with('/')
        || rest.contains(['\\', '\n', '\r'])
        || rest.split('/').any(|part| part == "..")
    {
        return Err(format!(
            "a {part} is relative to the root of its repository, with no `/` at its start, no `..` part and no `\\`: {given} is not"
        ));
    }
    Ok((name, rest))
}

/// The matcher of a pattern: `*` in one part of a path, `**` in any
/// number of parts.
fn matcher(pattern: &str) -> Result<GlobMatcher, String> {
    GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map(|glob| glob.compile_matcher())
        .map_err(|error| format!("the pattern {pattern} is not a glob: {}", error.kind()))
}

/// A glob of a context, `<repository>:<pattern>`, split: a repository in
/// kebab case, and a pattern relative to its root that `globset`
/// compiles.
pub fn glob(given: &str) -> Result<(&str, &str), String> {
    let (name, pattern) = split("glob", "pattern", given)?;
    matcher(pattern)?;
    Ok((name, pattern))
}

/// A path of a file, `<repository>:<path>`, split.
pub fn path(given: &str) -> Result<(&str, &str), String> {
    split("path", "path", given)
}

/// Globs as a context keeps them: each one time, sorted.
pub fn globs(given: &[String]) -> Vec<String> {
    let mut kept = given.to_vec();
    kept.sort();
    kept.dedup();
    kept
}

/// The repositories that `globs` name, sorted, each one time: the
/// repositories that a context binds.
pub fn repositories(globs: &[String]) -> Vec<String> {
    let mut names: Vec<String> = globs
        .iter()
        .filter_map(|glob| glob.split_once(':'))
        .map(|(name, _)| name.to_owned())
        .collect();
    names.sort();
    names.dedup();
    names
}

/// The names of the contexts of `bindings`, each `<name>@<version>`.
pub fn names(bindings: &[String]) -> Vec<&str> {
    bindings
        .iter()
        .map(|binding| {
            binding
                .split_once('@')
                .map_or(binding.as_str(), |(name, _)| name)
        })
        .collect()
}

/// What a version binds, in words: the whole system, its repositories,
/// its contexts, or its repositories and its contexts.
pub fn binds_words(repositories: &[String], contexts: &[String]) -> String {
    let contexts_words = match contexts {
        [] => String::new(),
        [one] => format!("the context {one}"),
        several => format!("the contexts {}", repository::words(several)),
    };
    match (repositories.is_empty(), contexts.is_empty()) {
        (true, true) => "the whole system".to_owned(),
        (false, true) => repository::words(repositories),
        (true, false) => contexts_words,
        (false, false) => format!("{}, and {contexts_words}", repository::words(repositories)),
    }
}

/// The contexts that the file `path` of `repository` falls in, from the
/// globs of the accepted version of each context, as the views hold them
/// at the log's last entry: each `<name>@<version>`, sorted. This is the
/// one answer that every command asks; none matches a glob of its own.
pub fn falls_in(conn: &Connection, repository: &str, path: &str) -> Result<Vec<String>, Failure> {
    let mut found: Vec<String> = Vec::new();
    for (name, version, pattern) in store::accepted_globs(conn, repository)? {
        let context = format!("{name}@{version}");
        if found.contains(&context) {
            continue;
        }
        let matches = matcher(&pattern)
            .map_err(|why| Failure::not_run(format!("a view could not be read: {why}")))?
            .is_match(path);
        if matches {
            found.push(context);
        }
    }
    found.sort();
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(pattern: &str, path: &str) -> bool {
        matcher(pattern).unwrap().is_match(path)
    }

    #[test]
    fn a_glob_names_its_repository() {
        assert_eq!(
            glob("sluice:crates/sluice/src/overview*.rs"),
            Ok(("sluice", "crates/sluice/src/overview*.rs"))
        );
        for (given, words) in [
            ("crates/sluice/src/overview.rs", "names no repository"),
            ("Sluice:src/a.rs", "kebab case"),
            ("sluice:", "names no pattern"),
            ("sluice:../flume/src/status.rs", "relative"),
            ("sluice:src/../a.rs", "no `..` part"),
            ("sluice:/src/a.rs", "no `/` at its start"),
            ("sluice:src\\a.rs", "no `\\`"),
            ("sluice:src/[a.rs", "is not a glob"),
        ] {
            let refused = glob(given).unwrap_err();
            assert!(refused.contains(words), "{given}: {refused}");
        }
    }

    #[test]
    fn a_star_stays_in_one_part_of_a_path() {
        let pattern = "crates/sluice/src/overview*.rs";
        assert!(matches(pattern, "crates/sluice/src/overview.rs"));
        assert!(matches(pattern, "crates/sluice/src/overview_new.rs"));
        assert!(!matches(pattern, "crates/sluice/src/overview/watch.rs"));
        assert!(!matches(pattern, "crates/sluice/src/submit.rs"));
        assert!(matches("src/billing/**", "src/billing/refund.rs"));
        assert!(matches("src/billing/**", "src/billing/a/b/c.rs"));
        assert!(matches("src/{a,b}.rs", "src/b.rs"));
        assert!(matches("src/?.rs", "src/c.rs"));
        assert!(matches("src/[ab].rs", "src/a.rs"));
        assert!(!matches("src/[ab].rs", "src/c.rs"));
    }

    #[test]
    fn globs_are_kept_one_time_sorted() {
        let given = [
            "sluice:src/a.rs".to_owned(),
            "flume:src/status.rs".to_owned(),
            "sluice:src/a.rs".to_owned(),
        ];
        assert_eq!(globs(&given), ["flume:src/status.rs", "sluice:src/a.rs"]);
        assert_eq!(repositories(&given), ["flume", "sluice"]);
    }

    #[test]
    fn a_binding_in_words() {
        let shop = ["shop".to_owned()];
        let billing = ["shop-billing@aaaaaaaaaaaa".to_owned()];
        assert_eq!(binds_words(&[], &[]), "the whole system");
        assert_eq!(binds_words(&shop, &[]), "shop");
        assert_eq!(
            binds_words(&[], &billing),
            "the context shop-billing@aaaaaaaaaaaa"
        );
        assert_eq!(
            binds_words(&shop, &billing),
            "shop, and the context shop-billing@aaaaaaaaaaaa"
        );
        assert_eq!(names(&billing), ["shop-billing"]);
    }

    #[test]
    fn a_path_names_its_repository() {
        assert_eq!(path("sluice:src/a.rs"), Ok(("sluice", "src/a.rs")));
        assert!(path("src/a.rs")
            .unwrap_err()
            .contains("names no repository"));
        assert!(path("sluice:/src/a.rs").unwrap_err().contains("relative"));
        assert!(path("sluice:src/../a.rs").unwrap_err().contains("relative"));
    }
}
