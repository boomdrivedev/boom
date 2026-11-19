//! A person's signature: the seam that signs and the check that
//! verifies.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use ssh_key::{Algorithm, HashAlg, PublicKey, SshSig};

use crate::Failure;

/// The two key types boom registers: security keys, whose signatures
/// carry the authenticator's user-presence flag (record 0006).
pub const TYPES: [&str; 2] = [
    "sk-ssh-ed25519@openssh.com",
    "sk-ecdsa-sha2-nistp256@openssh.com",
];

/// The user-presence bit of a security key signature's flags.
const USER_PRESENT: u8 = 0x01;

/// Refuses a key of any type but the two security key types, naming its
/// type and the two boom registers.
pub fn check_key_type(key: &PublicKey) -> Result<(), String> {
    match key.algorithm() {
        Algorithm::SkEd25519 | Algorithm::SkEcdsaSha2NistP256 => Ok(()),
        other => Err(format!(
            "a key of type {} signs without showing its person present; boom registers {} and {}",
            other.as_str(),
            TYPES[0],
            TYPES[1]
        )),
    }
}

/// The public key an armored SSH signature names as its signer, where
/// it is one.
pub fn signing_key(signature: &str) -> Option<PublicKey> {
    let signature = SshSig::from_pem(signature.trim_end().as_bytes()).ok()?;
    Some(PublicKey::new(signature.public_key().clone(), ""))
}

/// Verifies `signature`, an armored SSH signature, over `signed` by
/// `public_key`: one of the two security key types, in the namespace
/// `boom`, hashed with SHA-512, and its user-presence flag set (design
/// 7). Answers why where it does not.
pub fn verify(public_key: &PublicKey, signed: &[u8], signature: &str) -> Result<(), String> {
    check_key_type(public_key)?;
    let signature = SshSig::from_pem(signature.trim_end().as_bytes())
        .map_err(|error| format!("its signature is not an SSH signature: {error}"))?;
    if signature.namespace() != NAMESPACE {
        return Err(format!(
            "its signature is in the namespace {}, not {NAMESPACE}",
            signature.namespace()
        ));
    }
    if signature.hash_alg() != HashAlg::Sha512 {
        return Err(format!(
            "its signature hashes with {}, not sha512",
            signature.hash_alg().as_str()
        ));
    }
    if signature.public_key() != public_key.key_data() {
        return Err("its signature is by another key".to_owned());
    }
    public_key
        .verify(NAMESPACE, signed, &signature)
        .map_err(|error| format!("its signature does not verify: {error}"))?;
    let bytes = signature.signature().as_bytes();
    let flags = bytes
        .len()
        .checked_sub(5)
        .map(|at| bytes[at])
        .ok_or("its signature carries no security key flags")?;
    if flags & USER_PRESENT == 0 {
        return Err(
            "the signature does not show its person present: its user-presence flag is unset"
                .to_owned(),
        );
    }
    Ok(())
}

/// Something that signs with one key: a person's security key, through
/// their `ssh-keygen`, or a test's key made in software.
pub trait Signer {
    /// The key's public half.
    fn public_key(&self) -> &PublicKey;

    /// Signs `message` in the namespace `boom`, answering the armored
    /// SSH signature, or not run where the key's program did not sign.
    fn sign(&self, message: &[u8]) -> Result<String, Failure>;
}

/// Turns the path a command's `--key` names into a [`Signer`].
pub trait Keys {
    /// The signer for the key at `key`, refused where its public key
    /// cannot be read.
    fn signer(&self, key: &Path) -> Result<Box<dyn Signer + '_>, Failure>;
}

/// The person's own `ssh-keygen`, signing with the key a path names
/// and reading its public key from the file beside it, `<key>.pub`.
#[derive(Debug, Clone)]
pub struct SshKeygen {
    /// The program run to sign, `ssh-keygen` unless a test names
    /// another.
    pub program: PathBuf,
}

impl Default for SshKeygen {
    fn default() -> Self {
        SshKeygen {
            program: PathBuf::from("ssh-keygen"),
        }
    }
}

impl SshKeygen {
    /// The arguments it runs its program with to sign with `key`: the
    /// signed part on standard input, the signature on standard output.
    pub fn arguments(&self, key: &Path) -> Vec<OsString> {
        let mut arguments: Vec<OsString> = ["-Y", "sign", "-n", NAMESPACE, "-f"]
            .into_iter()
            .map(OsString::from)
            .collect();
        arguments.push(key.as_os_str().to_owned());
        arguments
    }
}

/// The namespace every person's signature in boom is made in.
pub const NAMESPACE: &str = "boom";

/// The file a key's public half is read from: `<key>.pub`.
pub fn public_key_path(key: &Path) -> PathBuf {
    let mut path = key.as_os_str().to_owned();
    path.push(".pub");
    PathBuf::from(path)
}

/// Reads an OpenSSH public key file, refused where it cannot be read.
pub fn read_public_key(path: &Path) -> Result<PublicKey, Failure> {
    let text = std::fs::read_to_string(path).map_err(|error| {
        Failure::refused(format!(
            "the public key {} cannot be read: {error}",
            path.display()
        ))
    })?;
    PublicKey::from_openssh(text.trim()).map_err(|error| {
        Failure::refused(format!(
            "{} is not an OpenSSH public key: {error}",
            path.display()
        ))
    })
}

impl Keys for SshKeygen {
    fn signer(&self, key: &Path) -> Result<Box<dyn Signer + '_>, Failure> {
        let public_key = read_public_key(&public_key_path(key))?;
        Ok(Box::new(SshKeygenSigner {
            program: self,
            key: key.to_owned(),
            public_key,
        }))
    }
}

struct SshKeygenSigner<'a> {
    program: &'a SshKeygen,
    key: PathBuf,
    public_key: PublicKey,
}

impl Signer for SshKeygenSigner<'_> {
    fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    fn sign(&self, message: &[u8]) -> Result<String, Failure> {
        use std::io::{Read, Write};
        use std::process::{Command, Stdio};

        let did_not_sign = |words: &str| {
            Failure::not_run(format!(
                "the key's program did not sign: {} {}",
                self.program.program.display(),
                words.trim()
            ))
        };
        let mut child = Command::new(&self.program.program)
            .args(self.program.arguments(&self.key))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| did_not_sign(&format!("could not be run: {error}")))?;
        // The key's own prompt, a touch asked for, reaches the person as
        // the program writes it, and its words are kept for a failure.
        let mut stderr = child.stderr.take().expect("standard error is piped");
        let echo = std::thread::spawn(move || {
            let mut words = Vec::new();
            let mut buffer = [0u8; 1024];
            while let Ok(read) = stderr.read(&mut buffer) {
                if read == 0 {
                    break;
                }
                let _ = std::io::stderr().write_all(&buffer[..read]);
                words.extend_from_slice(&buffer[..read]);
            }
            String::from_utf8_lossy(&words).into_owned()
        });
        let written = child
            .stdin
            .take()
            .expect("standard input is piped")
            .write_all(message);
        let output = child.wait_with_output();
        let words = echo.join().unwrap_or_default();
        let output = output.map_err(|error| did_not_sign(&format!("did not finish: {error}")))?;
        if let Err(error) = written {
            return Err(did_not_sign(&format!(
                "did not read the entry: {error} {words}"
            )));
        }
        if !output.status.success() {
            return Err(did_not_sign(&format!(
                "exited with {}: {words}",
                output.status
            )));
        }
        String::from_utf8(output.stdout)
            .map_err(|_| did_not_sign("wrote a signature that is not text"))
    }
}

/// A public key as one OpenSSH line, its type and its key, with no
/// comment.
pub fn key_line(key: &PublicKey) -> String {
    PublicKey::new(key.key_data().clone(), "")
        .to_openssh()
        .expect("a public key writes as an OpenSSH line")
}
