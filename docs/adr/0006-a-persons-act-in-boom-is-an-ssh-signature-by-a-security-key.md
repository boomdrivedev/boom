---
number: 6
title: A person's act in boom is an SSH signature by a security key, the person present
status: accepted
date: 2026-11-18
decision-makers:
- the change boom-keeps-decisions-in-a-database-on-the-machine's construct stage
---

# A person's act in boom is an SSH signature by a security key, the person present

Status: accepted. The construct stage made this decision. It settles
`open-signature-format` of the interview, on the requirement that the
user gave, `persons-acts-signed-with-presence`. Record 0005 keeps that
requirement. The decision is cheap to change until somebody keeps a
store: no log exists yet, and every entry names its format,
`boom-entry/1`.

Deciders: the construct stage of the change
`boom-keeps-decisions-in-a-database-on-the-machine`. Its brief asks
that "an agent proposes and only a person accepts or rejects".

Sources:

- `interview.md`, the answers of the user of 2026-11-01:
  `persons-acts-signed-with-presence` ("signed, with the person
  present"), `self-registered-first-owner` and `open-signature-format`.
- Record 0005.
- `PROTOCOL.sshsig` and `PROTOCOL.u2f` of OpenSSH, and `ssh-keygen(1)`
  on `-Y sign`, `-Y verify` and `no-touch-required` of the allowed
  signers.
- boomdrivedev/blueprints: the statement of timber, 11, 12 and 142, the
  signed run records of sluice, and 114 of weir. There, each tool signs
  in its own identity, as an in-toto statement in a DSSE envelope.

## Context and problem statement

An agent on the machine of a person runs as the same user as the
person. Thus boom cannot tell the accept of a person from the accept of
an agent by the user that ran it. On 2026-11-01 the user decided these
points:

- An act of a person holds the signature of that person, by a key that
  the ledger registers to that person.
- The key signs only when the person is present: the touch of a
  hardware key, a passkey or a biometric prompt.
- The tools of the format verify the signature, with no boom.

The interview did not decide the format: the attestation format of the
family, the SSH signatures of git, or another format.

## Considered options

1. An SSH signature (`ssh-keygen -Y sign`, namespace `boom`) by a
   security key, `sk-ssh-ed25519@openssh.com` or
   `sk-ecdsa-sha2-nistp256@openssh.com`, with its user-presence flag
   required (chosen).
2. The attestation format of the family, an in-toto statement in a
   DSSE envelope.
3. A key behind an SSH agent that asks for a biometric prompt before
   each signature.

## Decision outcome

1. **An SSH signature by a security key.** The act of a person has a
   signature in the signature format of OpenSSH, namespace `boom`. A
   security key that the store registers to that person makes the
   signature. The signature of a security key holds the user-presence
   flag of the authenticator. Thus the presence that the user asked for
   is in the signature, and boom refuses a signature without the flag.
   `ssh-keygen -Y verify` verifies the same signature with no boom, and it
   requires the flag by default. Not taken:
   - In-toto in DSSE (option 2). Its envelope holds no proof of
     presence for a key.
   - The assertion of a passkey. It needs the WebAuthn of a browser,
     and it has no command line.
   - A biometric prompt in an SSH agent (option 3). It leaves no trace
     in the signature, so a plain key file makes the same signature.
2. **boom registers only a security key.** boom registers a key of the
   two security key types. boom refuses a key of any other type and
   names its type.
3. **The ssh-keygen of the person signs.** boom gives the signed part
   of the entry to `ssh-keygen -Y sign -n boom -f <key>` on the machine
   of the person. There the touch of the key is the prompt of the key,
   not a prompt of boom (`no-prompts-repeats-add-nothing`). boom
   verifies the signature in its own code, so no read needs OpenSSH.
4. **The signature covers the act, not its place in the log.** The
   signed part does not include the number and the chain of the entry.
   Thus an entry that the drive puts in order later (record 0005, point
   4) keeps its signature.

## Consequences

- Good: a reader verifies the act of a person with OpenSSH alone. An
  agent cannot make an act of a person without the touch of the person.
- Good: the act of a person is one more entry in the log, with no
  service.
- Bad: a person needs a FIDO2 security key, and an OpenSSH with support
  for security keys. The `ssh-keygen` of macOS does not have this
  support.
- Bad: boom cannot prove that a registered key is in hardware. A person
  who registers a security key made in software signs without presence.
  boom proves that the registered key of a person signed the acts of
  that person, as `self-registered-first-owner` states, and nothing
  more.
- Neutral: later, the attestation of the family can hold the
  signature, as timber holds the claim of another party whole (timber
  12). This does not change what the person signs.
