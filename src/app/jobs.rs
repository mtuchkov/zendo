//! Visits that count as work, and the jobs of the agent in the background: recaps, titles, reviews.
use super::*;

/// One stay on a note. It counts as work after an edit or enough scrolls and clicks.
pub struct Visit {
    pub path: PathBuf,
    pub interactions: u32,
    pub edited: bool,
}

/// Scrolls and clicks that make a visit count, when there is no edit.
pub const ENGAGED: u32 = 12;
/// Claude writes recaps only after the user is idle this long.
pub const IDLE: Duration = Duration::from_secs(45);
/// At most this many automatic recaps per hour.
pub const AUTO_PER_HOUR: usize = 6;

pub struct RecapJob {
    pub rel: String,
    pub hash: String,
    pub text: String,
    pub buf: String,
    pub auto: bool,
    pub failed: Option<String>,
    /// Not a recap: a title and front matter for the file.
    pub meta: Option<crate::agent::work::Meta>,
    /// Not a recap: the confusing lines of the file.
    pub review: bool,
}
