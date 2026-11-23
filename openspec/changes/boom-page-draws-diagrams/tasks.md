No task adds a part of the repository. The script is a file of the
crate `boom`, so this change adds no moon project. Each task names the
moon task that proves it, run as `devenv shell -- moon run
<project>:<task>`. Each scenario of the delta specs is one test, named
by its scenario, in the test file that its task names.

## 1. Mermaid's script in the crate

- [x] 1.1 Add `crates/boom/assets/mermaid/mermaid.min.js` and `crates/boom/assets/mermaid/LICENSE`, the files `dist/mermaid.min.js` and `LICENSE` of the package `mermaid@11.17.2`, byte for byte. Exclude both from treefmt by name, with the reason, in `treefmt.nix`. Proved by `root:format` and `root:terms`
- [x] 1.2 In `page.rs`, hold both files with `include_bytes!`, with the version and the SHA-256 of the script as constants. A unit test in `page.rs` for "The pinned script". Proved by `boom:test`

## 2. The diagram on the page

- [x] 2.1 Give a fenced `mermaid` code block as `<pre class="mermaid">` with its source as text. A page whose HTML holds one ends its body with Mermaid's `<script>` and a link to the license; a page with none holds no `<script>`. Tests in `crates/boom/tests/page.rs` for "A diagram in a body", "A body with no diagram" and "A script in a diagram". Proved by `boom:test`
- [x] 2.2 Serve the script and the license under the token, as `text/javascript` and `text/plain`, and change the policy header to allow `script-src 'self'`. Tests in `page.rs` for "The script served", "The license served" and "The script with no token", and the policy in "The page served". Proved by `boom:test`
- [x] 2.3 Put each diagram on a light card, in both schemes. Proved by `boom:test`

## 3. A diagram in the ledger

- [x] 3.1 Test that a `mermaid` block is one paragraph of kind `code` with the fingerprint of its bytes. A test in `crates/boom/tests/narratives.rs` for "A diagram is a paragraph". Proved by `boom:test`

## 4. The pages and the checks

- [x] 4.1 Say in `README.md`, in Plain, that the page runs Mermaid's script alone, which boom serves. Proved by `root:format` and `root:terms`
- [x] 4.2 Run the checks of the projects that the change touches: `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `nix:typecheck`, `root:format`, `root:terms`, `root:specs` and `root:changes`.
