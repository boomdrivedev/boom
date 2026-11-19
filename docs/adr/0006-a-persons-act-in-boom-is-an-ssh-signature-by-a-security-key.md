---
number: 6
title: A person's act in boom is an SSH signature by a security key, the person present
status: accepted
date: 2026-11-18
decision-makers:
- the change boom-keeps-decisions-in-a-database-on-the-machine's construct stage
---

# A person's act in boom is an SSH signature by a security key, the person present

Status: accepted. The construct stage's own, settling the interview's
`open-signature-format` on the requirement the user gave,
`persons-acts-signed-with-presence`, which record 0005 keeps. Cheap to
change until a store is kept anywhere: no log exists yet, and every
entry names its format, `boom-entry/1`.

Deciders: the construct stage of the change
`boom-keeps-decisions-in-a-database-on-the-machine`, whose brief asks
that "an agent proposes and only a person accepts or rejects".

Sources: `interview.md`, the user's answers of 2026-11-01:
`persons-acts-signed-with-presence` ("signed, with the person present"),
`self-registered-first-owner` and `open-signature-format`; record 0005;
OpenSSH's `PROTOCOL.sshsig` and `PROTOCOL.u2f`, and `ssh-keygen(1)` on
`-Y sign`, `-Y verify` and the allowed signers' `no-touch-required`;
boomdrivedev/blueprints, timber's statement 11, 12 and 142, sluice's
signed run records and weir's 114, where each tool signs in its own
identity as an in-toto statement in a DSSE envelope.

## Context and problem statement

An agent on a person's machine runs as the same user as the person, so
boom cannot tell a person's accept from an agent's by who ran it. The
user settled on 2026-11-01 that a person's act carries their signature
by a key the ledger registers to them, a key that signs only with the
person present, a hardware key's touch, a passkey or a biometric
prompt, verifiable with the format's own tools and no boom. The
interview left the format open: the family's attestation format, git's
SSH signatures, or another.

## Considered options

1. An SSH signature (`ssh-keygen -Y sign`, namespace `boom`) by a
   security key, `sk-ssh-ed25519@openssh.com` or
   `sk-ecdsa-sha2-nistp256@openssh.com`, its user-presence flag
   required (chosen).
2. The family's attestation format, an in-toto statement in a DSSE
   envelope.
3. A key behind an SSH agent that asks for a biometric prompt before
   each signature.

## Decision outcome

1. **An SSH signature by a security key.** A person's act is signed in
   OpenSSH's signature format, namespace `boom`, by a security key the
   store registers to them. A security key's signature carries the
   authenticator's user-presence flag, so the presence the user asked
   for is in the signature itself, and boom refuses one without it.
   `ssh-keygen -Y verify` checks the same signature, and requires the
   flag by default, with no boom. Not taken: in-toto in DSSE (option
   2), whose envelope carries no proof of presence for a key, and a
   passkey's assertion, which needs a browser's WebAuthn and no command
   line; a biometric prompt in an SSH agent (option 3), which leaves no
   trace in the signature, so a plain key file would sign the same.
2. **Only a security key is registered.** boom registers a key of the
   two security key types and refuses any other, naming its type.
3. **The person's own ssh-keygen signs.** boom hands the entry's signed
   part to `ssh-keygen -Y sign -n boom -f <key>` on the person's
   machine, where the key's touch is the key's prompt and not boom's
   (`no-prompts-repeats-add-nothing`), and verifies in its own code, so
   no read needs OpenSSH.
4. **The signature covers the act, not its place in the log.** The
   signed part leaves out the entry's number and chain, so an entry the
   drive orders later (record 0005, point 4) keeps its signature.

## Consequences

- Good: a reader verifies a person's act with OpenSSH alone, and an
  agent cannot make one without the person's touch.
- Good: a person's act is one more entry in the log, with no service.
- Bad: a person needs a FIDO2 security key, and an OpenSSH built with
  security key support; macOS's own `ssh-keygen` lacks it.
- Bad: boom cannot prove a registered key is in hardware; a person who
  registers a security key made in software signs without presence.
  boom proves a person's acts were signed by their registered key, as
  `self-registered-first-owner` has it, and nothing more.
- Neutral: the signature can later be carried inside the family's
  attestation, as timber carries another party's claim whole (timber
  12), without changing what is signed.
