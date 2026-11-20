---
number: 1
title: boom is an open-source repository of its own, and its binary is boom
status: accepted
date: 2026-11-01
decision-makers:
- Chuck Swanberg
---

# boom is an open-source repository of its own, and its binary is boom

Status: accepted by the user on 2026-11-01.

Deciders: the user, on 2026-10-31 and 2026-11-01.
Sources: the scaffolding of flume and sluice, which this repository copies.

## Context and problem statement

A project has invariants, decisions, assumptions, decision records,
rules for a scope, a design book and a context map. These are one thing,
seen in several ways. That thing is a ledger of claims, and of the
events that propose, accept, reject or supersede the claims. Each other
form is a view of the ledger. Where does the tool that keeps that ledger
live? Is the tool open? What is the name of its binary?

## Considered options

1. A part of another tool: a command of the binary of that tool.
2. A repository of its own, agentplot/boom, open source under MIT, in Rust. Its binary is `boom`.

## Decision outcome

Option 2. The format of the ledger must be open, so that a project
trusts that it owns its history. A tool of its own keeps that format
free of any one caller. The tool is in Rust, so that it shares the
crates of the family. `boom` is a working name.

This repository records each later decision here, in one MADR record.
The rules in `docs/architecture/core.md` name these records as sources.

### Consequences

- The repository carries the MIT license.
- boom reads status from events and computes each view (core.1, core.2).
- The repository holds nothing commercial (core.4).
