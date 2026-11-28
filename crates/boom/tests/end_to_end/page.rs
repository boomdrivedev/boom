//! `boom page` through the binary: its line, the page at the URL that
//! the line gives, and the script whose length the response gives
//! (narrative-page). The other tests of the page run in the crate, each
//! request sent to the server's answer, in src/tests/page.rs.

use super::support;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};

use boom::page::POLICY;
use serde_json::Value;
use support::{Place, NARRATIVE};

/// `boom page`'s main path, through the binary: its line, the page at
/// the URL that the line gives, and the script whose length the
/// response gives, never in chunks.
#[test]
fn the_page_served() {
    let _slot = super::slot();
    let place = Place::new();
    let v1 = one_narrative(&place, &[]);
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let mut child = Command::new(env!("CARGO_BIN_EXE_boom"))
        .args(["page", NAME, "--no-open", "--port", &port.to_string()])
        .args(["--store", &place.store_arg()])
        .current_dir(place.dir.path())
        .env_remove("BOOM_REPOSITORY")
        .env("XDG_CONFIG_HOME", place.dir.path().join("config"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    // The child ends when the test ends, passed or failed.
    struct Ends(Child);
    impl Drop for Ends {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let stdout = child.stdout.take().unwrap();
    let _ends = Ends(child);
    let mut line = String::new();
    BufReader::new(stdout).read_line(&mut line).unwrap();
    let prefix = format!("Boom: page of {NAME}@{v1} at http://127.0.0.1:{port}/");
    assert!(line.starts_with(&prefix), "{line:?}");
    let path = &line.trim_end()[prefix.len() - 1..];
    let token = &path[1..path[1..].find('/').unwrap() + 1];
    assert_eq!(token.len(), 32, "{line:?}");
    assert_eq!(&path[token.len() + 1..], format!("/n/{NAME}"), "{line:?}");
    let get = |path: &str| {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).unwrap();
        let split = raw.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        let head = String::from_utf8_lossy(&raw[..split]).into_owned();
        (head, raw[split + 4..].to_vec())
    };
    let (head, body) = get(path);
    assert!(head.starts_with("HTTP/1.1 200"), "{head}");
    assert!(
        head.contains(&format!("Content-Security-Policy: {POLICY}")),
        "{head}"
    );
    assert!(String::from_utf8_lossy(&body).contains(&format!("{NAME}@{v1}")));
    let (head, body) = get(&format!("/{token}/mermaid-11.17.2.min.js"));
    let script = mermaid_file("mermaid.min.js");
    assert!(
        head.contains(&format!("Content-Length: {}", script.len())),
        "{head}"
    );
    assert!(!head.to_lowercase().contains("chunked"), "{head}");
    assert!(body == script);
}

const NAME: &str = "boom-end-to-end";

const TITLE: &str = "Boom, end to end";

const STATUS: &str = "status-read-from-events";

fn version(answer: &Value) -> String {
    answer["version"].as_str().unwrap().to_owned()
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

/// The bytes of a file of Mermaid's in the crate.
fn mermaid_file(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/mermaid")
            .join(name),
    )
    .unwrap()
}
