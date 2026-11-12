//! AGENTS.md, Code: a source file has at most 300 lines, a function at most 70, and unit tests live
//! in `*_tests.rs` files beside the code.

use std::path::{Path, PathBuf};
use syn::visit::Visit;

const FILE_LINES: usize = 300;
const FN_LINES: usize = 70;

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            sources(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") && !p.to_string_lossy().ends_with("_tests.rs") {
            out.push(p);
        }
    }
}

struct Fns<'a> {
    file: &'a str,
    long: Vec<String>,
    inline_tests: Vec<String>,
}

impl Fns<'_> {
    fn check(&mut self, name: &syn::Ident, start: proc_macro2::Span, block: &syn::Block) {
        use syn::spanned::Spanned;
        let lines = block.span().end().line - start.start().line + 1;
        if lines > FN_LINES {
            self.long.push(format!("{}: fn {name} has {lines} lines", self.file));
        }
    }
}

impl<'ast> Visit<'ast> for Fns<'_> {
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        use syn::spanned::Spanned;
        self.check(&f.sig.ident, f.sig.span(), &f.block);
        syn::visit::visit_item_fn(self, f);
    }
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        use syn::spanned::Spanned;
        self.check(&f.sig.ident, f.sig.span(), &f.block);
        syn::visit::visit_impl_item_fn(self, f);
    }
    fn visit_item_mod(&mut self, m: &'ast syn::ItemMod) {
        let test = m.attrs.iter().any(|a| a.path().is_ident("cfg") && quote_of(a).contains("test"));
        if test && m.content.is_some() {
            self.inline_tests.push(format!("{}: mod {} has its tests inline", self.file, m.ident));
        }
        syn::visit::visit_item_mod(self, m);
    }
}

fn quote_of(a: &syn::Attribute) -> String {
    match &a.meta {
        syn::Meta::List(l) => l.tokens.to_string(),
        _ => String::new(),
    }
}

#[test]
fn code_files_and_functions_stay_small() {
    let mut files = Vec::new();
    sources(Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path(), &mut files);
    files.sort();
    let mut bad = Vec::new();
    for p in &files {
        let text = std::fs::read_to_string(p).unwrap();
        let rel = p.strip_prefix(env!("CARGO_MANIFEST_DIR")).unwrap().display().to_string();
        let n = text.lines().count();
        if n > FILE_LINES {
            bad.push(format!("{rel}: {n} lines"));
        }
        let ast = syn::parse_file(&text).unwrap_or_else(|e| panic!("{rel}: {e}"));
        let mut v = Fns { file: &rel, long: Vec::new(), inline_tests: Vec::new() };
        v.visit_file(&ast);
        bad.extend(v.long);
        bad.extend(v.inline_tests);
    }
    assert!(bad.is_empty(), "files over {FILE_LINES} lines, functions over {FN_LINES} lines, inline tests:\n{}", bad.join("\n"));
}
