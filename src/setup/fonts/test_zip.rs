//! Zip archives for the tests: the local copies of the release archives.
use std::io::Write;

/// A zip archive of the members `(name, bytes, deflated)`.
pub fn zip(members: &[(&str, &[u8], bool)]) -> Vec<u8> {
    let (mut out, mut dir) = (Vec::new(), Vec::new());
    for (name, bytes, deflate) in members {
        let data = if *deflate { deflated(bytes) } else { bytes.to_vec() };
        let method: u16 = if *deflate { 8 } else { 0 };
        let offset = out.len() as u32;
        let sizes = [data.len() as u32, bytes.len() as u32];
        // The local header, with an extra field that the central directory does not have.
        out.extend([0x50, 0x4b, 0x03, 0x04, 20, 0, 0, 0]);
        out.extend(method.to_le_bytes());
        out.extend([0; 8]);
        sizes.iter().for_each(|s| out.extend(s.to_le_bytes()));
        out.extend((name.len() as u16).to_le_bytes());
        out.extend(4u16.to_le_bytes());
        out.extend(name.as_bytes());
        out.extend([0xca, 0xfe, 0, 0]);
        out.extend(&data);
        dir.extend([0x50, 0x4b, 0x01, 0x02, 20, 0, 20, 0, 0, 0]);
        dir.extend(method.to_le_bytes());
        dir.extend([0; 8]);
        sizes.iter().for_each(|s| dir.extend(s.to_le_bytes()));
        dir.extend((name.len() as u16).to_le_bytes());
        dir.extend([0; 12]);
        dir.extend(offset.to_le_bytes());
        dir.extend(name.as_bytes());
    }
    let (at, len, n) = (out.len() as u32, dir.len() as u32, members.len() as u16);
    out.extend(dir);
    out.extend([0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0]);
    out.extend(n.to_le_bytes());
    out.extend(n.to_le_bytes());
    out.extend(len.to_le_bytes());
    out.extend(at.to_le_bytes());
    out.extend([0, 0]);
    out
}

fn deflated(bytes: &[u8]) -> Vec<u8> {
    let mut e = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(bytes).unwrap();
    e.finish().unwrap()
}

/// The SHA-256 of the bytes, in hex.
pub fn sha(bytes: &[u8]) -> String {
    super::install::sha256(bytes)
}
