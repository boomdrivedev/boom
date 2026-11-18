//! The tests' keys: security keys made in software from seeds a test
//! names, signing as an authenticator does with the flags the test
//! chooses, and plain `ssh-ed25519` keys for the refusals (design 8).
//! No key is committed; each is made when its test runs (core.3).
//! No one test file uses all of it.
#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use boom::sign::{Keys, Signer};
use boom::Failure;
use ed25519_dalek::Signer as _;
use sha2::{Digest, Sha256};
use ssh_key::public::{Ed25519PublicKey, KeyData, SkEd25519};
use ssh_key::{Algorithm, HashAlg, LineEnding, PublicKey, Signature, SshSig};

/// The flags of a signature made with its person present.
pub const PRESENT: u8 = 0x01;
/// The flags of a signature made with no one present.
pub const ABSENT: u8 = 0x00;

/// The application a security key made by `ssh-keygen -t ed25519-sk` names.
const APPLICATION: &str = "ssh:";

#[derive(Clone)]
struct SoftKey {
    secret: [u8; 32],
    security: bool,
    flags: u8,
}

impl SoftKey {
    fn signing_key(&self) -> ed25519_dalek::SigningKey {
        ed25519_dalek::SigningKey::from_bytes(&self.secret)
    }

    fn public_key(&self) -> PublicKey {
        let point = Ed25519PublicKey(self.signing_key().verifying_key().to_bytes());
        let data = if self.security {
            KeyData::SkEd25519(SkEd25519::new(point, APPLICATION))
        } else {
            KeyData::Ed25519(point)
        };
        PublicKey::new(data, "")
    }

    fn sign(&self, namespace: &str, message: &[u8]) -> String {
        let signed_data = SshSig::signed_data(namespace, HashAlg::Sha512, message).unwrap();
        let signature = if self.security {
            // What an authenticator signs: the application's hash, the
            // flags, the counter and the message's hash.
            let counter: u32 = 1;
            let mut data = Vec::new();
            data.extend_from_slice(&Sha256::digest(APPLICATION.as_bytes()));
            data.push(self.flags);
            data.extend_from_slice(&counter.to_be_bytes());
            data.extend_from_slice(&Sha256::digest(&signed_data));
            let mut raw = self.signing_key().sign(&data).to_bytes().to_vec();
            raw.push(self.flags);
            raw.extend_from_slice(&counter.to_be_bytes());
            Signature::new(Algorithm::SkEd25519, raw).unwrap()
        } else {
            let raw = self.signing_key().sign(&signed_data).to_bytes().to_vec();
            Signature::new(Algorithm::Ed25519, raw).unwrap()
        };
        SshSig::new(
            self.public_key().key_data().clone(),
            namespace,
            HashAlg::Sha512,
            signature,
        )
        .unwrap()
        .to_pem(LineEnding::LF)
        .unwrap()
    }
}

/// Keys made in software, each at a path in a test's folder with its
/// public half beside it, `<path>.pub`.
pub struct SoftKeys {
    dir: PathBuf,
    keys: RefCell<HashMap<PathBuf, SoftKey>>,
}

impl SoftKeys {
    /// Keys whose files are in `dir`.
    pub fn new(dir: &Path) -> Self {
        SoftKeys {
            dir: dir.to_owned(),
            keys: RefCell::new(HashMap::new()),
        }
    }

    fn make(&self, name: &str, seed: u8, security: bool) -> PathBuf {
        let key = SoftKey {
            secret: [seed; 32],
            security,
            flags: PRESENT,
        };
        let path = self.dir.join(name);
        key.public_key()
            .write_openssh_file(&boom::sign::public_key_path(&path))
            .unwrap();
        self.keys.borrow_mut().insert(path.clone(), key);
        path
    }

    /// A security key, `sk-ssh-ed25519@openssh.com`, from `seed`,
    /// signing with its person present until a test sets its flags.
    pub fn security_key(&self, name: &str, seed: u8) -> PathBuf {
        self.make(name, seed, true)
    }

    /// A plain key, `ssh-ed25519`, from `seed`.
    pub fn plain_key(&self, name: &str, seed: u8) -> PathBuf {
        self.make(name, seed, false)
    }

    /// Sets the flags the key at `key` signs with.
    pub fn set_flags(&self, key: &Path, flags: u8) {
        self.keys.borrow_mut().get_mut(key).unwrap().flags = flags;
    }

    /// The public key of the key at `key`.
    pub fn public_key(&self, key: &Path) -> PublicKey {
        self.keys.borrow()[key].public_key()
    }

    /// The public key file beside `key`.
    pub fn public_key_file(&self, key: &Path) -> PathBuf {
        boom::sign::public_key_path(key)
    }

    /// Signs `message` with the key at `key` in `namespace`.
    pub fn sign_with(&self, key: &Path, namespace: &str, message: &[u8]) -> String {
        self.keys.borrow()[key].sign(namespace, message)
    }
}

struct SoftSigner {
    key: SoftKey,
    public_key: PublicKey,
}

impl Signer for SoftSigner {
    fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    fn sign(&self, message: &[u8]) -> Result<String, Failure> {
        Ok(self.key.sign(boom::sign::NAMESPACE, message))
    }
}

impl Keys for SoftKeys {
    fn signer(&self, key: &Path) -> Result<Box<dyn Signer + '_>, Failure> {
        let soft = self.keys.borrow().get(key).cloned().ok_or_else(|| {
            Failure::refused(format!(
                "the public key {}.pub cannot be read",
                key.display()
            ))
        })?;
        Ok(Box::new(SoftSigner {
            public_key: soft.public_key(),
            key: soft,
        }))
    }
}

/// A test's own folder: its keys, its store, and the environment its
/// commands run in, `XDG_DATA_HOME` and `HOME` inside it.
pub struct Place {
    /// The folder, removed when the test ends.
    pub dir: tempfile::TempDir,
    /// The test's keys, in `<dir>/keys`.
    pub keys: SoftKeys,
    /// The environment commands run in.
    pub env: boom::cli::Env,
    /// The store's path, `<dir>/boom.sqlite`.
    pub store: PathBuf,
}

impl Place {
    /// A new folder with no store in it.
    pub fn new() -> Place {
        let dir = tempfile::tempdir().unwrap();
        let keys_dir = dir.path().join("keys");
        std::fs::create_dir(&keys_dir).unwrap();
        let env = boom::cli::Env {
            xdg_data_home: Some(dir.path().join("data")),
            home: Some(dir.path().join("home")),
            cwd: Some(dir.path().to_owned()),
            clock: boom::cli::Clock::Fixed(1_763_424_000),
        };
        let store = dir.path().join("boom.sqlite");
        Place {
            keys: SoftKeys::new(&keys_dir),
            env,
            store,
            dir,
        }
    }

    /// Sets the clock commands read.
    pub fn at(&mut self, at: u64) {
        self.env.clock = boom::cli::Clock::Fixed(at);
    }

    /// A security key, its path as an argument.
    pub fn key(&self, name: &str, seed: u8) -> String {
        path_arg(&self.keys.security_key(name, seed))
    }

    /// The store's path as an argument.
    pub fn store_arg(&self) -> String {
        path_arg(&self.store)
    }

    /// Runs `args` with no `--store`, signing with the test's keys.
    pub fn run_bare(&self, args: &[&str]) -> boom::cli::Outcome {
        boom::cli::run(args.iter().copied(), &self.keys, &self.env)
    }

    /// Runs `args` on the place's store with `keys`.
    pub fn run_with(&self, keys: &dyn Keys, args: &[&str]) -> boom::cli::Outcome {
        let store = self.store_arg();
        let mut all: Vec<&str> = args.to_vec();
        all.extend(["--store", &store]);
        boom::cli::run(all, keys, &self.env)
    }

    /// Runs `args` on the place's store, signing with the test's keys.
    pub fn run(&self, args: &[&str]) -> boom::cli::Outcome {
        self.run_with(&self.keys, args)
    }

    /// Runs `args` with `--json`, asserting it answers, and gives its
    /// answer.
    pub fn json(&self, args: &[&str]) -> serde_json::Value {
        let mut all: Vec<&str> = args.to_vec();
        all.push("--json");
        let outcome = self.run(&all);
        assert_eq!(outcome.code, 0, "{args:?}: {outcome:?}");
        serde_json::from_str(&outcome.stdout).unwrap()
    }

    /// Makes the store with `ada` its first owner, giving ada's key.
    pub fn made(&self) -> String {
        let ada = self.key("ada", 1);
        let outcome = self.run(&["init", "--person", "ada", "--key", &ada]);
        assert_eq!(outcome.code, 0, "{outcome:?}");
        ada
    }

    /// Registers `name` with a new security key, signed by `owner`'s
    /// key, giving the new key.
    pub fn registered(&self, owner: &str, name: &str, seed: u8) -> String {
        let key = self.key(name, seed);
        let public = format!("{key}.pub");
        let outcome = self.run(&["register", name, &public, "--key", owner]);
        assert_eq!(outcome.code, 0, "{outcome:?}");
        key
    }

    /// An agent's proposal of a first version of an invariant, giving
    /// its version.
    pub fn proposed(&self, name: &str, statement: &str) -> String {
        let answer = self.json(&[
            "propose",
            "invariant",
            name,
            statement,
            "--scenario",
            "Given an amount, then it names its currency.",
            "--provenance",
            "run-1",
        ]);
        answer["version"].as_str().unwrap().to_owned()
    }

    /// The log's entries, as `boom log --json` gives them.
    pub fn log(&self) -> Vec<serde_json::Value> {
        self.json(&["log"])["entries"].as_array().unwrap().clone()
    }

    /// The claim `name`, as `boom show --json` gives it.
    pub fn show(&self, target: &str) -> serde_json::Value {
        self.json(&["show", target])
    }

    /// A connection of the test's own to the store's file.
    pub fn sqlite(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(&self.store).unwrap()
    }
}

/// A path as an argument.
pub fn path_arg(path: &Path) -> String {
    path.to_str().unwrap().to_owned()
}

impl Place {
    /// Runs `args`, asserting it refuses saying `words` and appends
    /// nothing.
    pub fn refused(&self, args: &[&str], words: &str) -> boom::cli::Outcome {
        let before = self.log();
        let outcome = self.run(args);
        assert_eq!(outcome.code, 2, "{args:?}: {outcome:?}");
        assert!(outcome.stderr.contains(words), "{words}: {outcome:?}");
        assert_eq!(self.log(), before, "{args:?} appended");
        outcome
    }

    /// The acts of the log's entries, in order.
    pub fn acts(&self) -> Vec<String> {
        self.log()
            .iter()
            .map(|entry| entry["act"].as_str().unwrap().to_owned())
            .collect()
    }
}

/// The scenario the tests' proposals give.
pub const SCENARIO: &str = "Given an amount, then it names its currency.";

impl Place {
    /// Writes `text` to a file in the place, giving its path.
    pub fn file(&self, name: &str, text: &str) -> String {
        let path = self.dir.path().join(name);
        std::fs::write(&path, text).unwrap();
        path_arg(&path)
    }

    /// An agent's proposal of a first version of `name` with the body
    /// in `body`, giving its answer.
    pub fn proposed_with_body(&self, name: &str, statement: &str, body: &str) -> serde_json::Value {
        self.json(&[
            "propose",
            "invariant",
            name,
            statement,
            "--scenario",
            SCENARIO,
            "--body",
            body,
            "--provenance",
            "run-1",
        ])
    }
}

/// A body of a few lines, with a character past ASCII.
pub const BODY: &str =
    "# Money\n\nAn amount is a number and a currency,\nnever a number alone: 40.00 €.\n";

impl Place {
    /// An agent's proposal of `statement` amending `base`, giving its
    /// version.
    pub fn amended(&self, name: &str, statement: &str, base: &str) -> String {
        let answer = self.json(&[
            "propose",
            "invariant",
            name,
            statement,
            "--scenario",
            SCENARIO,
            "--amends",
            base,
            "--provenance",
            "run-2",
        ]);
        assert_eq!(answer["base"], base);
        answer["version"].as_str().unwrap().to_owned()
    }

    /// Accepts `name@version` with `key`, asserting it answers.
    pub fn accepted(&self, name: &str, version: &str, key: &str) -> serde_json::Value {
        self.json(&["accept", &format!("{name}@{version}"), "--key", key])
    }

    /// The status `boom show` gives `name@version`.
    pub fn status(&self, name: &str, version: &str) -> String {
        self.show(&format!("{name}@{version}"))["status"]
            .as_str()
            .unwrap()
            .to_owned()
    }
}

impl Place {
    /// A store of three entries: ada's registration and two agents'
    /// proposals, of `money-has-currency` and `refund-within-payment`.
    pub fn three_entries(&self) -> String {
        let ada = self.made();
        self.proposed("money-has-currency", "Every amount names its currency.");
        self.proposed(
            "refund-within-payment",
            "A refund never exceeds the payment.",
        );
        ada
    }

    /// The log's rows as the file holds them, `(seq, signed)`.
    pub fn rows(&self) -> Vec<(i64, String)> {
        let conn = self.sqlite();
        let mut statement = conn
            .prepare("SELECT seq, signed FROM log ORDER BY seq")
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    }
}

impl Place {
    /// Appends `entry` to the log as another program would, chained
    /// correctly and leaving the views as they are, giving its signed
    /// part.
    pub fn append_by_hand(&self, entry: &boom::entry::Entry, sign_with: Option<&Path>) -> String {
        let signed = entry.signed_part();
        let signature = sign_with.map(|key| self.keys.sign_with(key, "boom", signed.as_bytes()));
        let conn = self.sqlite();
        let (seq, prev): (i64, String) = conn
            .query_row(
                "SELECT seq, hash FROM log ORDER BY seq DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let hash = boom::entry::hash(&prev, &signed, signature.as_deref());
        conn.execute(
            "INSERT INTO log (seq, prev, hash, signed, signature) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![seq + 1, prev, hash, signed, signature],
        )
        .unwrap();
        signed
    }
}

/// An accept of `name@version` by `person`, at a fixed time.
pub fn accept_entry(person: &str, name: &str, version: &str) -> boom::entry::Entry {
    boom::entry::Entry {
        at: 1_763_424_500,
        actor: boom::entry::Actor::Person(person.to_owned()),
        act: boom::entry::Act::Accepted {
            name: name.to_owned(),
            version: version.to_owned(),
        },
    }
}
