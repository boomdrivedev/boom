## ADDED Requirements

### Requirement: The page draws a diagram

The page SHALL give a fenced code block whose language is `mermaid` as `<pre class="mermaid">`, with its source as text. A page that holds one SHALL load one script, `<base>/mermaid-<version>.min.js`, and SHALL link to `<base>/mermaid-<version>.LICENSE`. A page that holds none SHALL hold no `<script>` element.

#### Scenario: A diagram in a body

- **WHEN** a narrative's body holds a fenced `mermaid` code block with `a --> b`, and its page is asked for
- **THEN** the page gives `<pre class="mermaid">` with the text `a --&gt; b`, one `<script>` element, whose source is `/<token>/mermaid-11.17.2.min.js`, and a link to `/<token>/mermaid-11.17.2.LICENSE`

#### Scenario: A body with no diagram

- **WHEN** a narrative's body holds a fenced `rust` code block and no `mermaid` block, and its page is asked for
- **THEN** the page holds no `<script>` element

### Requirement: boom serves Mermaid's script from itself

Boom SHALL hold Mermaid's script and Mermaid's license in its binary, at the version of the file `crates/boom/assets/mermaid/mermaid.min.js`. `GET <base>/mermaid-<version>.min.js` SHALL answer the script, with `Content-Type: text/javascript`. `GET <base>/mermaid-<version>.LICENSE` SHALL answer the license, with `Content-Type: text/plain`. Boom SHALL need no network for either.

#### Scenario: The script served

- **WHEN** `boom page` runs, and a request with the token asks for `mermaid-11.17.2.min.js`
- **THEN** boom answers 200 with `Content-Type: text/javascript`, and the bytes of `crates/boom/assets/mermaid/mermaid.min.js`

#### Scenario: The license served

- **WHEN** `boom page` runs, and a request with the token asks for `mermaid-11.17.2.LICENSE`
- **THEN** boom answers 200 with `Content-Type: text/plain`, and the bytes of `crates/boom/assets/mermaid/LICENSE`

#### Scenario: The script with no token

- **WHEN** a request asks for `/mermaid-11.17.2.min.js` with no token
- **THEN** boom answers 404

#### Scenario: The pinned script

- **WHEN** the tests of the crate run
- **THEN** the SHA-256 of the script that boom holds is the one that the code names, and the script names the version `11.17.2`

## MODIFIED Requirements

### Requirement: The page runs nothing from a body or another host

The page SHALL give each raw HTML of a body as text. Each response SHALL have the header `Content-Security-Policy: default-src 'none'; script-src 'self'; style-src 'unsafe-inline'; form-action 'self'`. The page SHALL hold its own style, SHALL load nothing from another host, and SHALL hold no script but Mermaid's, which boom serves.

#### Scenario: A script in a body

- **WHEN** a narrative's body holds `<script>alert(1)</script>`
- **THEN** the page gives the characters `<script>alert(1)</script>` as text, and the HTML of the page holds no `<script>` element

#### Scenario: Nothing from another host

- **WHEN** the page of a narrative is asked for
- **THEN** its HTML holds no URL of another host, and the response has the policy header

#### Scenario: A script in a diagram

- **WHEN** a narrative's body holds a fenced `mermaid` code block with `a["<script>alert(1)</script>"] --> b`
- **THEN** the page gives the characters `<script>alert(1)</script>` as text in the block, and the HTML of the page holds one `<script>` element, Mermaid's
