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
