//! The keys seam and a person's signature (design 7, 8; record 0006).

use super::support;

use std::ffi::OsString;
use std::path::Path;

use boom::sign::{verify, Key, Keys, SshKeygen};
use boom::Failure;
use support::{SoftKeys, ABSENT};

const ENTRY: &[u8] = br#"{"format":"boom-entry/1","act":"accepted"}"#;

#[test]
fn a_security_keys_signature_verifies() {
    let dir = tempfile::tempdir().unwrap();
    let keys = SoftKeys::new(dir.path());
    let ada = keys.security_key("ada", 1);
    let signer = keys.signer(&Key::at(&ada)).unwrap();
    let signature = signer.sign(ENTRY).unwrap();
    assert!(signature.starts_with("-----BEGIN SSH SIGNATURE-----"));
    assert_eq!(verify(signer.public_key(), ENTRY, &signature), Ok(()));
    assert!(verify(signer.public_key(), b"another entry", &signature).is_err());
    let bo = keys.security_key("bo", 2);
    assert_eq!(
        verify(&keys.public_key(&bo), ENTRY, &signature),
        Err("its signature is by another key".to_owned())
    );
}

#[test]
fn a_signature_without_presence_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let keys = SoftKeys::new(dir.path());
    let ada = keys.security_key("ada", 1);
    keys.set_flags(&ada, ABSENT);
    let signature = keys.signer(&Key::at(&ada)).unwrap().sign(ENTRY).unwrap();
    let refused = verify(&keys.public_key(&ada), ENTRY, &signature).unwrap_err();
    assert!(
        refused.starts_with("the signature does not show its person present"),
        "{refused}"
    );
}

#[test]
fn a_plain_keys_signature_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let keys = SoftKeys::new(dir.path());
    let plain = keys.plain_key("ada", 1);
    let signature = keys.signer(&Key::at(&plain)).unwrap().sign(ENTRY).unwrap();
    let refused = verify(&keys.public_key(&plain), ENTRY, &signature).unwrap_err();
    assert!(refused.contains("a key of type ssh-ed25519"), "{refused}");
    assert!(
        refused.contains("sk-ssh-ed25519@openssh.com")
            && refused.contains("sk-ecdsa-sha2-nistp256@openssh.com"),
        "{refused}"
    );
}

#[test]
fn a_signature_in_another_namespace_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let keys = SoftKeys::new(dir.path());
    let ada = keys.security_key("ada", 1);
    let signature = keys.sign_with(&ada, "git", ENTRY);
    let refused = verify(&keys.public_key(&ada), ENTRY, &signature).unwrap_err();
    assert_eq!(refused, "its signature is in the namespace git, not boom");
}

#[test]
fn ssh_keygen_is_asked_to_sign_in_the_namespace_boom() {
    let keygen = SshKeygen::default();
    assert_eq!(keygen.program, Path::new("ssh-keygen"));
    let arguments: Vec<OsString> = ["-Y", "sign", "-n", "boom", "-f", "/keys/ada"]
        .into_iter()
        .map(OsString::from)
        .collect();
    assert_eq!(keygen.arguments(Path::new("/keys/ada")), arguments);
}

#[test]
fn a_key_whose_program_cannot_be_run_does_not_sign() {
    let dir = tempfile::tempdir().unwrap();
    let keys = SoftKeys::new(dir.path());
    let ada = keys.security_key("ada", 1);
    let keygen = SshKeygen {
        program: dir.path().join("no-such-program"),
    };
    let signer = keygen.signer(&Key::at(&ada)).unwrap();
    assert_eq!(signer.public_key(), &keys.public_key(&ada));
    match signer.sign(ENTRY) {
        Err(Failure::NotRun(why)) => {
            assert!(why.starts_with("the key's program did not sign"), "{why}")
        }
        other => panic!("{other:?}"),
    }
}
