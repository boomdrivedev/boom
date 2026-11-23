## Context

`boom page` serves the page of a narrative on `127.0.0.1`, under a
token, computed from the log for each request (`narrative-page`). The
page holds its own style and no script. Its policy header is
`default-src 'none'; style-src 'unsafe-inline'; form-action 'self'`. A
fenced `mermaid` code block in a body thus shows as its source.

The user ruled on 2026-11-23: "Build Mermaid's script into boom; the
page loads it from 127.0.0.1."

No other tool of the family serves a page or draws a diagram: flume,
sluice, weir, timber and Boomdrive hold no Mermaid and no policy
header. boom owns this design, and no other tool must change with it.

This design keeps these claims of the store:

- `views-computed-from-the-ledger`: the picture is drawn in the
  browser, from the source that the log holds. No file and no commit
  holds a picture.
- `boom-mcp-serves-an-agent-the-commands-it-may-run-and-none-a-person-must-sign`:
  `boom mcp` serves no `page`, so this change adds no tool.
- `names-use-the-domains-terms`: `diagram`, `script` and `license` are
  the words of Mermaid and of HTTP. `mermaid` is the language name that
  Markdown writers give the block.
- `nothing-commercial`: Mermaid is an open-source project, under the
  MIT license. The page names no service of another host, and the
  script holds no name of one.
- `no-credential-in-the-repository`: the token stays random for each
  run of `boom page`.
- `tools-at-their-pinned-versions`: the script is one version, the
  file that the tree holds.
- `each-tracked-file-has-a-formatter`: treefmt excludes Mermaid's two
  files by name, with the reason.

## Goals / Non-Goals

**Goals:**

- A `mermaid` code block in a narrative's body shows as a picture on
  the page, with no network.
- A reader with no script reads its source.
- The page runs Mermaid's script and no other.

**Non-Goals:**

- A picture in `boom show` or in a hook. Those give the source.
- Another kind of diagram, or a script for another purpose.
- A Mermaid setting from the page, such as a theme. The page runs no
  script of its own to give one.

## Decisions

1. **The script is a checked-in file in the crate.** The file is
   `dist/mermaid.min.js` of the package `mermaid@11.17.2`, byte for
   byte, with the package's `LICENSE`, in
   `crates/boom/assets/mermaid/`. `include_bytes!` puts both in the
   binary. A unit test holds the script to its SHA-256 and to the
   version that it names. The package's integrity is
   `sha512-V6K3C8EBdEsPFZXSKMJe6ppQOENxuHARr9GvHX4hh47lAbhMRD9qf4oEK7LoaRQxULMa80/qt5gHO73aCleBBg==`.
   A cargo build and the flake's build read the file from the tree, with
   no network.

   Not taken:
   - A crate that wraps Mermaid. No crate gives Mermaid's own release at
     a version that its authors keep.
   - A picture drawn by boom in Rust. No Rust crate draws Mermaid's
     whole language, and Mermaid's own command line needs a browser.
   - The script from another host. The user's ruling refuses it, and
     the page works with no network.
2. **The page loads the script only where it draws a diagram.** A page
   whose HTML holds a diagram ends its body with one
   `<script src="<base>/mermaid-11.17.2.min.js"></script>`. A page with
   no diagram holds no `<script>` element. Mermaid draws each
   `.mermaid` element when the page loads, by its own default
   `startOnLoad`, so the page needs no script of its own to start it.
3. **The source stays in the page.** The renderer gives the block as
   `<pre class="mermaid">`, its source as escaped text. With no script,
   the reader reads the source. With the script, Mermaid puts its
   picture in place of the source. Raw HTML of a body stays text, so
   only the renderer writes `<pre class="mermaid">`, and boom decides
   to load the script from that mark.
4. **The policy allows one origin for scripts.** The header becomes
   `default-src 'none'; script-src 'self'; style-src 'unsafe-inline';
   form-action 'self'`. Each other response of `boom page` is HTML or
   text with `X-Content-Type-Options: nosniff`, so the browser runs no
   other response as a script. A script inline, from another host, or
   from an attribute is refused. Mermaid writes its picture's style as
   a `<style>` element, which `style-src 'unsafe-inline'` already
   allows. Mermaid's default `securityLevel` is `strict`: it cleans
   each label of HTML and turns off each click, so a body's diagram
   runs nothing of its own.
5. **The script and its license are paths of the page.** `GET
   <base>/mermaid-11.17.2.min.js` answers the script, `text/javascript`.
   `GET <base>/mermaid-11.17.2.LICENSE` answers the license,
   `text/plain`. Each path starts with the token, as each path of the
   page does. A page with a diagram names Mermaid, its version and a
   link to the license at its foot.
6. **A light card for the picture.** Mermaid's default theme draws dark
   lines. The page puts each diagram on a light card, in a light and a
   dark scheme, so the picture reads in both.
7. **A diagram is its source in the ledger.** The `narratives` delta
   says that a diagram in a body is a `mermaid` code block: one
   paragraph of kind `code`, whose fingerprint is that of its source.
   A note on a diagram is a note on that paragraph.

## Risks / Trade-offs

- [The binary grows by about 3.6 MB] → The script is part of the
  binary, which the ruling asks for. No other path keeps the page
  working with no network.
- [A new Mermaid version] → A change replaces the two files and the
  version and SHA-256 that the test holds, in one commit.
- [Mermaid holds a flaw] → The policy refuses each other host and each
  script that is not Mermaid's, and `securityLevel` stays `strict`.
- [A diagram with an error] → Mermaid draws its own error mark in place
  of the picture. The source stays in the log, and `boom show` gives
  it.
