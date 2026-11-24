No task adds a part of the repository. The work is in the crate `boom`,
so this change adds no moon project. Build on main after the changes
`a-note-from-the-page-is-written` and `a-new-draft-replaces-the-one-before`
land, since both touch `page.rs`. Each task names the moon task that
proves it, run as `sluice exec main -- moon run <project>:<task>`. Each
scenario of the delta spec is one test, named by its scenario, in
`crates/boom/tests/page.rs`, but the two scenarios of "The sidebar
folds on a narrow window", which task 5.1 proves in a browser.

## 1. The sidebar

- [ ] 1.1 In `page.rs`, compute the sidebar from `view::book` once for each request, with the narrative of the page marked `aria-current="page"` and each narrative that the outline does not hold marked, and give it on the book page, each narrative's page, each claim's page and the page of a search. Tests for "The sidebar of a narrative's page", "The sidebar of the book page", "A narrative that the outline does not hold", "A section of a context" and "The sidebar of a version's page". Proved by `boom:test`
- [ ] 1.2 Give the "Contents" control and the style that folds the sidebar under 60rem and shows it as a column of 17rem at 60rem or wider, in both schemes. Proved by `boom:test` and task 5.1

## 2. The search

- [ ] 2.1 In `search.rs`, search the paragraphs of the book's narratives, at the version that the book gives, with `search::search`, each paragraph a row, and give for each narrative the first paragraph that holds each word. Unit tests in `search.rs` for the first paragraph and for words in two paragraphs. Proved by `boom:test`
- [ ] 2.2 In `page.rs`, add the form to the sidebar and the route `GET <base>/search`: the narratives found, then each other claim that `boom search --all` gives, each as a link. Tests for "A word in a narrative's paragraph", "A narrative that is proposed", "A claim found", "The forms of a word", "Words in two paragraphs", "No word to search" and "A search with no token". Proved by `boom:test`

## 3. The page of a claim

- [ ] 3.1 Add the route `GET <base>/c/<name>[@<version>]` and the claim's page: its kind, version, status, statement, what it binds, body, versions, sources, the claims that it changes and the narratives of the book that include it; 303 for a narrative and an outline, 404 for no claim. Tests for "A claim's page", "A source out of date", "The name of a narrative" and "A claim accepted while the page is served". Proved by `boom:test`
- [ ] 3.2 Link the head of each claim's box on a narrative's page to the claim's page at the version meant. A test for "An included claim links to its page". Proved by `boom:test`

## 4. What the page keeps

- [ ] 4.1 Give the policy header and no `<script>` on the page of a search and the page of a claim. Tests for "The page of a search" and "The page of a claim" under "The navigation runs no script". The tests of the notes, the diagrams, the token and the policy that pass on main still pass. Proved by `boom:test`

## 5. The browser and the gate

- [ ] 5.1 A screenshot test in a real browser: on a copy of the store, run `boom page --no-open`, and load the book page, a narrative's page and the page of a search in Chromium with the site's pinned Playwright, at 1280 and at 400 pixels wide. At 400, click "Contents". The shots prove "A narrow window" and "A wide window". Keep the shots out of the tree, and attach them to the change's report
- [ ] 5.2 Say in `README.md`, in Plain, that the page gives the book's contents, a search and a page for each claim. Proved by `root:format` and `root:terms`
- [ ] 5.3 Run `sluice exec main` for the gate's checks: `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs`, `root:changes` and `root:terms`
