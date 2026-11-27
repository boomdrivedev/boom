//! Boom's own settings on a machine, in
//! `$XDG_CONFIG_HOME/boom/config.toml`, under `$HOME/.config` where
//! `XDG_CONFIG_HOME` is unset (persons-acts, "A person's key in Boom's
//! settings"). Its one table, `key`, names the person's security key,
//! `path`, and the program that gives the key's passphrase, `askpass`,
//! which `ssh-keygen` runs as `SSH_ASKPASS`. Each path is absolute, or
//! starts with `~/`, read as the home; boom reads no `$VAR` in it. No
//! command writes the file, and the log holds none of it.
//!
//! ```toml
//! key = { path = "~/.ssh/id_ecdsa_sk", askpass = "~/.local/bin/boom-askpass" }
//! ```

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::cli::Env;
use crate::judge::State;
use crate::sign::{self, Key};
use crate::Failure;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    key: Option<KeyTable>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyTable {
    path: String,
    askpass: Option<String>,
}

/// The settings file of `env`: `$XDG_CONFIG_HOME/boom/config.toml`, else
/// `$HOME/.config/boom/config.toml`; none where both are unset.
pub fn path(env: &Env) -> Option<PathBuf> {
    let config = env
        .xdg_config_home
        .clone()
        .or_else(|| env.home.as_ref().map(|home| home.join(".config")))?;
    Some(config.join("boom").join("config.toml"))
}

/// A path of the file as boom reads it: absolute, or `~/` read as the
/// home. Refused otherwise, naming the setting.
fn read_path(written: &str, setting: &str, file: &Path, env: &Env) -> Result<PathBuf, Failure> {
    let refused = |why: &str| {
        Failure::refused(format!(
            "{}: key.{setting} {why}: {written:?}",
            file.display()
        ))
    };
    if let Some(rest) = written.strip_prefix("~/") {
        let home = env
            .home
            .as_ref()
            .ok_or_else(|| refused("starts with ~/, and HOME is unset"))?;
        return Ok(home.join(rest));
    }
    let path = PathBuf::from(written);
    if !path.is_absolute() {
        return Err(refused("is a path that is absolute or starts with ~/"));
    }
    Ok(path)
}

/// The person's key that the settings name, with its passphrase program
/// where they name one; none where there is no file or the file has no
/// `key`. Refused where the file cannot be read, is not TOML, holds a
/// key boom does not read, or gives a path that is not absolute.
pub fn key(env: &Env) -> Result<Option<Key>, Failure> {
    let Some(file) = path(env) else {
        return Ok(None);
    };
    let text = match std::fs::read_to_string(&file) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(Failure::refused(format!(
                "Boom's settings {} cannot be read: {error}",
                file.display()
            )))
        }
    };
    let read: File = toml::from_str(&text).map_err(|error| {
        Failure::refused(format!(
            "Boom's settings {} are not read: {}",
            file.display(),
            error.message()
        ))
    })?;
    let Some(table) = read.key else {
        return Ok(None);
    };
    Ok(Some(Key {
        path: read_path(&table.path, "path", &file, env)?,
        askpass: table
            .askpass
            .as_deref()
            .map(|askpass| read_path(askpass, "askpass", &file, env))
            .transpose()?,
    }))
}

/// How a command that a person runs on this machine names their key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Signing {
    /// With `--key <key>`: Boom's settings hold no key that the store
    /// registers.
    #[default]
    Given,
    /// With no `--key`: Boom's settings hold the key of this person,
    /// whom the store registers.
    Settings {
        /// The person whose key it is.
        person: String,
    },
}

impl Signing {
    /// The words after a command for the key: ` --key <key>`, or none.
    pub fn key_words(&self) -> &'static str {
        match self {
            Signing::Given => " --key <key>",
            Signing::Settings { .. } => "",
        }
    }
}

/// How the commands that a person runs here name their key: with none
/// where Boom's settings hold a key whose public half, `<path>.pub`,
/// `state` registers to a person; else with `--key <key>`.
pub fn signing(env: &Env, state: &State) -> Signing {
    let Ok(Some(key)) = self::key(env) else {
        return Signing::Given;
    };
    let Ok(public_key) = sign::read_public_key(&sign::public_key_path(&key.path)) else {
        return Signing::Given;
    };
    match state.person_by_key(&public_key) {
        Some(person) => Signing::Settings {
            person: person.name.clone(),
        },
        None => Signing::Given,
    }
}
