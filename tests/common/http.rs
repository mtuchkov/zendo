//! A mock API server on `127.0.0.1`: it keeps each request and answers each with the same JSON
//! (docs/zendo/session-names/session-names.md, NAMES-15).

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

/// One request that the mock server got.
#[derive(Clone, Debug)]
pub struct Request {
    pub path: String,
    /// The headers, the names in lower case.
    pub headers: Vec<(String, String)>,
    pub body: serde_json::Value,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }
}

/// The mock server: its base URL and the requests so far.
pub struct MockApi {
    pub url: String,
    pub requests: Arc<Mutex<Vec<Request>>>,
}

/// Start a mock server that answers each request with `answer`.
pub fn mock_api(answer: serde_json::Value) -> MockApi {
    mock_api_status(200, answer)
}

/// Start a mock server that answers each request with the HTTP `status` and `answer`.
pub fn mock_api_status(status: u16, answer: serde_json::Value) -> MockApi {
    mock_api_after(status, answer, std::time::Duration::ZERO)
}

/// Start a mock server that answers each request after `delay` (REKEY-4, REKEY-5).
pub fn mock_api_after(status: u16, answer: serde_json::Value, delay: std::time::Duration) -> MockApi {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(Mutex::new(Vec::new()));
    let seen = requests.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut r = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            r.read_line(&mut line).unwrap();
            let path = line.split_whitespace().nth(1).unwrap_or("").to_string();
            let mut headers = Vec::new();
            loop {
                let mut h = String::new();
                if r.read_line(&mut h).unwrap() == 0 || h.trim().is_empty() {
                    break;
                }
                if let Some((k, v)) = h.split_once(':') {
                    headers.push((k.trim().to_lowercase(), v.trim().to_string()));
                }
            }
            let len = headers.iter().find(|(k, _)| k == "content-length").and_then(|(_, v)| v.parse().ok()).unwrap_or(0);
            let mut body = vec![0; len];
            r.read_exact(&mut body).unwrap();
            let body = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
            seen.lock().unwrap().push(Request { path, headers, body });
            std::thread::sleep(delay);
            let text = answer.to_string();
            let mut s = stream;
            let _ = write!(s, "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{text}", text.len());
        }
    });
    MockApi { url, requests }
}
