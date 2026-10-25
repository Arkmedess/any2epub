//! Segmentador de especificações técnicas de página única (`SinglePageSpecExtractor`).

use scraper::{Html, Selector};

use crate::domain::ir::{
    insert_subtopic_node, Chapter, ChapterElement, ExtractedBookPayload, TocNode,
};
use crate::domain::metadata::BookMetadata;
use crate::domain::traits::ContentSourceExtractor;
use crate::extractors::mediawiki::MediaWikiExtractor;
use crate::extractors::web::WebSiteExtractor;
use crate::utils::text::{escape_xhtml_fast, normalize_whitespace, slugify_anchor_id};

/// Extrator especializado em especificações monolíticas (ex: Effective Go em `go.dev/doc/effective_go`),
/// convertendo cada seção `<h2>` em um capítulo independente e subtópicos `<h3>`/`<h4>` em nós do sumário.
pub struct SinglePageSpecExtractor {
    /// URL da especificação monolítica.
    pub url: String,
    /// Metadados editoriais base.
    pub default_metadata: BookMetadata,
}

impl SinglePageSpecExtractor {
    /// Segmenta um documento HTML monolítico em múltiplos capítulos estruturados por `<h2>`.
    pub fn parse_spec_document(&self, doc: &Html) -> Vec<Chapter> {
        let block_sel = Selector::parse(
            "article.Doc h2, article.Doc h3, article.Doc h4, article.Doc p, article.Doc li, \
             article.Doc dt, article.Doc dd, article.Doc blockquote, article.Doc pre, article.Doc table, \
             main h2, main h3, main h4, main p, main li, main dt, main dd, main blockquote, main pre, main table",
        )
        .unwrap();

        let mut chapters: Vec<Chapter> = Vec::new();
        let mut current_ch_idx = 0usize;
        let mut current_title = String::new();
        let mut current_elements: Vec<ChapterElement> = Vec::new();
        let mut current_subtopics: Vec<TocNode> = Vec::new();
        let mut h3_idx = 0usize;
        let mut h4_idx = 0usize;
        let mut anchor_seq = 1usize;

        for node in doc.select(&block_sel) {
            let tag = node.value().name();
            if tag == "pre" {
                let code_txt =
                    WebSiteExtractor::clean_code_toolbar_prefix(&node.text().collect::<String>());
                if !code_txt.trim().is_empty() {
                    current_elements.push(ChapterElement::CodeBlock(escape_xhtml_fast(
                        code_txt.trim(),
                    )));
                }
                continue;
            }

            if tag == "table" {
                if let Some(tbl) = MediaWikiExtractor::extract_wikitable(node) {
                    current_elements.push(tbl);
                }
                continue;
            }

            if matches!(tag, "li" | "dd" | "blockquote")
                && WebSiteExtractor::has_nested_block_children(node)
            {
                continue;
            }

            let clean_txt = node.text().collect::<String>().replace('¶', "");
            let trimmed = clean_txt.trim();
            if trimmed.is_empty() || trimmed == "Table of Contents" {
                continue;
            }

            match tag {
                "h2" => {
                    if !current_elements.is_empty() && current_ch_idx > 0 {
                        chapters.push(Chapter {
                            index: current_ch_idx,
                            number_prefix: format!("{}", current_ch_idx),
                            title: format!("{}. {}", current_ch_idx, current_title),
                            filename: format!("chapter_{:03}.xhtml", current_ch_idx),
                            elements: std::mem::take(&mut current_elements),
                            footnotes: Vec::new(),
                            subtopics: std::mem::take(&mut current_subtopics),
                        });
                    }
                    current_ch_idx += 1;
                    current_title = trimmed.to_string();
                    h3_idx = 0;
                    h4_idx = 0;
                }
                "h3" => {
                    if current_ch_idx == 0 {
                        current_ch_idx = 1;
                        current_title = String::from("Introduction");
                    }
                    h3_idx += 1;
                    h4_idx = 0;
                    let ch_fname = format!("chapter_{:03}.xhtml", current_ch_idx);
                    let num_prefix = format!("{}.{}", current_ch_idx, h3_idx);
                    let id = node
                        .value()
                        .attr("id")
                        .map(|s| slugify_anchor_id(s, anchor_seq))
                        .unwrap_or_else(|| slugify_anchor_id(trimmed, anchor_seq));
                    anchor_seq += 1;
                    insert_subtopic_node(
                        &mut current_subtopics,
                        2,
                        num_prefix.clone(),
                        trimmed.to_string(),
                        format!("{}#{}", ch_fname, id),
                    );
                    current_elements.push(ChapterElement::SubheadingH2 {
                        id,
                        number_prefix: num_prefix,
                        title: escape_xhtml_fast(trimmed),
                    });
                }
                "h4" => {
                    if current_ch_idx == 0 {
                        current_ch_idx = 1;
                        current_title = String::from("Introduction");
                    }
                    if h3_idx == 0 {
                        h3_idx = 1;
                    }
                    h4_idx += 1;
                    let ch_fname = format!("chapter_{:03}.xhtml", current_ch_idx);
                    let num_prefix = format!("{}.{}.{}", current_ch_idx, h3_idx, h4_idx);
                    let id = slugify_anchor_id(trimmed, anchor_seq);
                    anchor_seq += 1;
                    insert_subtopic_node(
                        &mut current_subtopics,
                        3,
                        num_prefix.clone(),
                        trimmed.to_string(),
                        format!("{}#{}", ch_fname, id),
                    );
                    current_elements.push(ChapterElement::SubheadingH3 {
                        id,
                        number_prefix: num_prefix,
                        title: escape_xhtml_fast(trimmed),
                    });
                }
                "dt" => {
                    if current_ch_idx > 0 {
                        current_elements.push(ChapterElement::StandardParagraph(format!(
                            "<strong><code>{}</code></strong>",
                            escape_xhtml_fast(&normalize_whitespace(trimmed))
                        )));
                    }
                }
                "dd" => {
                    if current_ch_idx > 0 {
                        current_elements.push(ChapterElement::StandardParagraph(format!(
                            "— {}",
                            escape_xhtml_fast(&normalize_whitespace(trimmed))
                        )));
                    }
                }
                "li" => {
                    if current_ch_idx > 0 {
                        current_elements.push(ChapterElement::StandardParagraph(format!(
                            "• {}",
                            escape_xhtml_fast(&normalize_whitespace(trimmed))
                        )));
                    }
                }
                _ => {
                    if current_ch_idx > 0 {
                        current_elements.push(ChapterElement::StandardParagraph(
                            escape_xhtml_fast(&normalize_whitespace(trimmed)),
                        ));
                    }
                }
            }
        }

        if !current_elements.is_empty() && current_ch_idx > 0 {
            chapters.push(Chapter {
                index: current_ch_idx,
                number_prefix: format!("{}", current_ch_idx),
                title: format!("{}. {}", current_ch_idx, current_title),
                filename: format!("chapter_{:03}.xhtml", current_ch_idx),
                elements: current_elements,
                footnotes: Vec::new(),
                subtopics: current_subtopics,
            });
        }

        chapters
    }
}

impl ContentSourceExtractor for SinglePageSpecExtractor {
    fn extract_payload(&self) -> Result<ExtractedBookPayload, Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("pdf2epub-rs/0.9.0 (+https://www.w3.org/TR/epub-33/)")
            .build()?;

        let html_str = client.get(&self.url).send()?.text()?;
        let doc = Html::parse_document(&html_str);
        let chapters = self.parse_spec_document(&doc);

        Ok(ExtractedBookPayload {
            metadata: self.default_metadata.clone(),
            chapters,
            figures: Vec::new(),
        })
    }
}
