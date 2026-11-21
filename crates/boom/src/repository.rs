//! The repository of a session (repository-rules, "The repository is
//! named or declared"; repository-settings): the one `--repository`
//! names, else the one `BOOM_REPOSITORY` names, which a launcher such as
//! flume sets, else the registered repository whose remote matches a
//! remote of the git repository that holds the working folder. No file
//! in a repository names it: `.config/boom.toml` is not read.

use std::path::{Path, PathBuf};

use crate::judge::Repository;
use crate::Failure;

/// The variable a launcher sets to name the repository of a session.
pub const LAUNCHER: &str = "BOOM_REPOSITORY";

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

/// A remote in its one form: the host in lowercase, then the path, with
/// no scheme, no user, no port and no `.git` at its end
/// (repository-settings, "A remote in one form"). `https://`, `ssh://`
/// and `git@<host>:` forms are read; anything else gives none.
pub fn remote_form(remote: &str) -> Option<String> {
    let remote = remote.trim();
    let (host, path) = if let Some(rest) = remote
        .strip_prefix("https://")
        .or_else(|| remote.strip_prefix("http://"))
        .or_else(|| remote.strip_prefix("ssh://"))
        .or_else(|| remote.strip_prefix("git://"))
    {
        rest.split_once('/')?
    } else if let Some((user_host, path)) = remote.split_once(':') {
        if user_host.contains('/') || path.starts_with("//") {
            return None;
        }
        (user_host, path)
    } else {
        return None;
    };
    let host = host.rsplit_once('@').map_or(host, |(_, host)| host);
    let host = host.split_once(':').map_or(host, |(host, _)| host);
    let path = path.trim_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    if host.is_empty() || path.is_empty() || path.split('/').any(str::is_empty) {
        return None;
    }
    Some(format!("{}/{path}", host.to_ascii_lowercase()))
}

/// The common git folder of the repository at `root`: `.git` itself, or
/// for a worktree the folder its `gitdir` names, followed to its
/// `commondir`.
fn common_git_dir(root: &Path) -> Option<PathBuf> {
    let dot_git = root.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }
    let text = std::fs::read_to_string(&dot_git).ok()?;
    let gitdir = text.trim().strip_prefix("gitdir:")?.trim();
    let gitdir = root.join(gitdir);
    match std::fs::read_to_string(gitdir.join("commondir")) {
        Ok(common) => Some(gitdir.join(common.trim())),
        Err(_) => Some(gitdir),
    }
}

/// The remotes of the git repository that holds `cwd`, each as its
/// configuration names it, read from that repository's `config` with no
/// process started (design 3).
pub fn remotes(cwd: &Path) -> Vec<String> {
    let Some(config) = root(cwd)
        .and_then(|root| common_git_dir(&root))
        .and_then(|dir| std::fs::read_to_string(dir.join("config")).ok())
    else {
        return Vec::new();
    };
    let mut in_remote = false;
    let mut found = Vec::new();
    for line in config.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_remote = line.starts_with("[remote ");
            continue;
        }
        if !in_remote {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            if key.trim() == "url" {
                found.push(value.trim().trim_matches('"').to_owned());
            }
        }
    }
    found
}

/// The repository of a session: `named`, refused unless kebab case; else
/// `launcher`, refused unless the store registers it; else the one
/// registered repository whose remote matches a remote of the git
/// repository holding `cwd`, refused where two match; else none.
pub fn find(
    named: Option<&str>,
    launcher: Option<&str>,
    cwd: Option<&Path>,
    registered: &[Repository],
) -> Result<Option<String>, Failure> {
    if let Some(name) = named {
        check_name(name).map_err(Failure::Refused)?;
        return Ok(Some(name.to_owned()));
    }
    if let Some(name) = launcher {
        if !registered.iter().any(|repository| repository.name == name) {
            return Err(Failure::refused(format!(
                "{LAUNCHER} names {name}, a repository this store does not register: register it with boom repo add"
            )));
        }
        return Ok(Some(name.to_owned()));
    }
    let Some(cwd) = cwd else {
        return Ok(None);
    };
    let forms: Vec<String> = remotes(cwd)
        .iter()
        .filter_map(|remote| remote_form(remote))
        .collect();
    let mut matched: Vec<&str> = registered
        .iter()
        .filter(|repository| forms.contains(&repository.remote))
        .map(|repository| repository.name.as_str())
        .collect();
    matched.dedup();
    match matched.as_slice() {
        [] => Ok(None),
        [one] => Ok(Some((*one).to_owned())),
        several => Err(Failure::refused(format!(
            "this folder's remotes match {}: name one with --repository",
            words(
                &several
                    .iter()
                    .map(|name| (*name).to_owned())
                    .collect::<Vec<_>>()
            )
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registered(name: &str, remote: &str) -> Repository {
        Repository {
            name: name.to_owned(),
            remote: remote.to_owned(),
            session: true,
            set_by: "ada".to_owned(),
            set_in: 2,
        }
    }

    fn git_repository(at: &Path, remotes: &[(&str, &str)]) {
        std::fs::create_dir_all(at.join(".git")).unwrap();
        let mut config = String::from("[core]\n\tbare = false\n");
        for (name, url) in remotes {
            config.push_str(&format!("[remote \"{name}\"]\n\turl = {url}\n\tfetch = +refs/heads/*:refs/remotes/{name}/*\n"));
        }
        std::fs::write(at.join(".git/config"), config).unwrap();
    }

    #[test]
    fn three_forms_give_one_remote() {
        for remote in [
            "git@github.com:boomdrivedev/sluice.git",
            "https://github.com/boomdrivedev/sluice",
            "ssh://git@GitHub.com/boomdrivedev/sluice.git",
            "https://github.com/boomdrivedev/sluice.git/",
        ] {
            assert_eq!(
                remote_form(remote).as_deref(),
                Some("github.com/boomdrivedev/sluice"),
                "{remote}"
            );
        }
        assert_eq!(remote_form("not a remote"), None);
        assert_eq!(remote_form("/srv/git/sluice.git"), None);
    }

    #[test]
    fn a_remote_of_the_folder_names_its_repository() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("sluice");
        git_repository(
            &repo,
            &[("origin", "git@github.com:boomdrivedev/sluice.git")],
        );
        let deep = repo.join("crates/sluice");
        std::fs::create_dir_all(&deep).unwrap();
        let store = [registered("sluice", "github.com/boomdrivedev/sluice")];
        assert_eq!(
            find(None, None, Some(&deep), &store).unwrap().as_deref(),
            Some("sluice")
        );
        assert_eq!(find(None, None, Some(&deep), &[]).unwrap(), None);
    }

    #[test]
    fn a_worktree_reads_its_common_config() {
        let dir = tempfile::tempdir().unwrap();
        let bare = dir.path().join("sluice.git");
        std::fs::create_dir_all(bare.join("worktrees/main")).unwrap();
        std::fs::write(
            bare.join("config"),
            "[remote \"origin\"]\n\turl = https://github.com/boomdrivedev/sluice\n",
        )
        .unwrap();
        std::fs::write(bare.join("worktrees/main/commondir"), "../..\n").unwrap();
        let worktree = dir.path().join("main");
        std::fs::create_dir_all(&worktree).unwrap();
        std::fs::write(
            worktree.join(".git"),
            format!("gitdir: {}\n", bare.join("worktrees/main").display()),
        )
        .unwrap();
        assert_eq!(
            remotes(&worktree),
            vec!["https://github.com/boomdrivedev/sluice".to_owned()]
        );
    }

    #[test]
    fn the_order_named_then_launcher_then_remote() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("sluice");
        git_repository(
            &repo,
            &[("origin", "https://github.com/boomdrivedev/sluice")],
        );
        let store = [
            registered("sluice", "github.com/boomdrivedev/sluice"),
            registered("flume", "github.com/boomdrivedev/flume"),
        ];
        let found = |named, launcher| find(named, launcher, Some(&repo), &store).unwrap();
        assert_eq!(
            found(Some("ledger"), Some("flume")).as_deref(),
            Some("ledger")
        );
        assert_eq!(found(None, Some("flume")).as_deref(), Some("flume"));
        assert_eq!(found(None, None).as_deref(), Some("sluice"));
        assert!(matches!(
            find(None, Some("nowhere"), Some(&repo), &store),
            Err(Failure::Refused(why)) if why.contains("nowhere")
        ));
    }

    #[test]
    fn two_matches_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("both");
        git_repository(
            &repo,
            &[
                ("origin", "https://github.com/boomdrivedev/sluice"),
                ("other", "https://github.com/boomdrivedev/flume"),
            ],
        );
        let store = [
            registered("sluice", "github.com/boomdrivedev/sluice"),
            registered("flume", "github.com/boomdrivedev/flume"),
        ];
        assert!(matches!(
            find(None, None, Some(&repo), &store),
            Err(Failure::Refused(why)) if why.contains("sluice") && why.contains("flume")
        ));
    }

    #[test]
    fn a_declaration_file_is_not_read() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("shop");
        git_repository(&repo, &[]);
        std::fs::create_dir_all(repo.join(".config")).unwrap();
        std::fs::write(repo.join(".config/boom.toml"), "repository = \"shop\"\n").unwrap();
        assert_eq!(find(None, None, Some(&repo), &[]).unwrap(), None);
    }
}
