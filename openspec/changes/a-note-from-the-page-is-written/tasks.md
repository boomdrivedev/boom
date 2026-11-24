## 1. The page

- [x] 1.1 Give `Referrer-Policy: same-origin` in each response and in the `<meta name="referrer">` of each page, from one constant
- [x] 1.2 `boom:test` proves the scenarios "A note sent from the page's form" and "A form of another site with no referrer", and the first fails under `no-referrer`

## 2. The browser and the gate

- [x] 2.1 Send a note from the page's form in a browser, on a copy of the store, before and after the change
- [x] 2.2 Run `boom:format`, `boom:lint`, `boom:typecheck`, `boom:test`, `root:specs`, `root:changes` and `root:terms` through `sluice exec main`
