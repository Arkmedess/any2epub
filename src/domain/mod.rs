//! Modelos de domínio, Representação Intermediária (IR) e contratos de extração.

pub mod ir;
pub mod metadata;
pub mod traits;

pub use ir::{
    insert_subtopic_node, CalloutKind, Chapter, ChapterElement, ExtractedBookPayload,
    FootnoteEntry, PdfWordBox, RawPdfPage, ReconstructedLine, TocNode,
};
pub use metadata::{BookMetadata, ExtractedFigure};
pub use traits::ContentSourceExtractor;
