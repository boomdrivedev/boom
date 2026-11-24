No task adds a part of the repository. The work is in the crate `boom`,
so this change adds no moon project. Build on main after the changes
`a-note-from-the-page-is-written` and `a-new-draft-replaces-the-one-before`
land, since both touch `page.rs`. This change lands in one bolt with the
change of the claims view, which serves `<base>/claims/`. Each task
names the moon task that proves it, run as `sluice exec main -- moon run
<project>:<task>`. Each scenario of the delta spec is one test, named
by its scenario, in `crates/boom/tests/page.rs`, but the two scenarios
of "The sidebar folds on a narrow window", which task 5.1 proves in a
browser.

## 1. The shell

- [x] 1.1 In `page.rs`, replace `document` with `shell`: the bar with the views of `VIEWS`, the current view marked, the search form and the log's entry; the control and `<nav class="side">` where the view gives a sidebar; `<main>`; and Mermaid's foot. `VIEWS` holds "The book", `<base>/`, and "The claims", `<base>/claims/`. Tests for "The bar of a narrative's page", "The bar of a search" and "The order of the views". Proved by `boom:test`
- [x] 1.2 Give the style that folds the sidebar under 60rem, shows it as a column of 17rem at 60rem or wider, and wraps the bar on a narrow window, in both schemes. Proved by `boom:test` and task 5.1

## 2. The book's sidebar

- [x] 2.1 Compute the book's sidebar from `view::book` once for each request, with the narrative of the page marked `aria-current="page"` and each narrative that the outline does not hold marked, and give it on the book page, each narrative's page and the page of a search. Tests for "The sidebar of a narrative's page", "A section of a context", "Narratives only", "The current narrative", "The sidebar of the book page", "A narrative that the outline does not hold" and "The sidebar of a version's page". Proved by `boom:test`

## 3. The search

- [x] 3.1 In `search.rs`, search the paragraphs of the book's narratives, at the version that the book gives, with `search::search`, each paragraph a row, and give for each narrative the first paragraph that holds each word. Unit tests in `search.rs` for the first paragraph and for words in two paragraphs. Proved by `boom:test`
- [x] 3.2 In `page.rs`, add the route `GET <base>/search`: the narratives found, then each other claim that `boom search --all` gives, each as a link. Tests for "A claim found", "The forms of a word", "No word to search", "A search with no token", "A word in a narrative's paragraph", "A narrative that is proposed" and "Words in two paragraphs". Proved by `boom:test`

## 4. The claims' links and what the page keeps

- [x] 4.1 Link the head of each claim's box on a narrative's page to `<base>/claims/<name>@<version>`, at the version meant. Tests for "An included claim's link" and "An included claim out of date". Proved by `boom:test`
- [x] 4.2 Give the policy header and no `<script>` on the page of a search and on a page with no diagram. Tests for "The page of a search" and "The book page" under "The shell runs no script". The tests of the notes, the diagrams, the token and the policy that pass on main still pass. Proved by `boom:test`

## 5. The browser and the gate

- [x] 5.1 A screenshot test in a real browser: on a copy of the store, run `boom page --no-open`, and load the book page, a narrative's page and the page of a search in Chromium with the site's pinned Playwright, at 1280 and at 400 pixels wide. At 400, click "Contents". The shots prove "A narrow window" and "A wide window". Keep the shots out of the tree, and attach them to the change's report
- [x] 5.2 Say in `README.md`, in Plain, that the page gives a bar of views, the book's contents and a search. Proved by `root:format` and `root:terms`
- [ ] 5.3 Run `sluice exec main` for the gate's checks: `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs`, `root:changes` and `root:terms`
