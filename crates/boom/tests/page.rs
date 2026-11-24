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
    assert!(!body.contains("<form method=\"post\""), "{body}");
    assert!(
        body.contains(&format!("boom page {NAME} --person")),
        "{body}"
    );
    let with_person = served(&place, &[NAME, "--person", "bo"]);
    assert!(with_person
        .page(NAME)
        .body
        .contains("<form method=\"post\""));
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

// The shell, the book's sidebar and the search (narrative-page, "One
// shell for each view" to "The shell runs no script").

/// The part of `html` from `start` to the next `end`.
fn between(html: &str, start: &str, end: &str) -> String {
    let at = html
        .find(start)
        .unwrap_or_else(|| panic!("no {start}: {html}"));
    let rest = &html[at..];
    rest[..rest.find(end).unwrap()].to_owned()
}

/// The bar of a page.
fn bar(html: &str) -> String {
    between(html, "<header class=\"bar\">", "</header>")
}

/// The sidebar of a page.
fn side(html: &str) -> String {
    between(html, "<nav class=\"side\"", "</nav>")
}

/// The HTML of the page of the search `query`, as the function gives it.
fn search_html(place: &Place, query: &str) -> String {
    let context = Context {
        base: "/t".to_owned(),
        person: None,
    };
    Store::open(&place.store)
        .unwrap()
        .read(|conn| boom::page::search_page(conn, query, &context))
        .unwrap()
}

/// The number of the log's last entry.
fn last_entry(place: &Place) -> u64 {
    place.log().len() as u64
}

/// The item of the claim `name` in `html`, a page of a search.
fn found_claim(html: &str, name: &str) -> String {
    between(html, &format!("<li data-claim=\"{name}\">"), "</li>")
}

#[test]
fn the_bar_of_a_narratives_page() {
    let place = Place::new();
    book_store(&place);
    let html = html(&place, NAME, None, None);
    let bar = bar(&html);
    assert!(
        bar.contains("<a href=\"/t/\" aria-current=\"page\">The book</a>"),
        "{bar}"
    );
    assert!(
        bar.contains("<a href=\"/t/claims/\">The claims</a>"),
        "{bar}"
    );
    assert!(
        bar.contains("<form class=\"search\" role=\"search\" method=\"get\" action=\"/t/search\">"),
        "{bar}"
    );
    assert!(bar.contains("name=\"q\""), "{bar}");
    assert!(
        bar.contains(&format!("entry {}", last_entry(&place))),
        "{bar}"
    );
}

#[test]
fn the_bar_of_a_search() {
    let place = Place::new();
    book_store(&place);
    let html = search_html(&place, "bolt");
    let bar = bar(&html);
    assert!(bar.contains("href=\"/t/\""), "{bar}");
    assert!(bar.contains("href=\"/t/claims/\""), "{bar}");
    assert!(!bar.contains("aria-current"), "{bar}");
    assert!(bar.contains("value=\"bolt\""), "{bar}");
}

#[test]
fn the_order_of_the_views() {
    let place = Place::new();
    book_store(&place);
    let html = book_html(&place, None);
    let views = between(&bar(&html), "<nav class=\"views\"", "</nav>");
    let book = views.find(">The book<").unwrap();
    assert_eq!(views.find("<a "), views[..book].rfind("<a "), "{views}");
    assert!(book < views.find(">The claims<").unwrap(), "{views}");
}

#[test]
fn the_sidebar_of_a_narratives_page() {
    let place = Place::new();
    book_store(&place);
    let html = html(&place, NAME, None, None);
    let side = side(&html);
    let part_one = side
        .find("<p class=\"part\">Part I. The system</p>")
        .unwrap();
    let log = side.find(&format!("href=\"/t/n/{LOG}\"")).unwrap();
    let part_two = side
        .find("<p class=\"part\">Part II. The repositories</p>")
        .unwrap();
    let chapter = side.find("<p class=\"chapter\">boom</p>").unwrap();
    let end_to_end = side.find(&format!("href=\"/t/n/{NAME}\"")).unwrap();
    assert!(
        part_one < log && log < part_two && part_two < chapter && chapter < end_to_end,
        "{side}"
    );
    assert!(side.contains(TITLE), "{side}");
    assert!(side.contains("The drive&#39;s log"), "{side}");
}

#[test]
fn a_section_of_a_context() {
    let place = Place::new();
    book_store(&place);
    place.json(&[
        "propose",
        "context",
        "sluice-host-process",
        "The host process of the queue.",
        "--path",
        "sluice:crates/sluice/src/host/**",
        "--provenance",
        "run-1",
    ]);
    place.narrative(
        "the-merge-queue",
        "The merge queue",
        "A queue.\n",
        &["--context", "sluice-host-process"],
    );
    let html = html(&place, NAME, None, None);
    let side = side(&html);
    let chapter = side.find("<p class=\"chapter\">sluice</p>").unwrap();
    let section = side
        .find("<p class=\"section\">sluice-host-process</p>")
        .unwrap();
    let queue = side.find("href=\"/t/n/the-merge-queue\"").unwrap();
    assert!(chapter < section && section < queue, "{side}");
}

#[test]
fn narratives_only() {
    let place = Place::new();
    book_store(&place);
    let html = book_html(&place, None);
    assert!(!side(&html).contains(STATUS), "{html}");
    assert!(!html.contains(STATUS), "{html}");
}

#[test]
fn the_current_narrative() {
    let place = Place::new();
    book_store(&place);
    let html = html(&place, NAME, None, None);
    let side = side(&html);
    assert!(
        side.contains(&format!("href=\"/t/n/{NAME}\" aria-current=\"page\"")),
        "{side}"
    );
    assert_eq!(side.matches("aria-current").count(), 1, "{side}");
}

#[test]
fn the_sidebar_of_the_book_page() {
    let place = Place::new();
    book_store(&place);
    let html = book_html(&place, None);
    let side = side(&html);
    for name in [LOG, NAME, RECORDS] {
        assert!(side.contains(&format!("href=\"/t/n/{name}\"")), "{side}");
    }
    assert!(!side.contains("aria-current"), "{side}");
}

#[test]
fn a_narrative_that_the_outline_does_not_hold() {
    let place = Place::new();
    book_store(&place);
    place.narrative(
        "boom-for-a-reader",
        "Boom, for a reader",
        "A short one.\n",
        &["--repository", "boom"],
    );
    let html = html(&place, NAME, None, None);
    let side = side(&html);
    let chapter = &side[side.find("<p class=\"chapter\">boom</p>").unwrap()..];
    let last = chapter.rfind("<li>").unwrap();
    let item = &chapter[last..];
    assert!(
        item.starts_with("<li><a href=\"/t/n/boom-for-a-reader\">"),
        "{chapter}"
    );
    assert!(item.contains("not in the outline"), "{item}");
    let held = between(chapter, &format!("<li><a href=\"/t/n/{RECORDS}\""), "</li>");
    assert!(!held.contains("not in the outline"), "{held}");
}

#[test]
fn the_sidebar_of_a_versions_page() {
    let place = Place::new();
    let v1 = one_narrative(&place, &[]);
    let v2 = version(&place.narrative(
        NAME,
        TITLE,
        &format!("{NARRATIVE}\nOne more.\n"),
        &["--amends", &v1],
    ));
    assert_ne!(v1, v2);
    let html = html(&place, NAME, Some(&v1), None);
    assert!(
        side(&html).contains(&format!("href=\"/t/n/{NAME}\" aria-current=\"page\"")),
        "{html}"
    );
}

#[test]
fn a_claim_found() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let status = place.show(STATUS)["accepted"].as_str().unwrap().to_owned();
    let html = search_html(&place, "status events");
    let item = found_claim(&html, STATUS);
    assert!(
        item.contains(&format!(
            "invariant <a href=\"/t/claims/{STATUS}@{status}\"><code>{STATUS}@{status}</code></a>"
        )),
        "{item}"
    );
    assert!(item.contains("accepted"), "{item}");
    assert!(item.contains("A record never holds its status."), "{item}");
}

#[test]
fn the_forms_of_a_word() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let searched = place.json(&["search", "RECORDS", "--all"]);
    let names: Vec<&str> = searched["claims"]
        .as_array()
        .unwrap()
        .iter()
        .map(|claim| claim["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, [STATUS]);
    let html = search_html(&place, "RECORDS");
    assert!(
        html.contains(&format!("<li data-claim=\"{STATUS}\">")),
        "{html}"
    );
}

#[test]
fn no_word_to_search() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let page = served(&place, &[NAME]);
    let response = page.get(&format!("/{}/search?q=%2A%2A%2A", page.token));
    assert_eq!(response.status, 200, "{}", response.body);
    assert!(
        response
            .body
            .contains("boom search needs a word holding a letter or a digit"),
        "{}",
        response.body
    );
    assert!(!response.body.contains("data-claim"), "{}", response.body);
    assert!(!response.body.contains("<ul class=\"list found\">"));
}

#[test]
fn a_search_with_no_token() {
    let place = Place::new();
    book_store(&place);
    let page = served(&place, &[]);
    for path in [
        "/search?q=bolt".to_owned(),
        format!("/{}/search?q=bolt", "0".repeat(32)),
    ] {
        let response = page.get(&path);
        assert_eq!(response.status, 404, "{path}");
        assert!(!response.body.contains(NAME), "{path}: {}", response.body);
        assert!(!response.body.contains("Part I"), "{path}");
    }
}

#[test]
fn a_word_in_a_narratives_paragraph() {
    let place = Place::new();
    place.made_by("chuck");
    place.narrative(
        NAME,
        TITLE,
        "# Boom\n\nOne.\n\nTwo.\n\nThe views replay the log.\n\nA replay again.\n",
        &["--repository", "boom"],
    );
    let html = search_html(&place, "replay");
    let item = item(&html, NAME);
    assert!(item.contains(&format!("href=\"/t/n/{NAME}#p4\"")), "{item}");
    assert!(item.contains("The views replay the log."), "{item}");
    assert!(!item.contains("A replay again."), "{item}");
}

#[test]
fn a_narrative_that_is_proposed() {
    let place = Place::new();
    place.made_by("chuck");
    place.narrative(
        RECORDS,
        "Boom's change record",
        "A record.\n\nA bolt lands.\n",
        &["--repository", "boom"],
    );
    let html = search_html(&place, "bolt");
    let item = item(&html, RECORDS);
    assert!(
        item.contains(&format!("href=\"/t/n/{RECORDS}#p2\"")),
        "{item}"
    );
    assert!(item.contains("proposed"), "{item}");
}

#[test]
fn words_in_two_paragraphs() {
    let place = Place::new();
    place.made_by("chuck");
    place.narrative(
        RECORDS,
        "Boom's change record",
        "A record.\n\nA bolt waits.\n\nThree.\n\nFour.\n\nThe landing comes.\n",
        &["--repository", "boom"],
    );
    let html = search_html(&place, "bolt landing");
    let item = item(&html, RECORDS);
    assert!(item.contains(&format!("href=\"/t/n/{RECORDS}\"")), "{item}");
    assert!(!item.contains("#p"), "{item}");
}

#[test]
fn an_included_claims_link() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let status = place.show(STATUS)["accepted"].as_str().unwrap().to_owned();
    let html = html(&place, NAME, None, None);
    assert!(
        paragraph(&html, 3).contains(&format!(
            "<a href=\"/t/claims/{STATUS}@{status}\"><code>{STATUS}@{status}</code></a>"
        )),
        "{html}"
    );
}

#[test]
fn an_included_claim_out_of_date() {
    let place = Place::new();
    let v1 = one_narrative(&place, &[]);
    let status = place.show(STATUS)["accepted"].as_str().unwrap().to_owned();
    let chuck = place.key("chuck", 7);
    let newer = place.amended(STATUS, "A record never holds its own status.", &status);
    place.accepted(STATUS, &newer, &chuck);
    let html = html(&place, NAME, Some(&v1), None);
    let third = paragraph(&html, 3);
    assert!(
        third.contains(&format!("href=\"/t/claims/{STATUS}@{status}\"")),
        "{third}"
    );
    assert!(
        third.contains(&format!("out of date: accepted now {newer}")),
        "{third}"
    );
}

/// Asserts that a response holds no script and no URL of another host,
/// and gives the policy header.
fn runs_no_script(response: &Response) {
    assert_eq!(response.status, 200, "{}", response.body);
    assert!(
        !response.body.to_lowercase().contains("<script"),
        "{}",
        response.body
    );
    assert!(!response.body.contains("//"), "{}", response.body);
    assert_eq!(response.header("Content-Security-Policy"), Some(POLICY));
}

#[test]
fn the_page_of_a_search() {
    let place = Place::new();
    book_store(&place);
    let page = served(&place, &[]);
    runs_no_script(&page.get(&format!("/{}/search?q=bolt", page.token)));
}

#[test]
fn the_book_page() {
    let place = Place::new();
    book_store(&place);
    let page = served(&place, &[]);
    runs_no_script(&page.get(&format!("/{}/", page.token)));
}

// The views of the claims (claims-view).

/// Proposes a claim of `kind` named `name` with `extra`, a decision with
/// a body, giving its version.
fn claim_of(place: &Place, kind: &str, name: &str, words: &str, extra: &[&str]) -> String {
    let body;
    let mut args = vec!["propose", kind, name, words];
    if kind == "decision" && !extra.contains(&"--body") {
        body = place.decision_body();
        args.extend(["--body", body.as_str()]);
    }
    args.extend(extra);
    args.extend(["--provenance", "run-1"]);
    version(&place.json(&args))
}

/// Proposes and accepts a claim of `kind`, giving its version.
fn accepted_of(
    place: &Place,
    key: &str,
    kind: &str,
    name: &str,
    words: &str,
    extra: &[&str],
) -> String {
    let v = claim_of(place, kind, name, words, extra);
    place.accepted(name, &v, key);
    v
}

/// The context `sluice-host-process` with `glob`, accepted.
fn host_context(place: &Place, key: &str, glob: &str) -> String {
    accepted_of(
        place,
        key,
        "context",
        HOST,
        "The host process of the queue.",
        &["--path", glob],
    )
}

const HOST: &str = "sluice-host-process";
const WORDS: &str = "the-status-words";
const BINDS: &str = "only-an-accepted-version-binds-work";

impl Served {
    /// The view or page at `path` under the token.
    fn at(&self, path: &str) -> Response {
        self.get(&format!("/{}/{path}", self.token))
    }

    fn base(&self) -> String {
        format!("/{}", self.token)
    }
}

/// The part of `html` from the item of the claim `name` to its end.
fn claim_item(html: &str, name: &str) -> String {
    let start = html
        .find(&format!("data-claim=\"{name}\""))
        .unwrap_or_else(|| panic!("no item {name}: {html}"));
    let rest = &html[start..];
    rest[..rest.find("</div>\n").unwrap_or(rest.len())].to_owned()
}

/// The part of `html` under "Proposed, binding nothing", or nothing.
fn waiting(html: &str) -> String {
    html.find("<section class=\"waiting\"")
        .map(|start| {
            let rest = &html[start..];
            rest[..rest.find("</section>").unwrap()].to_owned()
        })
        .unwrap_or_default()
}

/// The part of `html` after "Proposed, binding nothing".
fn levels_part(html: &str) -> String {
    let main = &html[html.find("<main>").unwrap()..];
    match main.find("</section>") {
        Some(end) => main[end..].to_owned(),
        None => main.to_owned(),
    }
}

#[test]
fn the_bar_of_the_rules_view() {
    let place = Place::new();
    place.made_by("chuck");
    let page = served(&place, &[]);
    let html = page.at("rules").body;
    let bar = bar(&html);
    let base = page.base();
    for (words, path) in [
        ("The book", "/"),
        ("The rules", "/rules"),
        ("The decisions", "/decisions"),
        ("The terms", "/terms"),
        ("The claims", "/claims/"),
    ] {
        assert!(
            bar.contains(&format!("href=\"{base}{path}\"")),
            "{words}: {bar}"
        );
    }
    assert!(
        bar.contains(&format!(
            "<a href=\"{base}/rules\" aria-current=\"page\">The rules</a>"
        )),
        "{bar}"
    );
    assert_eq!(bar.matches("aria-current").count(), 1, "{bar}");
}

#[test]
fn the_sidebar_of_a_view() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    for name in ["a-one", "a-two", "a-three", "a-four"] {
        accepted_of(
            &place,
            &chuck,
            "invariant",
            name,
            "It holds.",
            &["--repository", "boom"],
        );
    }
    accepted_of(
        &place,
        &chuck,
        "assumption",
        "a-five",
        "It is so.",
        &["--repository", "boom"],
    );
    let page = served(&place, &[]);
    let side = side(&page.at("rules").body);
    assert!(
        side.contains(&format!(
            "<a href=\"{}/rules#boom\">boom <span class=\"count\">5</span>",
            page.base()
        )),
        "{side}"
    );
}

#[test]
fn a_rule_of_a_repository() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    let body = place.file("status.md", "Breaks it: a status field.\n");
    let v1 = accepted_of(
        &place,
        &chuck,
        "invariant",
        STATUS,
        "A record never holds its status.",
        &["--repository", "boom", "--body", &body],
    );
    let page = served(&place, &[]);
    let html = page.at("rules").body;
    let chapter = &html[html.find("<h3 id=\"boom\">").expect("the chapter boom")..];
    let item = claim_item(chapter, STATUS);
    assert!(
        item.contains(&format!("href=\"{}/claims/{STATUS}@{v1}\"", page.base())),
        "{item}"
    );
    assert!(item.contains("A record never holds its status."), "{item}");
    assert!(
        item.contains("<details><summary>details</summary>"),
        "{item}"
    );
    assert!(item.contains("Breaks it: a status field."), "{item}");
}

#[test]
fn a_rule_of_a_context() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    host_context(&place, &chuck, "sluice:crates/sluice/src/host/**");
    accepted_of(
        &place,
        &chuck,
        "invariant",
        "one-host-process",
        "One process.",
        &["--context", HOST],
    );
    let page = served(&place, &[]);
    let html = page.at("rules").body;
    let chapter = html.find("<h3 id=\"sluice\">").expect("the chapter sluice");
    let section = html
        .find(&format!("<h4 id=\"sluice/{HOST}\""))
        .expect("the section");
    let item = html.find("data-claim=\"one-host-process\"").unwrap();
    assert!(chapter < section && section < item, "{html}");
}

#[test]
fn a_rule_of_two_repositories() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    accepted_of(
        &place,
        &chuck,
        "invariant",
        "both-hold",
        "Both hold it.",
        &["--repository", "boom", "--repository", "sluice"],
    );
    let page = served(&place, &[]);
    let html = page.at("rules").body;
    let boom = html.find("<h3 id=\"boom\">").unwrap();
    let sluice = html.find("<h3 id=\"sluice\">").unwrap();
    let main = levels_part(&html);
    assert_eq!(
        main.matches("data-claim=\"both-hold\"").count(),
        2,
        "{html}"
    );
    let first = html.find("data-claim=\"both-hold\"").unwrap();
    let last = html.rfind("data-claim=\"both-hold\"").unwrap();
    assert!(boom < first && first < sluice && sluice < last, "{html}");
}

#[test]
fn not_a_rule() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    accepted_of(
        &place,
        &chuck,
        "decision",
        WORDS,
        "The status words.",
        &["--repository", "boom"],
    );
    accepted_of(
        &place,
        &chuck,
        "term",
        "bolt",
        "A batch of units.",
        &["--repository", "boom"],
    );
    let page = served(&place, &[]);
    let html = page.at("rules").body;
    assert!(!html.contains(WORDS), "{html}");
    assert!(!html.contains("data-claim=\"bolt\""), "{html}");
}

#[test]
fn the_rules_of_boom() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    accepted_of(
        &place,
        &chuck,
        "invariant",
        "of-the-system",
        "It holds.",
        &["--system"],
    );
    accepted_of(
        &place,
        &chuck,
        "invariant",
        "of-boom",
        "It holds.",
        &["--repository", "boom"],
    );
    accepted_of(
        &place,
        &chuck,
        "assumption",
        "boom-assumes",
        "It is so.",
        &["--repository", "boom"],
    );
    accepted_of(
        &place,
        &chuck,
        "invariant",
        "of-sluice",
        "It holds.",
        &["--repository", "sluice"],
    );
    let query = place.json(&["query", "--repository", "boom"]);
    let mut expected: Vec<String> = query["claims"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|claim| matches!(claim["kind"].as_str(), Some("invariant" | "assumption")))
        .map(|claim| claim["name"].as_str().unwrap().to_owned())
        .collect();
    expected.sort();
    assert_eq!(expected, ["boom-assumes", "of-boom", "of-the-system"]);
    let page = served(&place, &[]);
    let html = page.at("rules?repository=boom").body;
    let main = levels_part(&html);
    let mut given: Vec<String> = main
        .split("data-claim=\"")
        .skip(1)
        .map(|rest| rest[..rest.find('"').unwrap()].to_owned())
        .collect();
    given.sort();
    given.dedup();
    assert_eq!(given, expected, "{html}");
}

#[test]
fn decisions_in_the_order_accepted() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    accepted_of(
        &place,
        &chuck,
        "decision",
        WORDS,
        "The status words.",
        &["--repository", "boom"],
    );
    accepted_of(
        &place,
        &chuck,
        "decision",
        "a-term-binds-one-repository",
        "A term binds one repository.",
        &["--repository", "boom"],
    );
    let page = served(&place, &[]);
    let html = page.at("decisions").body;
    let chapter = &html[html.find("<h3 id=\"boom\">").unwrap()..];
    let words = chapter.find(&format!("data-claim=\"{WORDS}\"")).unwrap();
    let term = chapter
        .find("data-claim=\"a-term-binds-one-repository\"")
        .unwrap();
    assert!(words < term, "{chapter}");
    assert!(
        claim_item(chapter, WORDS).contains("accepted 20"),
        "{chapter}"
    );
}

#[test]
fn a_newer_version_proposed() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    let v1 = accepted_of(
        &place,
        &chuck,
        "decision",
        WORDS,
        "The status words.",
        &["--repository", "boom"],
    );
    claim_of(
        &place,
        "decision",
        WORDS,
        "The four status words.",
        &["--amends", &v1],
    );
    let page = served(&place, &[]);
    let html = page.at("decisions").body;
    let item = claim_item(&levels_part(&html), WORDS);
    assert!(item.contains(&format!("{WORDS}@{v1}")), "{item}");
    assert!(item.contains("a newer version is proposed"), "{item}");
}

#[test]
fn a_term_in_the_terms_view() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    accepted_of(
        &place,
        &chuck,
        "term",
        "bolt",
        "A set of units that land together.",
        &["--repository", "sluice", "--replaces", "batch"],
    );
    let page = served(&place, &[]);
    let html = page.at("terms").body;
    let chapter = &html[html.find("<h3 id=\"sluice\">").unwrap()..];
    let item = claim_item(chapter, "bolt");
    assert!(
        item.contains("A set of units that land together."),
        "{item}"
    );
    assert!(!html.contains("batch"), "{html}");
}

#[test]
fn the_page_of_a_term() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    accepted_of(
        &place,
        &chuck,
        "term",
        "bolt",
        "A set of units that land together.",
        &["--repository", "sluice", "--replaces", "batch"],
    );
    let page = served(&place, &[]);
    let html = page.at("claims/bolt").body;
    assert!(html.contains("It replaces <code>batch</code>."), "{html}");
}

#[test]
fn every_kind() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    let boom = ["--repository", "boom"];
    accepted_of(
        &place,
        &chuck,
        "context",
        "boom-store",
        "The store.",
        &["--path", "boom:crates/boom/src/store.rs"],
    );
    accepted_of(&place, &chuck, "term", "e-term", "A word.", &boom);
    accepted_of(
        &place,
        &chuck,
        "assumption",
        "d-assumption",
        "It is so.",
        &boom,
    );
    accepted_of(&place, &chuck, "decision", "c-decision", "A choice.", &boom);
    accepted_of(
        &place,
        &chuck,
        "invariant",
        "b-invariant",
        "It holds.",
        &boom,
    );
    place.narrative(NAME, TITLE, NARRATIVE, &boom);
    let page = served(&place, &[]);
    let html = page.at("claims/").body;
    let chapter = &html[html.find("<h3 id=\"boom\">").unwrap()..];
    let chapter = &chapter[..chapter.find("<h4").unwrap_or(chapter.len())];
    let places: Vec<usize> = [
        "b-invariant",
        "c-decision",
        "d-assumption",
        "e-term",
        "boom-store",
    ]
    .iter()
    .map(|name| {
        chapter
            .find(&format!("data-claim=\"{name}\""))
            .unwrap_or_else(|| panic!("no {name}: {chapter}"))
    })
    .collect();
    assert!(places.windows(2).all(|pair| pair[0] < pair[1]), "{chapter}");
    assert!(!html.contains(&format!("data-claim=\"{NAME}\"")), "{html}");
}

#[test]
fn a_contexts_section() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    host_context(&place, &chuck, "sluice:crates/host/**");
    let page = served(&place, &[]);
    let html = page.at("rules").body;
    let section = &html[html.find(&format!("<h4 id=\"sluice/{HOST}\"")).unwrap()..];
    let section = &section[..section.find("</p>").unwrap()];
    assert!(
        section.contains("<code>sluice:crates/host/**</code>"),
        "{section}"
    );
}

#[test]
fn a_contexts_page() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    host_context(&place, &chuck, "sluice:crates/host/**");
    let on = ["--context", HOST];
    accepted_of(&place, &chuck, "invariant", "host-one", "One.", &on);
    accepted_of(&place, &chuck, "invariant", "host-two", "Two.", &on);
    accepted_of(&place, &chuck, "decision", "host-choice", "A choice.", &on);
    let page = served(&place, &[]);
    let html = page.at(&format!("claims/{HOST}")).body;
    assert!(
        html.contains("<code>sluice:crates/host/**</code>"),
        "{html}"
    );
    for name in ["host-one", "host-two", "host-choice"] {
        assert!(
            html.contains(&format!("href=\"{}/claims/{name}@", page.base())),
            "{name}: {html}"
        );
    }
}

#[test]
fn a_proposed_decision() {
    let place = Place::new();
    place.made_by("chuck");
    let v1 = claim_of(
        &place,
        "decision",
        BINDS,
        "Only an accepted version binds work.",
        &["--system"],
    );
    let page = served(&place, &[]);
    let html = page.at("decisions").body;
    let waiting = waiting(&html);
    let group = waiting
        .find("<h3 class=\"group\">system</h3>")
        .expect("the group of the whole system");
    let item = claim_item(&waiting[group..], BINDS);
    assert!(
        item.contains("<span class=\"mark\">proposed</span>"),
        "{item}"
    );
    assert!(
        item.contains(&format!("boom accept {BINDS}@{v1}")),
        "{item}"
    );
    assert!(
        html.find("Proposed, binding nothing").unwrap() < html.find("<h2 id=\"system\">").unwrap(),
        "{html}"
    );
}

#[test]
fn one_command_for_every_proposal() {
    let place = Place::new();
    place.made_by("chuck");
    let v1 = claim_of(
        &place,
        "decision",
        BINDS,
        "Only an accepted version binds work.",
        &["--system"],
    );
    let v2 = claim_of(
        &place,
        "decision",
        WORDS,
        "The status words.",
        &["--repository", "boom"],
    );
    let page = served(&place, &[]);
    let waiting = waiting(&page.at("decisions").body);
    let head = &waiting[..waiting.find("<h3").unwrap()];
    let command = between(head, "boom accept", "</pre>");
    assert!(command.contains(&format!("{BINDS}@{v1}")), "{command}");
    assert!(command.contains(&format!("{WORDS}@{v2}")), "{command}");
}

#[test]
fn a_rejected_version() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    let v1 = claim_of(
        &place,
        "decision",
        WORDS,
        "The status words.",
        &["--repository", "boom"],
    );
    place.json(&[
        "reject",
        &format!("{WORDS}@{v1}"),
        "--reason",
        "Not yet.",
        "--key",
        &chuck,
    ]);
    let page = served(&place, &[]);
    for view in ["decisions", "claims/", "rules"] {
        let html = page.at(view).body;
        assert!(
            !html.contains(&format!("data-claim=\"{WORDS}\"")),
            "{view}: {html}"
        );
    }
}

#[test]
fn nothing_proposed() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    accepted_of(
        &place,
        &chuck,
        "decision",
        WORDS,
        "The status words.",
        &["--repository", "boom"],
    );
    claim_of(
        &place,
        "invariant",
        "a-rule",
        "It holds.",
        &["--repository", "boom"],
    );
    let page = served(&place, &[]);
    let html = page.at("decisions").body;
    assert!(!html.contains("Proposed, binding nothing"), "{html}");
}

#[test]
fn the_page_of_a_proposed_decision() {
    let place = Place::new();
    place.made_by("chuck");
    let v1 = claim_of(
        &place,
        "decision",
        BINDS,
        "Only an accepted version binds work.",
        &["--system"],
    );
    let page = served(&place, &[]);
    let html = page.at(&format!("claims/{BINDS}@{v1}")).body;
    assert!(html.contains("proposed, binds nothing"), "{html}");
    assert!(
        html.contains(&format!("boom accept {BINDS}@{v1}")),
        "{html}"
    );
    assert!(
        html.contains(&format!("boom reject {BINDS}@{v1} --reason")),
        "{html}"
    );
    assert!(!html.contains("method=\"post\""), "{html}");
}

#[test]
fn the_page_of_an_invariant() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let v1 = place.show(STATUS)["accepted"].as_str().unwrap().to_owned();
    let page = served(&place, &[]);
    let response = page.at(&format!("claims/{STATUS}"));
    assert_eq!(response.status, 200, "{}", response.body);
    let html = response.body;
    assert!(
        html.contains(&format!("invariant <code>{STATUS}@{v1}</code>")),
        "{html}"
    );
    assert!(
        html.contains("<span class=\"status\">accepted</span>"),
        "{html}"
    );
    assert!(html.contains("A record never holds its status."), "{html}");
    assert!(
        html.contains(&format!("href=\"{}/n/{NAME}#p3\"", page.base())),
        "{html}"
    );
    assert!(
        bar(&html).contains(&format!(
            "<a href=\"{}/claims/\" aria-current=\"page\">",
            page.base()
        )),
        "{html}"
    );
}

#[test]
fn no_such_claim() {
    let place = Place::new();
    place.made_by("chuck");
    let page = served(&place, &[]);
    assert_eq!(page.at("claims/no-such-claim").status, 404);
}

#[test]
fn a_superseded_and_a_rejected_version() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    let v1 = accepted_of(
        &place,
        &chuck,
        "invariant",
        STATUS,
        "A record never holds its status.",
        &["--repository", "boom"],
    );
    let v2 = place.amended(STATUS, "A record never holds its own status.", &v1);
    place.accepted(STATUS, &v2, &chuck);
    let v3 = place.amended(STATUS, "A record holds its standing.", &v2);
    place.json(&[
        "reject",
        &format!("{STATUS}@{v3}"),
        "--reason",
        "says standing",
        "--key",
        &chuck,
    ]);
    let page = served(&place, &[]);
    let html = page.at(&format!("claims/{STATUS}")).body;
    let versions = between(&html, "<p class=\"versions\">", "</p>");
    assert!(versions.contains(&v2), "{versions}");
    assert!(
        !versions.contains(&v1) && !versions.contains(&v3),
        "{versions}"
    );
    let history = between(&html, "<details class=\"history\">", "</details>");
    let first = between(&history, &format!("<li data-version=\"{v1}\">"), "</li>");
    assert!(first.contains("superseded"), "{first}");
    let third = between(&history, &format!("<li data-version=\"{v3}\">"), "</li>");
    assert!(third.contains("rejected"), "{third}");
    assert!(third.contains("says standing"), "{third}");
}

#[test]
fn a_claim_named_as_a_source() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    let v1 = accepted_of(
        &place,
        &chuck,
        "invariant",
        WORDS,
        "The status words.",
        &["--repository", "boom"],
    );
    let meant = format!("{WORDS}@{v1}");
    claim_of(
        &place,
        "decision",
        BINDS,
        "Only an accepted version binds work.",
        &["--system", "--source", &meant],
    );
    let v2 = place.amended(WORDS, "The four status words.", &v1);
    place.accepted(WORDS, &v2, &chuck);
    let page = served(&place, &[]);
    let html = page.at(&format!("claims/{WORDS}")).body;
    let named = between(&html, "<ul class=\"edges\" id=\"named-by\">", "</ul>");
    let item = between(&named, &format!("<li data-claim=\"{BINDS}\">"), "</li>");
    assert!(item.contains(&format!("<code>@{v1}</code>")), "{item}");
    assert!(
        item.contains(&format!("out of date: accepted now {v2}")),
        "{item}"
    );
}

#[test]
fn a_kept_file() {
    let place = Place::new();
    place.made_by("chuck");
    let brief = place.file("claim-status-brief.md", "The brief.\n");
    claim_of(
        &place,
        "invariant",
        WORDS,
        "The status words.",
        &["--repository", "boom", "--origin", &brief],
    );
    let fingerprint = boom::sha256_hex(b"The brief.\n");
    let page = served(&place, &[]);
    let html = page.at(&format!("claims/{WORDS}")).body;
    let origin = between(&html, "<h2>Origin</h2>", "</ul>");
    assert!(origin.contains("claim-status-brief.md"), "{origin}");
    assert!(origin.contains(&fingerprint), "{origin}");
    assert!(
        origin.contains(&format!("boom show --artifact {fingerprint}")),
        "{origin}"
    );
    assert!(!origin.contains("href"), "{origin}");
}

#[test]
fn a_note_on_a_paragraph_that_includes_the_claim() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    let events = "events-are-appended-never-changed";
    accepted_of(
        &place,
        &chuck,
        "invariant",
        events,
        "boom appends an event.",
        &["--system"],
    );
    let v1 = version(&place.narrative(
        LOG,
        "The drive's log",
        &format!("# The log\n\nEach entry stays: `{events}`.\n"),
        &["--system", "--includes", events],
    ));
    place.noted(&format!("{LOG}@{v1}"), 2, DRIVE);
    let page = served(&place, &[]);
    let html = page.at(&format!("claims/{events}")).body;
    let narratives = between(&html, "<h2>Narratives that include it</h2>", "</ul>\n");
    let note = between(&narratives, "<li class=\"note-of\">", "</li>");
    assert!(note.contains(DRIVE), "{note}");
    assert!(note.contains(" · open"), "{note}");
    assert!(note.contains("chuck"), "{note}");
    assert!(note.contains(" UTC"), "{note}");
    assert!(
        note.contains(&format!("href=\"{}/n/{LOG}#p2\"", page.base())),
        "{note}"
    );
}

#[test]
fn the_proposed_decisions_of_boom() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    claim_of(
        &place,
        "decision",
        "of-the-system",
        "A choice.",
        &["--system"],
    );
    claim_of(
        &place,
        "decision",
        "of-boom",
        "A choice.",
        &["--repository", "boom"],
    );
    claim_of(
        &place,
        "decision",
        "of-sluice",
        "A choice.",
        &["--repository", "sluice"],
    );
    accepted_of(
        &place,
        &chuck,
        "decision",
        "boom-accepted",
        "A choice.",
        &["--repository", "boom"],
    );
    let page = served(&place, &[]);
    let html = page.at("decisions?status=proposed&repository=boom").body;
    let main = &html[html.find("<main>").unwrap()..];
    assert!(main.contains("data-claim=\"of-the-system\""), "{main}");
    assert!(main.contains("data-claim=\"of-boom\""), "{main}");
    assert!(!main.contains("data-claim=\"of-sluice\""), "{main}");
    assert!(!main.contains("data-claim=\"boom-accepted\""), "{main}");
}

#[test]
fn a_search_in_the_rules_view() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    accepted_of(
        &place,
        &chuck,
        "invariant",
        STATUS,
        "A record never holds its status.",
        &["--repository", "boom"],
    );
    accepted_of(
        &place,
        &chuck,
        "invariant",
        "money-has-currency",
        "Every amount names its currency.",
        &["--repository", "boom"],
    );
    let searched = place.json(&["search", "RECORDS", "--all"]);
    assert_eq!(searched["claims"][0]["name"], STATUS);
    let page = served(&place, &[]);
    let main = levels_part(&page.at("rules?q=RECORDS").body);
    assert!(main.contains(&format!("data-claim=\"{STATUS}\"")), "{main}");
    assert!(
        !main.contains("data-claim=\"money-has-currency\""),
        "{main}"
    );
}

#[test]
fn a_proposed_claim_found() {
    let place = Place::new();
    place.made_by("chuck");
    claim_of(
        &place,
        "decision",
        "a-card-on-the-rail",
        "A waiting proposal is a card on the rail.",
        &["--repository", "boom"],
    );
    claim_of(
        &place,
        "decision",
        "another-choice",
        "Another choice.",
        &["--repository", "boom"],
    );
    let page = served(&place, &[]);
    let html = page.at("decisions?status=both&q=rail").body;
    let item = claim_item(&waiting(&html), "a-card-on-the-rail");
    assert!(
        item.contains("<span class=\"mark\">proposed</span>"),
        "{item}"
    );
    assert!(!html.contains("data-claim=\"another-choice\""), "{html}");
}

#[test]
fn a_kind_that_is_not_one() {
    let place = Place::new();
    place.made_by("chuck");
    let page = served(&place, &[]);
    let response = page.at("claims/?kind=rule");
    assert_eq!(response.status, 400, "{}", response.body);
    for kind in ["invariant", "decision", "assumption", "term", "context"] {
        assert!(response.body.contains(kind), "{kind}: {}", response.body);
    }
}

#[test]
fn a_claim_accepted_while_the_view_is_served() {
    let place = Place::new();
    let chuck = place.made_by("chuck");
    let v1 = claim_of(
        &place,
        "decision",
        WORDS,
        "The status words.",
        &["--repository", "boom"],
    );
    let page = served(&place, &[]);
    let before = page.at("decisions").body;
    assert!(
        waiting(&before).contains(&format!("data-claim=\"{WORDS}\"")),
        "{before}"
    );
    place.accepted(WORDS, &v1, &chuck);
    let after = page.at("decisions").body;
    assert!(waiting(&after).is_empty(), "{after}");
    assert!(
        levels_part(&after).contains(&format!("data-claim=\"{WORDS}\"")),
        "{after}"
    );
}

#[test]
fn the_rules_view() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let page = served(&place, &[]);
    runs_no_script(&page.at("rules"));
}

#[test]
fn a_view_with_no_token() {
    let place = Place::new();
    one_narrative(&place, &[]);
    let page = served(&place, &[]);
    for path in ["/rules", "/claims/", "/decisions"] {
        let response = page.get(path);
        assert_eq!(response.status, 404, "{path}");
        assert!(!response.body.contains(STATUS), "{path}: {}", response.body);
    }
}

#[test]
fn a_filters_value_given_back() {
    let place = Place::new();
    place.made_by("chuck");
    let page = served(&place, &[]);
    let html = page.at("claims/?q=%3Cb%3Ebolt%3C%2Fb%3E").body;
    assert!(html.contains("value=\"&lt;b&gt;bolt&lt;/b&gt;\""), "{html}");
    assert!(!html.contains("<b>"), "{html}");
}
