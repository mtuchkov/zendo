use super::*;
use crate::setup::fonts::test_zip::zip;

// FONT-1: zendo reads the members that it needs from a release archive, stored or deflated, by ranges.
#[test]
fn a_zip_gives_a_stored_and_a_deflated_member() {
    let bytes = zip(&[("ttf/A-Regular.ttf", b"regular regular regular", true), ("OFL.txt", b"license", false)]);
    let mut z = Zip::open(Box::new(Bytes(bytes))).unwrap();
    assert_eq!(z.read("ttf/A-Regular.ttf").unwrap(), b"regular regular regular");
    assert_eq!(z.read("OFL.txt").unwrap(), b"license");
}

// FONT-10: a member that is not in the archive and bytes that are not a zip are errors, not a panic.
#[test]
fn a_missing_member_and_a_broken_archive_are_errors() {
    let mut z = Zip::open(Box::new(Bytes(zip(&[("a", b"x", false)])))).unwrap();
    assert!(z.read("b").unwrap_err().to_string().contains("not in the archive"));
    assert!(Zip::open(Box::new(Bytes(b"<html>not found</html>".to_vec()))).is_err());
    let mut cut = zip(&[("a", b"x", false)]);
    cut.truncate(10);
    assert!(Zip::open(Box::new(Bytes(cut))).is_err());
}

// FONT-1: the size of a file behind the redirects of GitHub is in the last `content-range`.
#[test]
fn the_size_is_in_the_last_content_range() {
    let head = "HTTP/2 302\r\nlocation: x\r\n\r\nHTTP/2 206\r\nContent-Range: bytes 0-0/5622857\r\n\r\n";
    assert_eq!(total(head), Some(5_622_857));
    assert_eq!(total("HTTP/2 200\r\n\r\n"), None);
}
