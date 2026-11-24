No task adds a part of the repository. The change is Rust code of the
crate `boom`, so it adds no moon project and no dependency. Each task
names the moon task that proves it, run as `devenv shell -- moon run
<project>:<task>`. Each scenario of the delta specs is one test, named
by its scenario, in the test file that its task names.

## 1. The kind `outline`

- [x] 1.1 Add `Kind::Outline` to `claim.rs`, with `view` and the edge `narratives` in `Proposal`, each written only where there is one, and both in the content of a version. `boom propose outline` requires `--view`, refuses each view but `book`, and refuses an outline with no `--narrative`. Tests in the new file `crates/boom/tests/views.rs` for "An outline proposed", "No view", "A view that boom does not compute" and "No narrative", and in `crates/boom/tests/claims.rs` for "An outline with its title". Proved by `boom:test`
- [x] 1.2 Resolve each `--narrative` as `--source` is resolved, and keep the order given. `judge.rs` refuses a name that does not resolve, a claim that is not a narrative, a narrative given two times, and `--narrative` on a claim that is not an outline. Tests in `views.rs` for "A name stores its accepted version", "A name that does not resolve", "A claim that is not a narrative", "A narrative given two times", "Narratives on a decision" and "Another order is another version", and in `crates/boom/tests/edges.rs` for "An outline with its narratives". Proved by `boom:test`
- [x] 1.3 Attach each outline to the whole system wherever it runs, and refuse `--repository` and `--context` on it. Tests in `views.rs` for "Proposed in a worktree of a repository" and "An outline with a repository". Proved by `boom:test`
- [x] 1.4 `judge.rs` refuses the first version of an outline of the view `book` while another outline of the view `book` has a version that is not rejected, and names that outline. Tests in `views.rs` for "A second book" and "A book after a rejected book". Proved by `boom:test`
- [x] 1.5 `hook.rs` gives no outline, as it gives no narrative. Tests in `views.rs` for "An outline accepted with its narratives", and in `crates/boom/tests/hooks.rs` for "Session start gives no outline". Proved by `boom:test`

## 2. One level for a narrative

- [x] 2.1 `judge.rs` refuses a narrative whose attachment names two repositories, two contexts, or a repository and a context, and says that a narrative binds one level. Tests in `crates/boom/tests/narratives.rs` for "A narrative of one repository", "A narrative of two repositories", "A narrative of a repository and a context" and "An amendment keeps one level". Proved by `boom:test`

## 3. The design book

- [x] 3.1 Add `view.rs`: one function that computes the book from the views of the store, with Part I, the chapters of Part II, the sections of each context, and each narrative with its shown version, its status, its count of open notes and whether the outline holds it. It reads the narratives, the contexts and the outline of the view `book`, and no other claim. Tests in `views.rs` for "Three levels", "A context of two repositories", "A repository with no narrative" and "A narrative that changes its level". Proved by `boom:test`
- [x] 3.2 Order each level, the chapters and the sections by the outline, then by name. Tests in `views.rs` for "The outline's order", "The order of the chapters" and "The same book from two runs". Proved by `boom:test`
- [x] 3.3 Give each narrative that the outline does not hold at the end of its level, marked, and leave out a narrative whose every version is rejected and that the outline does not hold. Tests in `views.rs` for "A narrative not in the outline" and "A rejected narrative". Proved by `boom:test`
- [x] 3.4 `boom show <outline>` gives the title, the view, the body, the narratives in order with their mark out of date, the book, and the narratives that the outline does not hold. `--json` gives `view`, `narratives`, `book` and `unheld`. Tests in `views.rs` for "A narrative out of date" and "The book in words". Proved by `boom:test`

## 4. The page

- [x] 4.1 `page.rs` gives the page of the book in place of the list of narratives: the outline's head, each part, chapter and section as a heading, and each narrative as a link with its status, its open notes and its mark. With no outline, it says how to propose one. Tests in `crates/boom/tests/page.rs` for "The book on the page", "A narrative marked on the page" and "No outline". Proved by `boom:test`
- [x] 4.2 Add `--view <outline>` to `boom page`. With no name and no `--view`, `boom page` serves the book and prints `Boom: page of the design book at <URL>`. Boom refuses a `--view` that is not an outline, and a name with `--view`. Tests in `page.rs` for "The design book served", "A view served" and "A view that is not an outline". Proved by `boom:test`
- [x] 4.3 The page of a narrative that the book gives links to the book and to the narratives before and after it in the book. Tests in `page.rs` for "The narratives around one" and "The first narrative". Proved by `boom:test`

## 5. `boom mcp`

- [x] 5.1 `mcp.rs`: the tool `propose` takes the kind `outline`, `view` and `narratives`. Tests in `crates/boom/tests/mcp.rs` for "A tool's arguments are its command's" and "An outline proposed through its tool". Proved by `boom:test`

## 6. The pages and the gate

- [x] 6.1 Write the outline, the design book and `boom page --view` into `README.md` and `CLAUDE.md`, in Plain. Proved by `devenv shell -- moon run root:format root:terms`
- [x] 6.2 Run `boom terms check` on each file that the change writes, before each commit. Proved by `root:terms`
- [x] 6.3 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main -- moon run <task>`, then `sluice exec main` once the work is committed
