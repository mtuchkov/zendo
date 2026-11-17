//! Color themes. Each theme has the same named slots as the design canvas.

use ratatui::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Theme {
    pub name: &'static str,
    pub blurb: &'static str,
    pub bg: Color,
    pub panel: Color,
    /// The background of the float with a new note: it stands out of the theme, under the ink of
    /// `on_float` (FLOAT-16).
    pub float: Color,
    pub fg: Color,
    pub dim: Color,
    pub line: Color,
    pub sel: Color,
    pub acc: Color,
    pub acc2: Color,
    pub add: Color,
    pub del: Color,
    /// The colors of file extensions in the file tree.
    pub ext: Ext,
}

/// The colors of the kinds of files in the file tree, by extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ext {
    /// Prose: md, txt, rst, adoc.
    pub doc: Color,
    /// Source code.
    pub code: Color,
    /// Configuration and data: toml, yaml, json, xml, ini, lock.
    pub config: Color,
    /// Scripts and builds: sh, Makefile, Dockerfile.
    pub script: Color,
    /// Web: html, css.
    pub web: Color,
    pub other: Color,
}

/// The kind of a file by its extension, or its name when it has none.
pub fn ext_color(ext: &Ext, name: &str) -> Color {
    let e = name.rsplit_once('.').map_or(name, |(_, e)| e).to_ascii_lowercase();
    match e.as_str() {
        "md" | "txt" | "rst" | "adoc" | "org" => ext.doc,
        "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "cfg" | "conf" | "lock" | "properties" | "csv" | "env" => ext.config,
        "sh" | "bash" | "zsh" | "fish" | "makefile" | "dockerfile" | "containerfile" | "mk" | "ps1" | "bat" => ext.script,
        "html" | "htm" | "css" | "scss" | "sass" | "less" | "svg" | "vue" | "svelte" => ext.web,
        "rs" | "go" | "py" | "java" | "kt" | "kts" | "scala" | "c" | "h" | "cc" | "cpp" | "hpp" | "cs" | "js" | "mjs" | "cjs" | "ts" | "tsx" | "jsx"
        | "rb" | "php" | "swift" | "lua" | "sql" | "hs" | "ml" | "ex" | "exs" | "erl" | "clj" | "dart" | "zig" | "r" | "pl" | "groovy" | "gradle" | "proto" => ext.code,
        _ => ext.other,
    }
}

const fn hex(v: u32) -> Color {
    Color::Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

pub const THEMES: [Theme; 4] = [
    Theme {
        name: "Ink",
        blurb: "dark, warm",
        bg: hex(0x16171A),
        panel: hex(0x1C1E22),
        float: SAND,
        fg: hex(0xD6D3CB),
        dim: hex(0x8B8D94),
        line: hex(0x2A2D33),
        sel: hex(0x262A31),
        acc: hex(0xE2B15B),
        acc2: hex(0x7FB4CA),
        add: hex(0x8FBF8F),
        del: hex(0xE0937F),
        ext: Ext { doc: hex(0x7FB4CA), code: hex(0xE2B15B), config: hex(0xB89BD6), script: hex(0x8FBF8F), web: hex(0xE0937F), other: hex(0x8B8D94) },
    },
    Theme {
        name: "Daylight",
        blurb: "light, cool",
        bg: hex(0xF4F5F7),
        panel: hex(0xEAECEF),
        float: UMBER,
        fg: hex(0x22252B),
        dim: hex(0x5E646E),
        line: hex(0xD7DAE0),
        sel: hex(0xDFE4EB),
        acc: hex(0xA84A06),
        acc2: hex(0x1F6FA8),
        add: hex(0x2E6B3A),
        del: hex(0xA63D2A),
        ext: Ext { doc: hex(0x1F6FA8), code: hex(0xA84A06), config: hex(0x6B4BA6), script: hex(0x2E6B3A), web: hex(0xA63D2A), other: hex(0x5E646E) },
    },
    Theme {
        name: "Moss",
        blurb: "dark, soft",
        bg: hex(0x141A17),
        panel: hex(0x19211D),
        float: SAND,
        fg: hex(0xC9D1C5),
        dim: hex(0x84907F),
        line: hex(0x26302A),
        sel: hex(0x222C26),
        acc: hex(0xD9A65A),
        acc2: hex(0x8FBF9F),
        add: hex(0x9CCB9C),
        del: hex(0xE0937F),
        ext: Ext { doc: hex(0x8FBF9F), code: hex(0xD9A65A), config: hex(0xB3A1D0), script: hex(0x9CCB9C), web: hex(0xE0937F), other: hex(0x84907F) },
    },
    Theme {
        name: "Mono",
        blurb: "high contrast",
        bg: hex(0x0B0B0C),
        panel: hex(0x131314),
        float: SAND,
        fg: hex(0xEDEDED),
        dim: hex(0x9A9A9A),
        line: hex(0x2A2A2A),
        sel: hex(0x222222),
        acc: hex(0xFFFFFF),
        acc2: hex(0xBDBDBD),
        add: hex(0xEDEDED),
        del: hex(0x9A9A9A),
        ext: Ext { doc: hex(0xEDEDED), code: hex(0xFFFFFF), config: hex(0xBDBDBD), script: hex(0xD4D4D4), web: hex(0xC8C8C8), other: hex(0x9A9A9A) },
    },
];

/// The float color of a dark theme: sand.
const SAND: Color = hex(0xEEEAE0);
/// The float color of a light theme: umber.
const UMBER: Color = hex(0x2B2722);

/// The color of the cursor in the float: rust, it shows on sand and on umber (FLOAT-16).
pub const CARET: Color = hex(0xC8442B);

/// The ink on sand: dark, warm colors that read on a light background.
const ON_SAND: Theme = Theme {
    name: "Sand",
    blurb: "the float of a dark theme",
    bg: SAND,
    // The background of code: a darker sand.
    panel: hex(0xE3DED1),
    float: SAND,
    fg: hex(0x26231D),
    dim: hex(0x5F584C),
    line: hex(0xA79F8E),
    sel: hex(0xDFD9CB),
    acc: hex(0x9A4305),
    acc2: hex(0x1D6496),
    add: hex(0x2E6B3A),
    del: hex(0xA03A27),
    ext: Ext { doc: hex(0x1D6496), code: hex(0x9A4305), config: hex(0x6B4BA6), script: hex(0x2E6B3A), web: hex(0xA03A27), other: hex(0x5F584C) },
};

/// The ink on umber: light, warm colors that read on a dark background.
const ON_UMBER: Theme = Theme {
    name: "Umber",
    blurb: "the float of a light theme",
    bg: UMBER,
    // The background of code: a lighter umber.
    panel: hex(0x38332C),
    float: UMBER,
    fg: hex(0xE6E0D2),
    dim: hex(0xA49C8C),
    line: hex(0x524B40),
    sel: hex(0x3E3830),
    acc: hex(0xE2B15B),
    acc2: hex(0x7FB4CA),
    add: hex(0x8FBF8F),
    del: hex(0xE0937F),
    ext: Ext { doc: hex(0x7FB4CA), code: hex(0xE2B15B), config: hex(0xB89BD6), script: hex(0x8FBF8F), web: hex(0xE0937F), other: hex(0xA49C8C) },
};

/// A light color: its perceived brightness is above the middle. A color that is not RGB is not.
const fn light(c: Color) -> bool {
    match c {
        Color::Rgb(r, g, b) => 299 * r as u32 + 587 * g as u32 + 114 * b as u32 > 128_000,
        _ => false,
    }
}

impl Theme {
    /// The colors of the float: its background the float color of the theme, its text the ink
    /// that reads on it, dark on a light float and light on a dark one (FLOAT-16).
    pub fn on_float(&self) -> Theme {
        let ink = if light(self.float) { ON_SAND } else { ON_UMBER };
        Theme { bg: self.float, float: self.float, ..ink }
    }
}

pub fn index_of(name: &str) -> usize {
    THEMES
        .iter()
        .position(|t| t.name.eq_ignore_ascii_case(name))
        .unwrap_or(0)
}

/// The period of a blinking button: from its color to the accent and back (ATTN-3).
pub const PULSE: std::time::Duration = std::time::Duration::from_secs(2);

/// The pulse of a blinking button `t` after it started: 0, then 1 after half the period, then 0
/// again; smooth at both ends (ATTN-3).
pub fn pulse(t: std::time::Duration) -> f32 {
    let x = t.as_secs_f32() / PULSE.as_secs_f32() * std::f32::consts::TAU;
    (1.0 - x.cos()) / 2.0
}

/// `a` mixed with `b`: `k` 0 gives `a`, 1 gives `b`. A color that is not RGB has no mix: the
/// nearer end.
pub fn mix(a: Color, b: Color, k: f32) -> Color {
    let k = k.clamp(0.0, 1.0);
    match (a, b) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let m = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * k).round() as u8;
            Color::Rgb(m(r1, r2), m(g1, g2), m(b1, b2))
        }
        _ if k < 0.5 => a,
        _ => b,
    }
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod tests;
