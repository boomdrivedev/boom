No task adds a part of the repository. The work is in the crate `boom`,
so this change adds no moon project. Build on the change
`the-book-page-has-navigation`, in one bolt with it: its bar, its
sidebar and its search are the shell of this one. Each task names the moon task
that proves it, run as `sluice exec main -- moon run <project>:<task>`.
Each scenario of the delta spec is one test, named by its scenario, in
`crates/boom/tests/page.rs`.

## 1. The claims of each level

- [x] 1.1 In `view.rs`, compute for each level, the whole system, each repository and each context, its accepted and its proposed versions of each kind but narrative and outline, from what each version binds, with a claim of two repositories under each. The tests "A rule of a context" and "A rule of two repositories" prove it. Proved by `boom:test`
- [x] 1.2 In `view.rs`, compute for a claim the claims whose accepted or proposed version names it with `--source` or `--changes`, each with the version meant. The test "A claim named as a source" proves an edge out of date. Proved by `boom:test`

## 2. The views

- [x] 2.1 In `page.rs`, give the views in the bar, the current one with `aria-current="page"`, and on a view the sidebar of its levels with their counts. Tests for "The bar of the rules view" and "The sidebar of a view". Proved by `boom:test`
- [x] 2.2 Add the route `GET <base>/rules`: each accepted invariant, then assumption, of each level, by name, its body in `<details>`; with `?repository=<name>`, the levels and the claims of `boom query --repository <name>`. Tests for "A rule of a repository", "A rule of a context", "A rule of two repositories", "Not a rule" and "The rules of boom". Proved by `boom:test`
- [x] 2.3 Add the route `GET <base>/decisions`: each accepted decision by level, in the order of the entry that accepted its first version, with its date, marked where a newer version is proposed. Tests for "Decisions in the order accepted" and "A newer version proposed". Proved by `boom:test`
- [x] 2.4 Add the route `GET <base>/terms`: each accepted term by repository, by its word, with no old word; and the old words on a term's page. Tests for "A term in the terms view" and "The page of a term". Proved by `boom:test`
- [x] 2.5 Add the route `GET <base>/claims/`: each accepted claim of every kind but narrative and outline, by level, then by kind, then by name. A test for "Every kind". Proved by `boom:test`
- [x] 2.6 Give each context as a section of each view under its repository, with its accepted globs, and on a context's page its globs and the accepted claims that bind it. Tests for "A context's section" and "A context's page". Proved by `boom:test`

## 3. The proposed versions

- [x] 3.1 Give, at the head of each view, "Proposed, binding nothing": each proposed version of its kinds, grouped by the level it binds, in a dashed frame marked "proposed", with `boom accept <name>@<version>`, and one `boom accept` with each version of the list; no rejected or superseded version in any view. Tests for "A proposed decision", "One command for every proposal", "A rejected version" and "Nothing proposed". Proved by `boom:test`
- [x] 3.2 Give the page of a proposed version in a dashed frame marked "proposed, binds nothing", with `boom accept` and `boom reject --reason`, and no `POST` form. A test for "The page of a proposed decision". Proved by `boom:test`

## 4. A claim's page in full

- [x] 4.0 Add the route `GET <base>/claims/<name>[@<version>]`: the claim's kind, version, status, statement, binding, body, versions, sources, the claims it changes, and each narrative of the book that includes it, linked to its paragraph; 404 for a name that no claim has. Tests for "The page of an invariant" and "No such claim". Proved by `boom:test`
- [x] 4.1 Give "History" on a claim's page: each version neither proposed nor accepted, with its status, the entry that closed it, the person and a rejection's reason; and the accepted and proposed versions alone in its versions line. A test for "A superseded and a rejected version". Proved by `boom:test`
- [x] 4.2 Give the claims that name the claim, each as a link, with the version meant, marked out of date. A test for "A claim named as a source". Proved by `boom:test`
- [x] 4.3 Give the origin as its name, fingerprint and `boom show --artifact <fingerprint>`, or as `git:<repository>@<commit>:<path>`, and serve no kept file's bytes. A test for "A kept file". Proved by `boom:test`
- [x] 4.4 Give the notes on each paragraph of a narrative of the book that includes the claim, open or answered, each with a link to its paragraph. A test for "A note on a paragraph that includes the claim". Proved by `boom:test`

## 5. Filters and search

- [x] 5.1 Give each view a `GET` form with `kind`, `status`, `repository`, `level` and `q`; `status` `both` when left out; `q` with the matching of `search::words` and `search::search` over the claims the other fields keep; 400 with the values a field takes for a value it does not take; the count of what the view gives. Tests for "The proposed decisions of boom", "A search in the rules view", "A proposed claim found" and "A kind that is not one". Proved by `boom:test`

## 6. What the views keep

- [x] 6.1 Compute each view from the views at the log's last entry for each request. A test for "A claim accepted while the view is served". Proved by `boom:test`
- [x] 6.2 Give the policy header, no `<script>`, no URL of another host, 404 with no token, and each filter's value escaped. Tests for "The rules view", "A view with no token" and "A filter's value given back". The tests of the book, the notes, the diagrams, the token and the policy that pass on main still pass. Proved by `boom:test`

## 7. The words and the gate

- [x] 7.1 Say in `README.md`, and in the views table of the site's `how-its-used` page, in Plain, that `boom page` gives the rules, the decisions, the terms and every claim. Proved by `root:format`, `root:terms` and `site:build`
- [ ] 7.2 Run `sluice exec main` for the gate's checks: `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `site:build`, `root:specs`, `root:changes` and `root:terms`
