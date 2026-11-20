//! Builds large stores through the store's own append path, and times
//! what the command line cannot sign for: an accept by a security key
//! made in software, as the tests make theirs.
//!
//! cargo run --release --example scale -- generate <dir> <entries>
//! cargo run --release --example scale -- behind <dir> <entries>
//! cargo run --release --example scale -- propose-accept <dir>

use std::path::{Path, PathBuf};
use std::time::Instant;

use boom::claim::{Kind, Proposal};
use boom::entry::{self, Act, Actor, Attach, Entry};
use boom::sign::{Keys, Signer};
use boom::store::{Append, Store};
use boom::Failure;
use ed25519_dalek::Signer as _;
use sha2::{Digest, Sha256};
use ssh_key::public::{KeyData, SkEd25519};
use ssh_key::{Algorithm, HashAlg, LineEnding, PublicKey, Signature, SshSig};

const CLAIMS: usize = 5_000;
const REPOSITORIES: usize = 20;
const BATCH: usize = 10_000;
const WORDS: [&str; 8] = [
    "amounts", "refunds", "orders", "invoices", "currency", "payments", "ledgers", "queues",
];

/// A security key made in software from a seed, signing with its person
/// present, as an authenticator does.
struct SoftKey {
    secret: [u8; 32],
}

impl SoftKey {
    fn signing_key(&self) -> ed25519_dalek::SigningKey {
        ed25519_dalek::SigningKey::from_bytes(&self.secret)
    }

    fn public_key(&self) -> PublicKey {
        let point =
            ssh_key::public::Ed25519PublicKey(self.signing_key().verifying_key().to_bytes());
        PublicKey::new(KeyData::SkEd25519(SkEd25519::new(point, "ssh:")), "")
    }

    fn sign(&self, message: &[u8]) -> String {
        let signed_data =
            SshSig::signed_data(boom::sign::NAMESPACE, HashAlg::Sha512, message).unwrap();
        let flags = 0x01;
        let counter: u32 = 1;
        let mut data = Vec::new();
        data.extend_from_slice(&Sha256::digest(b"ssh:"));
        data.push(flags);
        data.extend_from_slice(&counter.to_be_bytes());
        data.extend_from_slice(&Sha256::digest(&signed_data));
        let mut raw = self.signing_key().sign(&data).to_bytes().to_vec();
        raw.push(flags);
        raw.extend_from_slice(&counter.to_be_bytes());
        let signature = Signature::new(Algorithm::SkEd25519, raw).unwrap();
        SshSig::new(
            self.public_key().key_data().clone(),
            boom::sign::NAMESPACE,
            HashAlg::Sha512,
            signature,
        )
        .unwrap()
        .to_pem(LineEnding::LF)
        .unwrap()
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
        Ok(self.key.sign(message))
    }
}

/// Ada's key, the one key of every store this makes.
struct AdasKey;

impl Keys for AdasKey {
    fn signer(&self, _key: &Path) -> Result<Box<dyn Signer + '_>, Failure> {
        let key = SoftKey { secret: [1; 32] };
        Ok(Box::new(SoftSigner {
            public_key: key.public_key(),
            key,
        }))
    }
}

fn env(dir: &Path) -> boom::cli::Env {
    boom::cli::Env {
        xdg_data_home: Some(dir.join("data")),
        home: Some(dir.join("home")),
        cwd: Some(dir.to_owned()),
        clock: boom::cli::Clock::Fixed(1_763_424_000),
    }
}

fn store_of(dir: &Path) -> PathBuf {
    dir.join("boom.sqlite")
}

fn run(dir: &Path, args: &[&str]) -> boom::cli::Outcome {
    let store = store_of(dir);
    let mut all: Vec<&str> = args.to_vec();
    all.extend(["--store", store.to_str().unwrap()]);
    let outcome = boom::cli::run(all, &AdasKey, &env(dir));
    assert_eq!(outcome.code, 0, "{args:?}: {outcome:?}");
    outcome
}

fn proposal(claim: usize, version: usize, base: Option<String>) -> Proposal {
    let word = WORDS[claim % WORDS.len()];
    Proposal::new(
        Kind::Invariant,
        &format!("claim-{claim}"),
        &format!("Claim {claim} keeps its {word}, as version {version} says."),
        Vec::new(),
        None,
        base,
        None,
    )
}

/// One claim's life, step by step: propose a version, then accept it
/// (half the time), reject it (a quarter) or leave it waiting.
struct Life {
    standing: Option<String>,
    versions: usize,
    waiting: Option<String>,
}

fn generate(dir: &Path, entries: usize) {
    std::fs::create_dir_all(dir).unwrap();
    let key = dir.join("ada");
    let soft = SoftKey { secret: [1; 32] };
    soft.public_key()
        .write_openssh_file(&boom::sign::public_key_path(&key))
        .unwrap();
    run(
        dir,
        &["init", "--person", "ada", "--key", key.to_str().unwrap()],
    );
    let mut store = Store::open(&store_of(dir)).unwrap();
    let mut lives: Vec<Life> = (0..CLAIMS)
        .map(|_| Life {
            standing: None,
            versions: 0,
            waiting: None,
        })
        .collect();
    let mut made = 1;
    let mut step = 0usize;
    let started = Instant::now();
    while made < entries {
        let mut batch: Vec<(String, Option<String>)> = Vec::new();
        while batch.len() < BATCH && made + batch.len() < entries {
            let claim = step % CLAIMS;
            let round = step / CLAIMS;
            step += 1;
            let life = &mut lives[claim];
            match life.waiting.take() {
                None => {
                    life.versions += 1;
                    let proposal = proposal(claim, life.versions, life.standing.clone());
                    let version = proposal.version.clone();
                    let entry = Entry {
                        at: 1_763_424_600,
                        actor: Actor::Agent {
                            provenance: format!("run-{round}"),
                        },
                        act: Act::Proposed {
                            claim: proposal,
                            imported: None,
                            attach: Some(Attach {
                                repositories: vec![format!("repo-{}", claim % REPOSITORIES)],
                            }),
                        },
                    };
                    batch.push((entry.signed_part(), None));
                    life.waiting = Some(version);
                }
                Some(version) => {
                    let act = match life.versions % 4 {
                        0 | 1 => Act::Accepted {
                            name: format!("claim-{claim}"),
                            version: version.clone(),
                        },
                        2 => Act::Rejected {
                            name: format!("claim-{claim}"),
                            version: version.clone(),
                            reason: "Not what the project means.".to_owned(),
                        },
                        _ => {
                            // Left waiting: the next version amends the
                            // standing one beside it.
                            continue;
                        }
                    };
                    if matches!(act, Act::Accepted { .. }) {
                        life.standing = Some(version);
                    }
                    let signed = Entry {
                        at: 1_763_424_700,
                        actor: Actor::Person("ada".to_owned()),
                        act,
                    }
                    .signed_part();
                    let signature = soft.sign(signed.as_bytes());
                    batch.push((signed, Some(signature)));
                }
            }
        }
        let appends: Vec<Append<'_>> = batch
            .iter()
            .map(|(signed, signature)| Append {
                signed,
                signature: signature.as_deref(),
                blob: None,
            })
            .collect();
        let (written, _) = store.append_all(&appends).unwrap();
        made += written.len();
        eprintln!("{made} entries, {:.1} s", started.elapsed().as_secs_f64());
    }
}

/// Appends `count` agents' proposals to the log as another program
/// does, leaving the views behind.
fn behind(dir: &Path, count: usize) {
    let conn = rusqlite::Connection::open(store_of(dir)).unwrap();
    let (mut seq, mut prev): (i64, String) = conn
        .query_row(
            "SELECT seq, hash FROM log ORDER BY seq DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    for at in 0..count {
        let signed = Entry {
            at: 1_763_424_800,
            actor: Actor::Agent {
                provenance: "another-program".to_owned(),
            },
            act: Act::Proposed {
                claim: Proposal::new(
                    Kind::Invariant,
                    &format!("behind-{seq}-{at}"),
                    "A claim another program proposed.",
                    Vec::new(),
                    None,
                    None,
                    None,
                ),
                imported: None,
                attach: None,
            },
        }
        .signed_part();
        seq += 1;
        let hash = entry::hash(&prev, &signed, None);
        conn.execute(
            "INSERT INTO log (seq, prev, hash, signed, signature) VALUES (?1, ?2, ?3, ?4, NULL)",
            rusqlite::params![seq, prev, hash, signed],
        )
        .unwrap();
        prev = hash;
    }
}

/// Times one agent's proposal and its accept by ada, as the command
/// line runs them.
fn propose_accept(dir: &Path) {
    let name = format!("timed-{}", std::process::id());
    let started = Instant::now();
    let proposed = run(
        dir,
        &[
            "propose",
            "invariant",
            &name,
            "A timed claim holds.",
            "--repository",
            "repo-3",
            "--provenance",
            "timing",
            "--json",
        ],
    );
    let propose = started.elapsed();
    let version = serde_json::from_str::<serde_json::Value>(&proposed.stdout).unwrap()["version"]
        .as_str()
        .unwrap()
        .to_owned();
    let key = dir.join("ada");
    let started = Instant::now();
    run(
        dir,
        &[
            "accept",
            &format!("{name}@{version}"),
            "--key",
            key.to_str().unwrap(),
        ],
    );
    let accept = started.elapsed();
    println!(
        "propose {:.0} ms, accept {:.0} ms",
        propose.as_secs_f64() * 1000.0,
        accept.as_secs_f64() * 1000.0
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = Path::new(&args[2]);
    match args[1].as_str() {
        "generate" => generate(dir, args[3].parse().unwrap()),
        "behind" => behind(dir, args[3].parse().unwrap()),
        "propose-accept" => propose_accept(dir),
        other => panic!("no command {other}"),
    }
}
