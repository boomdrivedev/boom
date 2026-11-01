# Tasks

Every new file is written from design.md alone and holds nothing
commercial (core.4). Clause numbers the site cites are the ones the
statement holds once group 3 is done.

## 1. The decision records

- [ ] 1.1 Write `docs/adr/0002-the-ledger-holds-claims-narratives-contexts-relationships-and-terms.md` in record 0001's form (design D1) with the substance of "0002" in design.md's section The records. Verify: `devenv shell -- moon run root:adrs` passes, counting 2 records.
- [ ] 1.2 Write `docs/adr/0003-a-status-is-read-from-seven-events-and-a-claim-changes-by-amendment-or-supersession.md` (D1, "0003"). Verify: `devenv shell -- moon run root:adrs` passes, counting 3 records.
- [ ] 1.3 Write `docs/adr/0004-a-verdict-is-a-record-beside-the-claims-and-a-cells-status-and-its-two-queues-are-computed.md` (D1, "0004"). Verify: `devenv shell -- moon run root:adrs` passes, counting 4 records.
- [ ] 1.4 Write `docs/adr/0005-boom-reconciles-and-judges-each-repository-at-a-commit-on-a-host-with-no-other-service.md` (D1, "0005"). Verify: `devenv shell -- moon run root:adrs` passes, counting 5 records.
- [ ] 1.5 Write `docs/adr/0006-boom-check-reads-a-change-in-three-layers-and-the-gate-blocks-regressions-never-the-backlog.md` (D1, "0006"). Verify: `devenv shell -- moon run root:adrs` passes, counting 6 records.
- [ ] 1.6 Write `docs/adr/0007-a-claims-scope-is-derived-and-each-repository-is-one-context-until-someone-models-the-system.md` (D1, "0007"). Verify: `devenv shell -- moon run root:adrs` passes, counting 7 records.
- [ ] 1.7 Write `docs/adr/0008-a-person-whose-role-allows-it-accepts-never-an-agent-and-never-the-records-author.md` (D1, "0008"), its consequences naming the person working alone and `boom mcp` offering no accept. Verify: `devenv shell -- moon run root:adrs` passes, counting 8 records.
- [ ] 1.8 Write `docs/adr/0009-millpond-and-boom-work-both-ways-and-millpond-takes-in-a-backlog-only-where-no-other-planner-plans-it.md` (D1, "0009"), with (a), (b) and (c) and the ruling's request id among its sources. Verify: `devenv shell -- moon run root:adrs` passes, counting 9 records.
- [ ] 1.9 Write `docs/adr/0010-a-proposal-awaiting-a-person-shows-as-a-millpond-signal-where-no-other-planner-is-connected.md` (D1, "0010"). Verify: `devenv shell -- moon run root:adrs` passes, counting 10 records, and `devenv shell -- moon run root:format` passes.

## 2. The statement's check

- [ ] 2.1 Write `check-statement.sh` (design D4): POSIX sh and awk, the statement's path its one argument with `design/requirements.md` the default, failing with the number and each line where two clauses, two invariants or two scenarios share a number, or where a citation names a clause the file lacks or a record `docs/adr` lacks, and printing the counts otherwise. Verify: `devenv shell -- moon run root:format` passes, shellcheck and shfmt finding nothing.
- [ ] 2.2 Write the fixtures `tests/statement/holds.md` (Parts A to C with clauses 1, 2, 3, 5 and 3a, a citation `(3,` broken across a line before `5)`, `(record 0001)`, invariants I1 and I2, scenarios S1 and S2, and a numbered list `1.` in its section 12), `shared-number.md` (two clauses `4.` and two invariants `**I2.**`) and `unresolved.md` (a citation `(9)` with no clause 9 and `(record 0099)`), and `check-statement.test.sh`, which runs the check over each and fails unless `holds.md` passes, `shared-number.md` fails naming 4 and I2, and `unresolved.md` fails naming 9 and record 0099. Verify: `devenv shell -- moon run root:format` passes; 2.3 runs the cases.
- [ ] 2.3 Add `test` to the root's `moon.yml` (D4), `check-fast`, run from the workspace root, with the inputs D4 names and a comment in the file's manner saying what it holds. Verify: `devenv shell -- moon run root:test` passes.

## 3. The statement

- [ ] 3.1 Write `design/requirements.md`'s title, preamble with the mechanism rule, `## 1. Purpose`, `## 2. Actors` and `## 3. Vocabulary` (design D3 and the section The statement), in the family's requirements style, and add `statement` to the root's `moon.yml` (D4), `check-fast`, with its inputs and a comment saying what it holds. Verify: `devenv shell -- moon run root:statement` passes, reading no clause yet.
- [ ] 3.2 Write Part A, sections A.1 to A.10, one clause per invariant the section The statement lists, numbered from 1 in reading order, each citing the record behind it and earlier clauses by number, and naming no command, file, format, field, service or tool (D3). Verify: `devenv shell -- moon run root:statement` passes, counting Part A's clauses.
- [ ] 3.3 Write Part B, sections B.1 to B.9, each seam's operations in abstract names and its guarantee as clauses, numbered on from Part A and naming no tool. Verify: `devenv shell -- moon run root:statement` passes.
- [ ] 3.4 Write Part C, sections C.1 to C.9, opening "Parts A and B do not change to admit a profile.", each profile mapping its seam's operations to how it satisfies them, citing Part B's clauses, C.4 sending its binding to section 10. Verify: `devenv shell -- moon run root:statement` passes.
- [ ] 3.5 Write `## 7. Global invariants` (I1 onward, each citing its clauses), `## 8. Non-goals`, `## 9. Environment givens` (constraints, then the stack's displaceable defaults), `## 10. Questions the model must answer` (marked as not requirements), `## 11. Scenarios the model must satisfy` (S1 onward, each citing the clauses it exercises) and `## 12. What to deliver`. Verify: `devenv shell -- moon run root:statement` passes, every clause and record cited by an invariant or a scenario existing.
- [ ] 3.6 Add `design/requirements.md`, the statement, to the table in `README.md`, and make its line on acceptance say that an agent proposes and a person whose role allows it accepts, never their own proposal (record 0008). Verify: `devenv shell -- prose` passes over `README.md`.

## 4. The design pages

- [ ] 4.1 Write `site/scripts/design.ts` (design D5): it empties and writes `site/src/content/docs/design/`, the statement's page with each clause, invariant and scenario an anchored paragraph showing its own number and each citation linked, and one page per record titled, described, ordered and edit-linked from its frontmatter, failing with the file's name on a record that lacks its number, title, status or date. Add `/site/src/content/docs/design/` to `.gitignore`. In `site/moon.yml`, add `design-pages` (no layer, `cache: false`, inputs `scripts/design.ts`, `/design/requirements.md`, `/docs/adr/*.md`), make `typecheck` and `build` depend on it and take `/design/requirements.md` and `/docs/adr/*.md` as inputs, and make `prose` leave out `design/` by glob; in `package.json`, make `dev` run the script first. Verify: `devenv shell -- moon run site:format site:lint site:typecheck site:build site:prose` passes, the build writing `dist/design/statement/index.html` and `dist/design/decisions/0010/index.html`.
- [ ] 4.2 In `site/astro.config.mjs`, add the sidebar group "Design" after "boom check", holding "The statement", by `slug: "design/statement"`, which Starlight refuses to build when no page has it, and "Decisions", generated from `design/decisions`. Verify: `devenv shell -- moon run site:build` passes.

## 5. The millpond diagram

- [ ] 5.1 Write `site/src/diagrams/millpond.ts` (design D10), wide and narrow, and register it in `site/src/diagrams/index.ts`; change `fits.ts`'s millpond lines to "boom challenge, boom propose" and "boom backlog --json"; name the new diagram in `site/README.md`'s paragraph on the diagrams. Verify: `devenv shell -- moon run site:test site:format site:lint` passes.

## 6. The pages, in line with the records

- [ ] 6.1 `how-its-used.mdx` as design.md's section The site says, each statement about the design linking its clauses and records (D6). Verify: `devenv shell -- moon run site:build site:prose` passes.
- [ ] 6.2 `configuration.md`: `planner`, a `[roles]` sketch, `challenged` and `due` events, the verdict's id and judge. Verify: `devenv shell -- moon run site:build site:prose` passes.
- [ ] 6.3 `commands.md`: `boom judge`, `boom reconcile`, `boom backlog` (with `--planner` and each entry's key, D9), `boom challenge` and `boom annotate` in the table with their own sections; `boom query --planner`; `boom accept`'s rule and its refusal; `boom mcp` with no accept and no reject; "Also in the design" removed. Verify: `devenv shell -- moon run site:build site:prose` passes.
- [ ] 6.4 `check.mdx`: each section cites record 0006 and its clause. Verify: `devenv shell -- moon run site:build site:prose` passes.
- [ ] 6.5 `integrations.mdx`: millpond both ways, (a), (b) with the `millpond` diagram and a caption naming each station, and (c), with proposals as signals where millpond is the planner; `boom mcp`'s note on accept citing record 0008; sluice and flume citing their profiles. Verify: `devenv shell -- moon run site:build site:prose` passes, the page including the `millpond` diagram.
- [ ] 6.6 `roadmap.md`: "Settled" lists records 0001 to 0010, each linked to its page with the clauses that state it, and links the global invariants from `#I1` and the scenarios from `#S1`; "Leaning" and "Proposed" keep what no record decided; "Open questions" are the statement's section 10, each linked, with the build order kept. Verify: `devenv shell -- moon run site:build site:prose` passes.
- [ ] 6.7 `index.mdx`: "Where it stands" names the statement and records 0001 to 0010, linked; the cards and "Who it is for" cite their records. Verify: `devenv shell -- moon run site:build site:prose` passes.
- [ ] 6.8 The walkthroughs (design D7), none showing a person accepting a version they proposed: `first-claims.md` with the agent proposing, Ada accepting, and her own accept refused once; `an-agent-proposes.md` with the rule in place of the open question; `a-change-breaks-a-claim.md` citing records 0006 and 0008; `importing-adrs.md` with the agent proposing the new decision and its questions linked to the statement. Verify: `devenv shell -- moon run site:build site:prose` passes.

## 7. The checks of the projects touched

- [ ] 7.1 With the work committed, run the checks of the root and the site. Verify: `devenv shell -- moon run root:format root:adrs root:statement root:test root:changes site:format site:lint site:typecheck site:test site:prose site:build` passes every task.

## Workflow follow-up

- sluice runs the whole gate, `#check-fast` on `main`, when the change lands.
- `docs/architecture/core.md` is drawn from records 0002 to 0010 in a separate step, as the user decided on 2026-11-01.
