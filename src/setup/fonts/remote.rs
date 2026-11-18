//! The files of the font releases: a source of bytes by range, the network through `curl`, and a
//! zip archive read by ranges, so that only the members that zendo needs come over the network.
use std::io;

/// Bytes that can be read by range: a file on a server, or bytes in memory.
pub trait Source {
    fn size(&mut self) -> io::Result<u64>;
    fn range(&mut self, start: u64, len: u64) -> io::Result<Vec<u8>>;
}

/// Opens a source for a URL.
pub trait Net {
    fn open(&mut self, url: &str) -> io::Result<Box<dyn Source>>;
}

/// Bytes in memory.
pub struct Bytes(pub Vec<u8>);

impl Source for Bytes {
    fn size(&mut self) -> io::Result<u64> {
        Ok(self.0.len() as u64)
    }
    fn range(&mut self, start: u64, len: u64) -> io::Result<Vec<u8>> {
        let end = start.checked_add(len).filter(|e| *e <= self.0.len() as u64).ok_or_else(|| bad("a range past the end"))?;
        Ok(self.0[start as usize..end as usize].to_vec())
    }
}

/// The network through `curl`: it follows the redirects of GitHub and uses the proxy and the
/// certificates of the system.
pub struct Curl;

impl Net for Curl {
    fn open(&mut self, url: &str) -> io::Result<Box<dyn Source>> {
        Ok(Box::new(CurlFile { url: url.to_string(), size: None }))
    }
}

struct CurlFile {
    url: String,
    size: Option<u64>,
}

impl CurlFile {
    fn curl(&self, args: &[&str]) -> io::Result<Vec<u8>> {
        let out = std::process::Command::new("curl")
            .args(["-fsSL", "--retry", "2", "--connect-timeout", "20", "--max-time", "600"])
            .args(args)
            .arg(&self.url)
            .stdin(std::process::Stdio::null())
            .output()
            .map_err(|e| if e.kind() == io::ErrorKind::NotFound { bad("curl is not installed") } else { e })?;
        if !out.status.success() {
            return Err(bad(String::from_utf8_lossy(&out.stderr).trim().trim_start_matches("curl: ")));
        }
        Ok(out.stdout)
    }
}

impl Source for CurlFile {
    fn size(&mut self) -> io::Result<u64> {
        if let Some(n) = self.size {
            return Ok(n);
        }
        // The headers of every hop of the redirects; the last `content-range` is of the file.
        let head = String::from_utf8_lossy(&self.curl(&["-r", "0-0", "-o", "/dev/null", "-D", "-"])?).into_owned();
        let n = total(&head).ok_or_else(|| bad("the server does not give the size of the file"))?;
        self.size = Some(n);
        Ok(n)
    }
    fn range(&mut self, start: u64, len: u64) -> io::Result<Vec<u8>> {
        if len == 0 {
            return Ok(Vec::new());
        }
        let bytes = self.curl(&["-r", &format!("{start}-{}", start + len - 1)])?;
        if bytes.len() as u64 != len {
            return Err(bad(format!("got {} bytes of {len}", bytes.len())));
        }
        Ok(bytes)
    }
}

/// The size of the file from the last `content-range: bytes 0-0/<size>` header.
fn total(head: &str) -> Option<u64> {
    head.lines().rfind(|l| l.to_ascii_lowercase().starts_with("content-range:"))?.rsplit('/').next()?.trim().parse().ok()
}

fn bad(s: impl Into<String>) -> io::Error {
    io::Error::other(s.into())
}

/// A zip archive: the central directory, read once, and the source of its members.
pub struct Zip {
    src: Box<dyn Source>,
    entries: Vec<Entry>,
}

struct Entry {
    name: String,
    method: u16,
    size: u64,
    offset: u64,
}

const END: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
const CENTRAL: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
/// The end record and its longest comment.
const TAIL: u64 = 22 + 65_535;

fn u16_at(b: &[u8], i: usize) -> Option<u64> {
    Some(u16::from_le_bytes(b.get(i..i + 2)?.try_into().ok()?) as u64)
}

fn u32_at(b: &[u8], i: usize) -> Option<u64> {
    Some(u32::from_le_bytes(b.get(i..i + 4)?.try_into().ok()?) as u64)
}

impl Zip {
    /// Read the central directory of the archive.
    pub fn open(mut src: Box<dyn Source>) -> io::Result<Zip> {
        let size = src.size()?;
        let tail_len = size.min(TAIL);
        let tail = src.range(size - tail_len, tail_len)?;
        let end = tail.windows(4).rposition(|w| w == END).ok_or_else(|| bad("not a zip archive"))?;
        let (len, at) = u32_at(&tail, end + 12).zip(u32_at(&tail, end + 16)).ok_or_else(|| bad("a broken zip archive"))?;
        let dir = src.range(at, len)?;
        Ok(Zip { src, entries: entries(&dir) })
    }

    /// The bytes of a member.
    pub fn read(&mut self, name: &str) -> io::Result<Vec<u8>> {
        let e = self.entries.iter().find(|e| e.name == name).ok_or_else(|| bad(format!("{name} is not in the archive")))?;
        let (method, size, offset) = (e.method, e.size, e.offset);
        let head = self.src.range(offset, 30)?;
        let skip = u16_at(&head, 26).zip(u16_at(&head, 28)).map(|(n, x)| 30 + n + x).ok_or_else(|| bad("a broken zip member"))?;
        let data = self.src.range(offset + skip, size)?;
        match method {
            0 => Ok(data),
            8 => {
                let mut out = Vec::new();
                io::Read::read_to_end(&mut flate2::read::DeflateDecoder::new(&data[..]), &mut out)?;
                Ok(out)
            }
            m => Err(bad(format!("{name}: zip method {m} is not supported"))),
        }
    }
}

/// The entries of a central directory.
fn entries(dir: &[u8]) -> Vec<Entry> {
    let mut out = Vec::new();
    let mut i = 0;
    while dir.get(i..i + 4) == Some(&CENTRAL[..]) {
        let Some(e) = entry(dir, i) else { break };
        i = e.1;
        out.push(e.0);
    }
    out
}

/// The entry at `i`, and where the next one starts.
fn entry(dir: &[u8], i: usize) -> Option<(Entry, usize)> {
    let (n, x, c) = (u16_at(dir, i + 28)? as usize, u16_at(dir, i + 30)? as usize, u16_at(dir, i + 32)? as usize);
    let name = String::from_utf8_lossy(dir.get(i + 46..i + 46 + n)?).into_owned();
    let e = Entry { name, method: u16_at(dir, i + 10)? as u16, size: u32_at(dir, i + 20)?, offset: u32_at(dir, i + 42)? };
    Some((e, i + 46 + n + x + c))
}

#[cfg(test)]
#[path = "remote_tests.rs"]
mod tests;
