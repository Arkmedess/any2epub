//! # `any2epub-rs` (`v0.10.0`) — Motor Autoral Multi-Fonte para W3C EPUB 3.3
//!
//! Biblioteca e pipeline modular em Rust projetado para converter:
//! 1. **Livros em PDF de centenas de páginas** (com reparo de CMap CID, capa e figuras),
//! 2. **Documentações Lineares mdBook, Sphinx, Pro Git AsciiDoc e MDN Yari**,
//! 3. **Portais em Grafo MediaWiki / ArchWiki** (com busca em largura BFS e subtópicos até 4º nível),
//! 4. **Especificações Monolíticas (`SinglePageSpec` / Effective Go)**,
//!
//! Gerando pacotes **W3C EPUB 3.3** estritos com Sumário Hierárquico Aninhado (`<ol>` recursivo em `nav.xhtml`).

pub mod cli;
pub mod domain;
pub mod epub;
pub mod extractors;
pub mod utils;
pub mod viewer;

pub use cli::{CliArgs, CliCommand, SourceKind};
pub use domain::{
    insert_subtopic_node, BookMetadata, CalloutKind, Chapter, ChapterElement,
    ContentSourceExtractor, ExtractedBookPayload, ExtractedFigure, FootnoteEntry, PdfWordBox,
    RawPdfPage, ReconstructedLine, TocNode,
};
pub use epub::validator::{NativeEpubValidator, ValidationReport};
pub use epub::NativeEpub3Builder;
pub use extractors::{
    build_extractor, extract_pdf_with_cover_and_figures, repair_broken_cmap_cid,
    LocalMarkdownExtractor, MediaWikiExtractor, PdfExtractor, PdfProsePipeline,
    SinglePageSpecExtractor, WebSiteExtractor,
};
pub use utils::{escape_xhtml_fast, normalize_whitespace, slugify_anchor_id, strip_leading_tags};
