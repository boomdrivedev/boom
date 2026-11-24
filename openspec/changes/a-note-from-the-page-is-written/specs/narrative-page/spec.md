## ADDED Requirements

### Requirement: A browser sends the page's own origin with its form

Each response SHALL give the header `Referrer-Policy: same-origin`, and each page SHALL give the same policy in `<meta name="referrer">`. A browser then sends the page's own `Origin` with the page's form, and sends no URL of the page to another host. Boom SHALL refuse a `POST` with `Origin: null`.

#### Scenario: A note sent from the page's form

- **WHEN** a browser sends the form of paragraph 3 of the page, with the `Origin` that it sends under the page's referrer policy
- **THEN** the log gets one `noted` entry for paragraph 3, and the page at `#p3` gives the note

#### Scenario: A form of another site with no referrer

- **WHEN** a `POST` with the token gives the header `Origin: null`
- **THEN** boom answers 403, and appends nothing
