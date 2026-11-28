//! The program that signs, in the place of ssh-keygen: it gets the
//! passphrase program of the settings as ssh-keygen does (persons-acts).
//! The other tests of signatures run in the crate, in
//! src/tests/signatures.rs.

#[path = "../src/tests/support.rs"]
mod support;

use boom::sign::{Key, Keys, SshKeygen};
use boom::Failure;
use support::SoftKeys;

#[test]
fn the_passphrase_program_of_the_settings_reaches_ssh_keygen() {
    use std::os::unix::fs::PermissionsExt as _;

    let dir = tempfile::tempdir().unwrap();
    let keys = SoftKeys::new(dir.path());
    let ada = keys.security_key("ada", 1);
    // A program in the place of ssh-keygen: it asks the passphrase
    // program as ssh-keygen does, writes what it got, and signs nothing.
    let program = dir.path().join("ssh-keygen");
    let seen = dir.path().join("seen");
    std::fs::write(
        &program,
        format!(
            "#!/bin/sh\n{{ echo \"require=$SSH_ASKPASS_REQUIRE\"; echo \"passphrase=$(\"$SSH_ASKPASS\" 'Enter passphrase for key')\"; }} > '{}'\ncat > /dev/null\nexit 1\n",
            seen.display()
        ),
    )
    .unwrap();
    let askpass = dir.path().join("askpass");
    std::fs::write(&askpass, "#!/bin/sh\necho the-tests-passphrase\n").unwrap();
    for file in [&program, &askpass] {
        std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let keygen = SshKeygen { program };
    let key = Key {
        path: ada.clone(),
        askpass: Some(askpass),
    };
    match keygen.signer(&key).unwrap().sign(ENTRY) {
        Err(Failure::NotRun(why)) => {
            assert!(why.starts_with("the key's program did not sign"), "{why}")
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        std::fs::read_to_string(&seen).unwrap(),
        "require=force\npassphrase=the-tests-passphrase\n"
    );
}

const ENTRY: &[u8] = br#"{"format":"boom-entry/1","act":"accepted"}"#;
