## Context

`boom page` serves the page on `127.0.0.1`, each path under a token of
128 random bits. A note is a plain HTML form. Boom refuses a `POST`
whose `Origin` is not `http://127.0.0.1:<port>`. Each response gave
`Referrer-Policy: no-referrer`, and each page `<meta name="referrer"
content="no-referrer">`, so that no URL of the page, with its token,
reaches another host.

Fetch, "append a request `Origin` header", gives `Origin: null` for a
`POST` that is not CORS under the policy `no-referrer`, also for a
`POST` to the page's own origin. Thus boom refused each note of the
page's own form.

This change keeps these claims:

- `views-computed-from-the-ledger@36a056c1145b`: the page that a note
  goes back to is computed from the log, so it gives the note.
- `no-credential-in-the-repository@a349a974ad13`: the token is made for
  each run, and the tree holds none.
- `the-gate-validates-specs-strictly@d8e057526e31` and
  `open-changes-archive-cleanly@de24a2987568`: the change passes
  `openspec validate --strict` and archives alone.

## Goals / Non-Goals

**Goals:**

- A note sent from the page's own form reaches the log, and the page
  gives it.
- A `POST` from another site is refused, as before.
- No URL of the page reaches another host, as before.

**Non-Goals:**

- A change to the note, the log or the views.

## Decisions

### The referrer policy `same-origin`

Under `same-origin` a browser sends the page's own `Origin` with a
request to the page's own origin, and no referrer to another host. The
check of the `Origin` stays as it is, and boom keeps nothing more.

The alternatives:

- A token in each form, that boom makes and checks. The path already
  holds a token of 128 bits, which another site does not know. A second
  token is a second thing to make, to write into each form and to check,
  for no more safety.
- No check of the `Origin`. The token of the path then is the only
  check. The check of the `Origin` costs nothing, and it refuses a form
  of another site where the token has leaked, so it stays.
- `strict-origin`. It also keeps the `Origin` of the page's own form,
  but sends the page's origin, with its port, to each other host that a
  link of a body names. `same-origin` sends nothing to another host.

### Boom refuses `Origin: null`

A form of another site whose page gives `no-referrer` sends
`Origin: null`. Boom refuses it, as it refuses each `Origin` that is not
the page's own.

## Risks / Trade-offs

- A browser that sends no `Origin` with a form `POST` cannot send a
  note. Each browser that the page is for sends it, so the check stays.
