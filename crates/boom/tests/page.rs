//! `boom page`: the page of a narrative, with the claims it includes and
//! the notes on its paragraphs, served on 127.0.0.1 under a token and
//! computed from the log for each request (narrative-page).

mod support;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use boom::page::{Context, MERMAID_VERSION, POLICY};
use boom::store::Store;
use serde_json::Value;
use support::{Place, NARRATIVE};

const NAME: &str = "boom-end-to-end";
const TITLE: &str = "Boom, end to end";
const STATUS: &str = "status-read-from-events";
const DRIVE: &str = "Say what a drive is.";

fn version(answer: &Value) -> String {
    answer["version"].as_str().unwrap().to_owned()
}

/// The HTML of the page of `target`, as the function gives it.
fn html(place: &Place, name: &str, version: Option<&str>, person: Option<&str>) -> String {
    let context = Context {
        base: "/t".to_owned(),
        person: person.map(str::to_owned),
    };
    Store::open(&place.store)
        .unwrap()
        .read(|conn| boom::page::narrative_page(conn, name, version, &context))
        .unwrap()
}

/// The part of `html` from the paragraph `n` to the next one.
fn paragraph(html: &str, n: u64) -> String {
    let start = html
        .find(&format!("id=\"p{n}\""))
        .unwrap_or_else(|| panic!("no paragraph {n}: {html}"));
    let rest = &html[start..];
    let end = rest.find("</section>").unwrap();
    rest[..end].to_owned()
}

/// The part of `html` after the body.
fn after_body(html: &str) -> String {
    html[html.find("</article>").unwrap()..].to_owned()
}

/// A store whose one person is chuck, with `status-read-from-events`
/// accepted and the narrative proposed including it, giving the
/// narrative's version.
fn one_narrative(place: &Place, extra: &[&str]) -> String {
    let chuck = place.made_by("chuck");
    let status = place.proposed(STATUS, "A record never holds its status.");
    place.accepted(STATUS, &status, &chuck);
    let mut args = vec!["--includes", STATUS, "--repository", "boom"];
    args.extend(extra);
    version(&place.narrative(NAME, TITLE, NARRATIVE, &args))
}

#[test]
fn what_the_page_gives() {
    let place = Place::new();
    let v1 = one_narrative(&place, &[]);
    let html = html(&place, NAME, None, Some("chuck"));
    assert!(html.contains(&format!("<h1>{TITLE}</h1>")), "{html}");
    assert!(html.contains(&format!("{NAME}@{v1}")), "{html}");
    assert!(html.contains("proposed by an agent (run-1)"), "{html}");
    assert!(html.contains("binds boom"), "{html}");
    assert!(
        html.contains(&format!("href=\"/t/n/{NAME}@{v1}\">{v1}</a> proposed")),
        "{html}"
    );
    for n in 1..=5 {
        assert!(html.contains(&format!("id=\"p{n}\"")), "{n}");
        assert!(html.contains(&format!("href=\"#p{n}\"")), "{n}");
    }
}

#[test]
fn the_accept_of_a_proposed_version() {
    let place = Place::new();
    let v1 = one_narrative(&place, &[]);
    let html = html(&place, NAME, Some(&v1), Some("chuck"));
    assert!(
        html.contains(&format!("boom accept {NAME}@{v1} --key &lt;key&gt;")),
        "{html}"
    );
    assert!(!html.contains("accept\""), "no form accepts: {html}");
    assert!(!html.contains("/accept"), "no form accepts: {html}");
}

#[test]
fn a_note_at_its_paragraph() {
    let place = Place::new();
    let v1 = one_narrative(&place, &[]);
    let note = place.noted(&format!("{NAME}@{v1}"), 3, DRIVE);
    let html = html(&place, NAME, Some(&v1), Some("chuck"));
    let third = paragraph(&html, 3);
    assert!(third.contains(note["note"].as_str().unwrap()), "{third}");
    assert!(third.contains("chuck"), "{third}");
    assert!(third.contains(DRIVE), "{third}");
    assert!(third.contains(" · open"), "{third}");
    assert!(!paragraph(&html, 2).contains(DRIVE));
}

#[test]
fn a_note_answered_by_the_next_version() {
    let place = Place::new();
    let v1 = one_narrative(&place, &[]);
    let note = place.noted(&format!("{NAME}@{v1}"), 2, DRIVE)["note"]
        .as_str()
        .unwrap()
        .to_owned();
    let v2 = version(&place.narrative(
        NAME,
        TITLE,
        &format!("{NARRATIVE}\nA drive is a store with its runner.\n"),
        &[
            "--amends",
            &v1,
            "--answers",
            &format!("{note}=Paragraph 6 says it now."),
        ],
    ));
    let html = html(&place, NAME, Some(&v2), Some("chuck"));
    let end = after_body(&html);
    assert!(end.contains(&note), "{end}");
    assert!(
        end.contains("Boom keeps claims."),
        "the words of its paragraph in v1: {end}"
    );
    assert!(end.contains("Paragraph 6 says it now."), "{end}");
    assert!(end.contains(&format!("answered by {v2}")), "{end}");
}

#[test]
fn a_claim_at_its_paragraph() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let html = html(&place, NAME, None, Some("chuck"));
    let third = paragraph(&html, 3);
    assert!(third.contains("box claim"), "{third}");
    assert!(
        third.contains("A record never holds its status."),
        "{third}"
    );
    assert!(!after_body(&html).contains("box claim"), "{html}");
}

#[test]
fn a_claim_that_no_paragraph_names() {
    let place = Place::new();
    place.made_by("chuck");
    place.proposed(
        "events-are-appended",
        "boom appends an event, and never changes it.",
    );
    place.narrative(
        NAME,
        TITLE,
        NARRATIVE,
        &["--includes", "events-are-appended"],
    );
    let html = html(&place, NAME, None, Some("chuck"));
    let end = after_body(&html);
    assert!(
        end.contains("Claims it includes that no paragraph names"),
        "{end}"
    );
    assert!(
        end.contains("boom appends an event, and never changes it."),
        "{end}"
    );
}

#[test]
fn an_include_out_of_date_on_the_page() {
    let place = Place::new();
    let v1 = one_narrative(&place, &[]);
    let status = place.show(STATUS)["accepted"].as_str().unwrap().to_owned();
    let chuck = place.key("chuck", 7);
    let newer = place.amended(STATUS, "A record never holds its own status.", &status);
    place.accepted(STATUS, &newer, &chuck);
    let html = html(&place, NAME, Some(&v1), Some("chuck"));
    assert!(
        paragraph(&html, 3).contains(&format!("out of date: accepted now {newer}")),
        "{html}"
    );
}

/// The HTML of the page of the design book, as the function gives it.
fn book_html(place: &Place, outline: Option<&str>) -> String {
    let context = Context {
        base: "/t".to_owned(),
        person: None,
    };
    Store::open(&place.store)
        .unwrap()
        .read(|conn| boom::page::book_page(conn, outline, &context))
        .unwrap()
}

/// The item of the narrative `name` in `html`, a page of the book.
fn item(html: &str, name: &str) -> String {
    let start = html
        .find(&format!("<li data-narrative=\"{name}\">"))
        .unwrap_or_else(|| panic!("no item {name}: {html}"));
    let rest = &html[start..];
    rest[..rest.find("</li>").unwrap()].to_owned()
}

/// Proposes the outline of the book with `narratives`, giving its
/// version.
fn book_outline(place: &Place, narratives: &[&str]) -> String {
    let mut args = vec![
        "propose",
        "outline",
        BOOK,
        "The design book of Boomdrive",
        "--view",
        "book",
    ];
    for held in narratives {
        args.extend(["--narrative", held]);
    }
    args.extend(["--provenance", "run-1"]);
    version(&place.json(&args))
}

const BOOK: &str = "boomdrive-design-book";
const LOG: &str = "the-drives-log";
const RECORDS: &str = "change-records";

/// The store of "The book on the page": `the-drives-log` binds the whole
/// system, `boom-end-to-end` binds `boom` with two open notes, and
/// `change-records` binds `boom`; the outline holds the three in that
/// order.
fn book_store(place: &Place) {
    let v1 = one_narrative(place, &[]);
    place.noted(&format!("{NAME}@{v1}"), 1, "One.");
    place.noted(&format!("{NAME}@{v1}"), 2, "Two.");
    place.narrative(LOG, "The drive's log", "One log.\n", &["--system"]);
    place.narrative(
        RECORDS,
        "Boom's change record",
        "A record.\n",
        &["--repository", "boom"],
    );
    book_outline(place, &[LOG, NAME, RECORDS]);
}

#[test]
fn the_book_on_the_page() {
    let place = Place::new();
    book_store(&place);
    let html = book_html(&place, None);
    assert!(
        html.contains("<h1>The design book of Boomdrive</h1>"),
        "{html}"
    );
    assert!(html.contains(&format!("outline <code>{BOOK}@")), "{html}");
    let part_one = html.find("<h2>Part I. The system</h2>").expect("Part I");
    let part_two = html
        .find("<h2>Part II. The repositories</h2>")
        .expect("Part II");
    let chapter = html.find("<h3>boom</h3>").expect("the chapter boom");
    let log = html.find(&format!("data-narrative=\"{LOG}\"")).unwrap();
    let end_to_end = html.find(&format!("data-narrative=\"{NAME}\"")).unwrap();
    assert!(part_one < log && log < part_two && part_two < chapter && chapter < end_to_end);
    let entry = item(&html, NAME);
    assert!(entry.contains(&format!("href=\"/t/n/{NAME}\"")), "{entry}");
    assert!(entry.contains("data-open=\"2\""), "{entry}");
    assert!(entry.contains("2 open notes"), "{entry}");
    assert!(!entry.contains("not in the outline"), "{entry}");
}

#[test]
fn a_narrative_marked_on_the_page() {
    let place = Place::new();
    book_store(&place);
    place.narrative(
        "boom-for-a-reader",
        "Boom, for a reader",
        "A short one.\n",
        &["--repository", "boom"],
    );
    let html = book_html(&place, None);
    let chapter = &html[html.find("<h3>boom</h3>").unwrap()..];
    let last = chapter.rfind("<li data-narrative=").unwrap();
    assert!(
        chapter[last..].starts_with("<li data-narrative=\"boom-for-a-reader\">"),
        "{chapter}"
    );
    assert!(
        item(&html, "boom-for-a-reader").contains("not in the outline"),
        "{html}"
    );
}

#[test]
fn no_outline() {
    let place = Place::new();
    one_narrative(&place, &[]);
    place.narrative(
        "boom-for-a-reader",
        "Boom, for a reader",
        "A short one.\n",
        &["--repository", "boom"],
    );
    let html = book_html(&place, None);
    let chapter = &html[html.find("<h3>boom</h3>").expect("the chapter boom")..];
    let first = chapter.find("boom-end-to-end").unwrap();
    let second = chapter.find("boom-for-a-reader").unwrap();
    assert!(first < second, "{chapter}");
    for name in [NAME, "boom-for-a-reader"] {
        assert!(item(&html, name).contains("not in the outline"), "{html}");
    }
    assert!(html.contains("boom propose outline"), "{html}");
}

#[test]
fn the_narratives_around_one() {
    let place = Place::new();
    book_store(&place);
    let html = html(&place, NAME, None, None);
    let nav = &html[html.find("<nav class=\"book\">").expect("the book's links")..];
    assert!(nav.contains("rel=\"contents\" href=\"/t/\""), "{nav}");
    assert!(
        nav.contains(&format!("rel=\"prev\" href=\"/t/n/{LOG}\"")),
        "{nav}"
    );
    assert!(
        nav.contains(&format!("rel=\"next\" href=\"/t/n/{RECORDS}\"")),
        "{nav}"
    );
}

#[test]
fn the_first_narrative() {
    let place = Place::new();
    book_store(&place);
    let html = html(&place, LOG, None, None);
    let nav = &html[html.find("<nav class=\"book\">").expect("the book's links")..];
    assert!(nav.contains("rel=\"contents\" href=\"/t/\""), "{nav}");
    assert!(
        nav.contains(&format!("rel=\"next\" href=\"/t/n/{NAME}\"")),
        "{nav}"
    );
    assert!(!nav.contains("rel=\"prev\""), "{nav}");
}

#[test]
fn a_script_in_a_body() {
    let place = Place::new();
    place.made_by("chuck");
    place.narrative(
        NAME,
        TITLE,
        "Before.\n\n<script>alert(1)</script>\n\nAn <script>alert(2)</script> inline one.\n",
        &[],
    );
    let html = html(&place, NAME, None, Some("chuck"));
    assert!(
        html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
        "{html}"
    );
    assert!(
        html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"),
        "{html}"
    );
    assert!(!html.to_lowercase().contains("<script"), "{html}");
}

#[test]
fn nothing_from_another_host() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let html = html(&place, NAME, None, Some("chuck"));
    assert!(!html.contains("http://"), "{html}");
    assert!(!html.contains("https://"), "{html}");
    assert!(!html.contains("//"), "{html}");
}

/// A body with a diagram, `a --> b`, between two paragraphs of text.
const DIAGRAM: &str = "Before.\n\n```mermaid\nflowchart LR\n  a --> b\n```\n\nAfter.\n";

/// narrative-page, "The page draws a diagram".
#[test]
fn a_diagram_in_a_body() {
    let place = Place::new();
    place.made_by("chuck");
    place.narrative(NAME, TITLE, DIAGRAM, &[]);
    let html = html(&place, NAME, None, Some("chuck"));
    assert!(
        paragraph(&html, 2).contains("<pre class=\"mermaid\">flowchart LR\n  a --&gt; b\n</pre>"),
        "{html}"
    );
    assert_eq!(html.matches("<script").count(), 1, "{html}");
    assert!(
        html.contains(&format!(
            "<script src=\"/t/mermaid-{MERMAID_VERSION}.min.js\"></script>"
        )),
        "{html}"
    );
    assert!(
        html.contains(&format!("href=\"/t/mermaid-{MERMAID_VERSION}.LICENSE\"")),
        "{html}"
    );
    assert_eq!(MERMAID_VERSION, "11.17.2");
}

/// narrative-page, "The page draws a diagram".
#[test]
fn a_body_with_no_diagram() {
    let place = Place::new();
    place.made_by("chuck");
    place.narrative(NAME, TITLE, "Before.\n\n```rust\nfn main() {}\n```\n", &[]);
    let html = html(&place, NAME, None, Some("chuck"));
    assert!(html.contains("language-rust"), "{html}");
    assert!(!html.contains("<pre class=\"mermaid\">"), "{html}");
    assert!(!html.contains("mermaid-"), "{html}");
    assert!(!html.to_lowercase().contains("<script"), "{html}");
}

/// narrative-page, "The page runs nothing from a body or another host".
#[test]
fn a_script_in_a_diagram() {
    let place = Place::new();
    place.made_by("chuck");
    place.narrative(
        NAME,
        TITLE,
        "```mermaid\nflowchart LR\n  a[\"<script>alert(1)</script>\"] --> b\n```\n",
        &[],
    );
    let html = html(&place, NAME, None, Some("chuck"));
    assert!(
        html.contains("a[\"&lt;script&gt;alert(1)&lt;/script&gt;\"] --&gt; b"),
        "{html}"
    );
    assert_eq!(html.to_lowercase().matches("<script").count(), 1, "{html}");
    assert!(
        html.contains(&format!(
            "<script src=\"/t/mermaid-{MERMAID_VERSION}.min.js\">"
        )),
        "{html}"
    );
}

// The server, through the binary.

/// A run of `boom page`, killed when the test ends.
struct Served {
    child: Child,
    url: String,
    port: u16,
    token: String,
}

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A free port of 127.0.0.1.
fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Starts `boom page <args> --no-open --port <port>` on the place's
/// store, in the place's folder, and reads its line.
fn served(place: &Place, args: &[&str]) -> Served {
    let port = free_port();
    let mut child = Command::new(env!("CARGO_BIN_EXE_boom"))
        .arg("page")
        .args(args)
        .args([
            "--no-open",
            "--port",
            &port.to_string(),
            "--store",
            &place.store_arg(),
        ])
        .current_dir(place.dir.path())
        .env_remove("BOOM_REPOSITORY")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut line = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    assert!(line.starts_with("Boom: page of "), "{line:?}");
    let url = line.trim_end().rsplit(" at ").next().unwrap().to_owned();
    let prefix = format!("http://127.0.0.1:{port}/");
    assert!(url.starts_with(&prefix), "{url}");
    let token = url[prefix.len()..].split('/').next().unwrap().to_owned();
    assert_eq!(token.len(), 32, "{url}");
    Served {
        child,
        url,
        port,
        token,
    }
}

/// A response: its status, its headers and its body, as text and as
/// bytes.
struct Response {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
    bytes: Vec<u8>,
}

impl Response {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(field, _)| field.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

/// Sends `method path` to `port` with `headers` and `body`.
fn send(port: u16, method: &str, path: &str, headers: &[(&str, &str)], body: &str) -> Response {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut request = format!("{method} {path} HTTP/1.1\r\nConnection: close\r\n");
    for (name, value) in headers {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    if !body.is_empty() || method == "POST" {
        request.push_str(&format!(
            "Content-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n",
            body.len()
        ));
    }
    request.push_str("\r\n");
    request.push_str(body);
    stream.write_all(request.as_bytes()).unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).unwrap();
    let split = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap_or(raw.len());
    let head = String::from_utf8_lossy(&raw[..split]).into_owned();
    let bytes = raw.get(split + 4..).unwrap_or_default().to_vec();
    let body = String::from_utf8_lossy(&bytes).into_owned();
    let mut lines = head.lines();
    let status = lines
        .next()
        .unwrap()
        .split(' ')
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    let headers = lines
        .filter_map(|line| line.split_once(": "))
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .collect();
    Response {
        status,
        headers,
        body,
        bytes,
    }
}

impl Served {
    fn host(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    fn get(&self, path: &str) -> Response {
        send(self.port, "GET", path, &[("Host", &self.host())], "")
    }

    fn page(&self, target: &str) -> Response {
        self.get(&format!("/{}/n/{target}", self.token))
    }

    fn post(&self, target: &str, origin: &str, body: &str) -> Response {
        send(
            self.port,
            "POST",
            &format!("/{}/n/{target}/note", self.token),
            &[("Host", &self.host()), ("Origin", origin)],
            body,
        )
    }
}

#[test]
fn the_page_served() {
    let place = Place::new();
    let v1 = one_narrative(&place, &[]);
    let page = served(&place, &[NAME]);
    assert!(page.url.ends_with(&format!("/n/{NAME}")), "{}", page.url);
    let path = page
        .url
        .trim_start_matches(&format!("http://{}", page.host()))
        .to_owned();
    let response = page.get(&path);
    assert_eq!(response.status, 200, "{}", response.body);
    assert!(
        response.body.contains(&format!("{NAME}@{v1}")),
        "{}",
        response.body
    );
    assert_eq!(response.header("Content-Security-Policy"), Some(POLICY));
}

/// The bytes of a file of Mermaid's in the crate.
fn mermaid_file(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/mermaid")
            .join(name),
    )
    .unwrap()
}

/// narrative-page, "boom serves Mermaid's script from itself".
#[test]
fn the_script_served() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let page = served(&place, &[NAME]);
    let response = page.get(&format!("/{}/mermaid-11.17.2.min.js", page.token));
    assert_eq!(response.status, 200);
    assert!(
        response
            .header("Content-Type")
            .is_some_and(|value| value.starts_with("text/javascript")),
        "{:?}",
        response.headers
    );
    assert_eq!(
        response.header("Content-Length"),
        Some(mermaid_file("mermaid.min.js").len().to_string().as_str())
    );
    assert!(response.bytes == mermaid_file("mermaid.min.js"));
}

/// narrative-page, "boom serves Mermaid's script from itself".
#[test]
fn the_license_served() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let page = served(&place, &[NAME]);
    let response = page.get(&format!("/{}/mermaid-11.17.2.LICENSE", page.token));
    assert_eq!(response.status, 200);
    assert!(
        response
            .header("Content-Type")
            .is_some_and(|value| value.starts_with("text/plain")),
        "{:?}",
        response.headers
    );
    assert_eq!(response.bytes, mermaid_file("LICENSE"));
}

/// narrative-page, "boom serves Mermaid's script from itself".
#[test]
fn the_script_with_no_token() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let page = served(&place, &[NAME]);
    for path in [
        "/mermaid-11.17.2.min.js".to_owned(),
        format!("/{}/mermaid-11.17.2.min.js", "0".repeat(32)),
    ] {
        let response = page.get(&path);
        assert_eq!(response.status, 404, "{path}");
        assert!(response.bytes.len() < 1024, "{path}");
    }
}

#[test]
fn not_a_narrative() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let outcome = place.run(&["page", STATUS, "--no-open"]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(
        outcome.stderr.contains("boom page gives a narrative"),
        "{outcome:?}"
    );
    assert!(outcome.stdout.is_empty(), "{outcome:?}");
}

#[test]
fn no_token() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let page = served(&place, &[NAME]);
    for path in [
        "/".to_owned(),
        format!("/n/{NAME}"),
        format!("/{}/n/{NAME}", "0".repeat(32)),
        format!("/{}x/n/{NAME}", page.token),
    ] {
        let response = page.get(&path);
        assert_eq!(response.status, 404, "{path}");
        assert!(!response.body.contains(NAME), "{path}: {}", response.body);
        assert!(!response.body.contains("Boom keeps"), "{path}");
    }
}

#[test]
fn another_host_name() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let page = served(&place, &[NAME]);
    let response = send(
        page.port,
        "GET",
        &format!("/{}/n/{NAME}", page.token),
        &[("Host", "example.com")],
        "",
    );
    assert_eq!(response.status, 403);
    assert!(!response.body.contains("Boom keeps"));
}

#[test]
fn idle() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let mut page = served(&place, &[NAME, "--idle", "1"]);
    let started = Instant::now();
    loop {
        if let Some(status) = page.child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(0));
            assert!(
                started.elapsed() >= Duration::from_secs(55),
                "{:?}",
                started.elapsed()
            );
            break;
        }
        assert!(
            started.elapsed() < Duration::from_secs(90),
            "boom page did not stop"
        );
        std::thread::sleep(Duration::from_millis(250));
    }
}

#[test]
fn a_note_from_the_terminal() {
    let place = Place::new();
    let v1 = one_narrative(&place, &[]);
    let page = served(&place, &[NAME]);
    assert!(!page.page(NAME).body.contains(DRIVE));
    place.noted(&format!("{NAME}@{v1}"), 2, DRIVE);
    let response = page.page(NAME);
    assert!(
        paragraph(&response.body, 2).contains(DRIVE),
        "{}",
        response.body
    );
}

/// Each file under `folder`, with its size, in order.
fn files(folder: &std::path::Path) -> Vec<(String, u64)> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(folder).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            found.push((path.display().to_string(), entry.metadata().unwrap().len()));
        }
    }
    found.sort();
    found
}

#[test]
fn no_file_written() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let names = |found: Vec<(String, u64)>| -> Vec<String> {
        found.into_iter().map(|(name, _)| name).collect()
    };
    let before = names(files(place.dir.path()));
    let page = served(&place, &[NAME]);
    for _ in 0..5 {
        assert_eq!(page.page(NAME).status, 200);
        assert_eq!(page.get(&format!("/{}/", page.token)).status, 200);
    }
    assert_eq!(names(files(place.dir.path())), before);
    drop(page);
    assert_eq!(names(files(place.dir.path())), before);
}

#[test]
fn a_note_sent() {
    let place = Place::new();
    let v1 = one_narrative(&place, &[]);
    let page = served(&place, &[NAME]);
    let target = format!("{NAME}@{v1}");
    let origin = format!("http://{}", page.host());
    let response = page.post(&target, &origin, "paragraph=3&words=Say+what+a+drive+is.");
    assert_eq!(response.status, 303, "{}", response.body);
    assert_eq!(
        response.header("Location"),
        Some(format!("/{}/n/{target}#p3", page.token).as_str())
    );
    let last = place.log().last().unwrap().clone();
    assert_eq!(last["act"], "noted");
    assert_eq!(last["actor"], serde_json::json!({"person": "chuck"}));
    let signed: Value = serde_json::from_str(last["signed"].as_str().unwrap()).unwrap();
    assert_eq!(signed["paragraph"], 3);
    assert_eq!(signed["words"], DRIVE);
    assert!(paragraph(&page.page(&target).body, 3).contains(DRIVE));
}

#[test]
fn a_form_from_another_site() {
    let place = Place::new();
    let v1 = one_narrative(&place, &[]);
    let page = served(&place, &[NAME]);
    let before = place.log();
    let target = format!("{NAME}@{v1}");
    let response = page.post(&target, "https://example.com", "paragraph=3&words=Hello");
    assert_eq!(response.status, 403);
    assert_eq!(place.log(), before);
}

/// The value of the attribute `name` of the first element of `html` that
/// holds it.
fn attribute<'a>(html: &'a str, name: &str) -> &'a str {
    let start = html
        .find(&format!(" {name}=\""))
        .unwrap_or_else(|| panic!("no {name}: {html}"))
        + name.len()
        + 3;
    let rest = &html[start..];
    &rest[..rest.find('"').unwrap()]
}

/// The `Origin` that a browser sends with a form that a page of `own`
/// posts to `own`, under the page's referrer policy `policy` (Fetch,
/// "append a request `Origin` header"): `null` under `no-referrer`, and
/// `own` under each other policy, as `own` is `http://127.0.0.1:<port>`,
/// an origin that the browser trusts.
fn origin_of_the_form(policy: &str, own: &str) -> String {
    match policy {
        "no-referrer" => "null".to_owned(),
        _ => own.to_owned(),
    }
}

/// narrative-page, "A note sent from the page's form".
#[test]
fn a_note_sent_from_the_pages_form() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let page = served(&place, &[NAME]);
    let shown = page.page(NAME);
    let policy = shown.header("Referrer-Policy").unwrap().to_owned();
    let html = shown.body;
    let meta = &html[html.find("<meta name=\"referrer\"").unwrap()..];
    assert_eq!(attribute(meta, "content"), policy, "{html}");
    let form = paragraph(&html, 3);
    let action = attribute(&form, "action").to_owned();
    assert_eq!(
        attribute(&form[form.find("name=\"paragraph\"").unwrap()..], "value"),
        "3"
    );
    let origin = origin_of_the_form(&policy, &format!("http://{}", page.host()));
    let response = send(
        page.port,
        "POST",
        &action,
        &[("Host", &page.host()), ("Origin", &origin)],
        "paragraph=3&words=Say+what+a+drive+is.",
    );
    assert_eq!(response.status, 303, "{policy}: {}", response.body);
    let last = place.log().last().unwrap().clone();
    assert_eq!(last["act"], "noted");
    let location = response.header("Location").unwrap();
    let (path, at) = location.split_once('#').unwrap();
    assert_eq!(at, "p3");
    assert!(paragraph(&page.get(path).body, 3).contains(DRIVE));
}

/// narrative-page, "A form of another site with no referrer".
#[test]
fn a_form_of_another_site_with_no_referrer() {
    let place = Place::new();
    let v1 = one_narrative(&place, &[]);
    let page = served(&place, &[NAME]);
    let before = place.log();
    let target = format!("{NAME}@{v1}");
    for origin in ["null", "http://localhost", "http://127.0.0.1"] {
        let response = page.post(&target, origin, "paragraph=3&words=Hello");
        assert_eq!(response.status, 403, "{origin}");
    }
    assert_eq!(place.log(), before);
}

#[test]
fn two_persons_and_no_person() {
    let place = Place::new();
    let ada = place.made();
    place.registered(&ada, "bo", 2);
    place.narrative(NAME, TITLE, NARRATIVE, &[]);
    let page = served(&place, &[NAME]);
    let body = page.page(NAME).body;
    assert!(!body.contains("<form"), "{body}");
    assert!(
        body.contains(&format!("boom page {NAME} --person")),
        "{body}"
    );
    let with_person = served(&place, &[NAME, "--person", "bo"]);
    assert!(with_person.page(NAME).body.contains("<form"));
}

#[test]
fn the_design_book_served() {
    let place = Place::new();
    book_store(&place);
    let page = served(&place, &[]);
    assert!(
        page.url.ends_with(&format!("/{}/", page.token)),
        "{}",
        page.url
    );
    let response = page.get(&format!("/{}/", page.token));
    assert_eq!(response.status, 200, "{}", response.body);
    assert!(
        response.body.contains("Part I. The system"),
        "{}",
        response.body
    );
}

#[test]
fn a_view_served() {
    let place = Place::new();
    book_store(&place);
    let page = served(&place, &["--view", BOOK]);
    assert!(page.url.ends_with(&format!("/v/{BOOK}")), "{}", page.url);
    let response = page.get(&format!("/{}/v/{BOOK}", page.token));
    assert_eq!(response.status, 200, "{}", response.body);
    assert!(
        response
            .body
            .contains("<h1>The design book of Boomdrive</h1>"),
        "{}",
        response.body
    );
}

#[test]
fn a_view_that_is_not_an_outline() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let outcome = place.run(&["page", "--view", NAME, "--no-open"]);
    assert_eq!(outcome.code, 2, "{outcome:?}");
    assert!(
        outcome.stderr.contains("--view names an outline"),
        "{outcome:?}"
    );
    assert!(outcome.stdout.is_empty(), "{outcome:?}");
}
