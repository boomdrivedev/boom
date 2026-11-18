//! A repository's name: one given outright, or the one its
//! `.config/boom.toml` declares (repository-rules; design 4).
//!
//! The declaration is read from the nearest folder holding it, from the
//! working folder up to the nearest folder holding `.git`, and outside a
//! repository from none, so a host's own `~/.config` is never read as a
//! repository's (record 0008, point 5).

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::Failure;

/// Where a repository declares its name, from a folder of it.
pub const DECLARATION: &str = ".config/boom.toml";

/// A declaration's one key.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Declaration {
    repository: String,
}

/// A repository's name is kebab case, by the rule for a claim's name.
pub fn check_name(name: &str) -> Result<(), String> {
    crate::claim::check_name(name).map_err(|_| {
        format!(
            "a repository's name is kebab case, lowercase letters and digits in words joined by single hyphens: {name:?} is not"
        )
    })
}

/// What `repositories` binds, in words: the repositories by name, or the
/// whole system where there are none.
pub fn words(repositories: &[String]) -> String {
    match repositories {
        [] => "the whole system".to_owned(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The nearest folder holding `.git`, a file or a folder, from `cwd` up.
pub fn root(cwd: &Path) -> Option<PathBuf> {
    cwd.ancestors()
        .find(|folder| folder.join(".git").exists())
        .map(Path::to_owned)
}

/// The repository `named` names, refused unless kebab case; else the one
/// the nearest `.config/boom.toml` declares, from `cwd` up to the
/// repository's root; else none. A declaration that cannot be read is
/// refused, naming the file and what could not be read.
pub fn find(named: Option<&str>, cwd: Option<&Path>) -> Result<Option<String>, Failure> {
    if let Some(name) = named {
        check_name(name).map_err(Failure::Refused)?;
        return Ok(Some(name.to_owned()));
    }
    let Some(cwd) = cwd else {
        return Ok(None);
    };
    let Some(root) = root(cwd) else {
        return Ok(None);
    };
    let found = cwd
        .ancestors()
        .take_while(|folder| folder.starts_with(&root))
        .map(|folder| folder.join(DECLARATION))
        .find(|file| file.exists());
    found.map(|file| declared(&file)).transpose()
}

/// The name the declaration at `file` holds.
fn declared(file: &Path) -> Result<String, Failure> {
    let cannot = |what: String| {
        Failure::refused(format!(
            "the declaration {} cannot be read: {what}",
            file.display()
        ))
    };
    let text = std::fs::read_to_string(file).map_err(|error| cannot(error.to_string()))?;
    let declaration: Declaration =
        toml::from_str(&text).map_err(|error| cannot(error.message().to_owned()))?;
    check_name(&declaration.repository).map_err(cannot)?;
    Ok(declaration.repository)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(at: &Path) -> PathBuf {
        std::fs::create_dir_all(at).unwrap();
        at.to_owned()
    }

    fn declare(at: &Path, text: &str) {
        folder(&at.join(".config"));
        std::fs::write(at.join(DECLARATION), text).unwrap();
    }

    fn refused(found: Result<Option<String>, Failure>) -> String {
        match found {
            Err(Failure::Refused(why)) => why,
            other => panic!("not refused: {other:?}"),
        }
    }

    #[test]
    fn the_nearest_declaration_in_the_repository() {
        let dir = tempfile::tempdir().unwrap();
        let repo = folder(&dir.path().join("repo"));
        folder(&repo.join(".git"));
        declare(&repo, "repository = \"shop\"\n");
        let deep = folder(&repo.join("src/money"));
        assert_eq!(find(None, Some(&deep)).unwrap().as_deref(), Some("shop"));
        assert_eq!(find(None, Some(&repo)).unwrap().as_deref(), Some("shop"));
        let nested = folder(&repo.join("crates/ledger"));
        declare(&nested, "repository = \"ledger\"\n");
        let inside = folder(&nested.join("src"));
        assert_eq!(
            find(None, Some(&inside)).unwrap().as_deref(),
            Some("ledger")
        );
        assert_eq!(
            find(Some("billing"), Some(&inside)).unwrap().as_deref(),
            Some("billing")
        );
        // A worktree's `.git` is a file.
        let worktree = folder(&dir.path().join("worktree"));
        std::fs::write(worktree.join(".git"), "gitdir: ../repo/.git\n").unwrap();
        declare(&worktree, "repository = \"shop\"\n");
        assert_eq!(
            find(None, Some(&worktree)).unwrap().as_deref(),
            Some("shop")
        );
    }

    #[test]
    fn no_declaration_outside_a_repository() {
        let dir = tempfile::tempdir().unwrap();
        let outside = folder(&dir.path().join("outside"));
        declare(&outside, "repository = \"shop\"\n");
        assert_eq!(root(&outside), None);
        assert_eq!(find(None, Some(&outside)).unwrap(), None);
        assert_eq!(find(None, None).unwrap(), None);
    }

    #[test]
    fn no_declaration_above_the_repository() {
        let dir = tempfile::tempdir().unwrap();
        let above = folder(&dir.path().join("above"));
        declare(&above, "repository = \"shop\"\n");
        let repo = folder(&above.join("repo"));
        folder(&repo.join(".git"));
        let inside = folder(&repo.join("src"));
        assert_eq!(root(&inside).as_deref(), Some(repo.as_path()));
        assert_eq!(find(None, Some(&inside)).unwrap(), None);
    }

    #[test]
    fn a_declared_name_not_in_kebab_case_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let repo = folder(&dir.path().join("repo"));
        folder(&repo.join(".git"));
        declare(&repo, "repository = \"Shop\"\n");
        let why = refused(find(None, Some(&repo)));
        let file = repo.join(DECLARATION);
        assert!(why.contains(&file.display().to_string()), "{why}");
        assert!(why.contains("a repository's name is kebab case"), "{why}");
        let named = refused(find(Some("Shop_Front"), Some(&repo)));
        assert!(
            named.starts_with("a repository's name is kebab case"),
            "{named}"
        );
    }

    #[test]
    fn a_declaration_with_another_key_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let repo = folder(&dir.path().join("repo"));
        folder(&repo.join(".git"));
        declare(&repo, "repository = \"shop\"\ncontext = \"billing\"\n");
        let why = refused(find(None, Some(&repo)));
        let file = repo.join(DECLARATION);
        assert!(why.contains(&file.display().to_string()), "{why}");
        assert!(why.contains("context"), "{why}");
        declare(&repo, "repository = [\"shop\"\n");
        let why = refused(find(None, Some(&repo)));
        assert!(why.contains(&file.display().to_string()), "{why}");
    }
}
