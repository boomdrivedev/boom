## Why

On 2026-11-23 the user sent a note from `boom page
the-links-of-the-lineage` in the browser, and no note entry is in the
log. Each response and each page give the referrer policy
`no-referrer`. Under that policy a browser sends `Origin: null` with a
form, also with a form that the page posts to itself. Boom refuses a
`POST` whose `Origin` is not the page's own, so boom refused the note of
the page's own form with 403, "A note is sent from this page alone."

## What Changes

- Each response gives the header `Referrer-Policy: same-origin`, and
  each page the same policy in its `<meta name="referrer">`.
- A browser then sends the page's own `Origin` with the page's form,
  and boom writes the note. A request to another host gets no URL of the
  page, so no token leaves the page, as before.
- Boom still refuses a `POST` whose `Origin` is not the page's own,
  `null` too, so a form of another site is refused as before.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `narrative-page`: a new requirement, "A browser sends the page's own
  origin with its form".

## Impact

- `crates/boom/src/page.rs`: the referrer policy of each response and
  each page.
- `crates/boom/tests/page.rs`: a test of a note sent from the page's
  form as a browser sends it, and of a form of another site with no
  referrer.
- No change to the log, the views, the token or the check of the
  `Origin`.
