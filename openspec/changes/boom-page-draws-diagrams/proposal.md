## Why

A narrative's body is Markdown, and a design often draws its steps as
a Mermaid diagram in a fenced `mermaid` code block. The page of a
narrative runs no script, so it gives each such block as its source
text, not as a picture. The narrative `change-records@bd137df9ada7`
gives its chain of steps as Mermaid source on `boom page
change-records`.

The user ruled on 2026-11-23: "Build Mermaid's script into boom; the
page loads it from 127.0.0.1."

## What Changes

- **A diagram on the page.** The page gives a fenced code block whose
  language is `mermaid` as a `<pre class="mermaid">` that holds its
  source as text. In a browser, Mermaid's script draws it as a picture.
  A reader with no script reads its source.
- **Mermaid's script is part of boom.** boom holds Mermaid's script,
  version 11.17.2, in its binary, with Mermaid's license. The script is
  the checked-in file `crates/boom/assets/mermaid/mermaid.min.js`, and a
  test holds it to its version and its SHA-256. `boom page` serves it
  under the page's token from `127.0.0.1`, with no network.
- **The page runs one script.** A page that holds a diagram loads
  Mermaid's script and no other script. A page with no diagram holds no
  `<script>` element. The policy header allows a script from the page's
  own origin alone. The page still loads nothing from another host.
- **The license beside the script.** `boom page` serves Mermaid's
  license beside the script, and a page with a diagram links to it.
- **A diagram is its source in the ledger.** A diagram in a body is one
  paragraph of kind `code`. boom keeps its source as the body's bytes,
  and only the page draws it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `narrative-page`: the page draws a diagram with Mermaid's script,
  which boom serves, and its policy allows that one script.
- `narratives`: a diagram in a body is a `mermaid` code block, one
  paragraph of kind `code`, kept as its source.

## Impact

- `crates/boom`: `page.rs` (the diagram, the script, the license and
  the policy), a new folder `crates/boom/assets/mermaid/` with
  Mermaid's `mermaid.min.js` and `LICENSE` as Mermaid's package gives
  them, and the tests in `crates/boom/tests/page.rs` and
  `crates/boom/tests/narratives.rs`.
- The binary grows by about 3.6 MB, the size of Mermaid's script.
- `treefmt.nix`: an exclude by name for Mermaid's two files, with its
  reason.
- `README.md`: the page runs Mermaid's script alone.
- No new crate, no new dependency in `Cargo.lock`, no new moon project.
