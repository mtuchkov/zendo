use std::time::Instant;
use zendo::docs::tree::{Tree, View};
#[test]
#[ignore]
fn scan_time() {
    let root = std::path::PathBuf::from(std::env::var("SCAN_DIR").unwrap());
    for view in [View::Docs, View::Text, View::Text] {
        let mut t = Tree::default();
        t.view = view;
        let s = Instant::now();
        t.scan(&root);
        let scan = s.elapsed().as_secs_f64() * 1e3;
        let s = Instant::now();
        let n = t.rows().len();
        println!("{view:?}: {} files, scan {scan:.0} ms, rows {n} in {:.0} ms", t.files.len(), s.elapsed().as_secs_f64() * 1e3);
    }
}
