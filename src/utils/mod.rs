//! Utilitários transversais de manipulação textual, sanitização XHTML5 e expressões regulares compiladas.

pub mod regexes;
pub mod text;

pub use regexes::{
    caption_regex, chapter_heading_regex, footnote_line_regex, inline_footnote_ref_regex,
    terminal_punctuation_regex,
};
pub use text::{escape_xhtml_fast, normalize_whitespace, slugify_anchor_id, strip_leading_tags};
