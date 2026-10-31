//! Motores de extração multi-fonte e fábrica polimórfica (`ContentSourceExtractor`).

pub mod markdown;
pub mod mediawiki;
pub mod pdf;
pub mod spec;
pub mod web;

use std::path::PathBuf;

use crate::cli::args::{CliArgs, SourceKind};
use crate::domain::metadata::BookMetadata;
use crate::domain::traits::ContentSourceExtractor;

pub use markdown::LocalMarkdownExtractor;
pub use mediawiki::MediaWikiExtractor;
pub use pdf::{
    extract_pdf_with_cover_and_figures, repair_broken_cmap_cid, PdfExtractor, PdfProsePipeline,
};
pub use spec::SinglePageSpecExtractor;
pub use web::WebSiteExtractor;

/// Constrói polimorficamente o extrator apropriado ([`ContentSourceExtractor`]) com base nos argumentos da CLI.
pub fn build_extractor(
    args: &CliArgs,
    effective_kind: SourceKind,
    base_metadata: BookMetadata,
) -> Box<dyn ContentSourceExtractor> {
    let input = args.input.clone().unwrap_or_default();
    match effective_kind {
        SourceKind::MediaWiki => Box::new(MediaWikiExtractor {
            root_url: input,
            max_depth: args.max_depth,
            max_chapters: args.max_chapters,
            default_metadata: base_metadata,
        }),
        SourceKind::SinglePageSpec => Box::new(SinglePageSpecExtractor {
            url: input,
            default_metadata: base_metadata,
        }),
        SourceKind::WebArticle
        | SourceKind::WebDocs
        | SourceKind::SphinxDocs
        | SourceKind::AsciiDocBook
        | SourceKind::MdnGuide => Box::new(WebSiteExtractor {
            start_url: input,
            crawl_docs_toc: effective_kind != SourceKind::WebArticle,
            toc_selector: args.toc_selector.clone(),
            default_metadata: base_metadata,
        }),
        SourceKind::Markdown => Box::new(LocalMarkdownExtractor {
            input_path: PathBuf::from(input),
            default_metadata: base_metadata,
        }),
        SourceKind::Pdf | SourceKind::Auto => Box::new(PdfExtractor {
            pdf_path: PathBuf::from(input),
            margin_cutoff: args.margin_cutoff,
            default_metadata: base_metadata,
        }),
    }
}
