//! Subsistema de extração de documentos PDF (CMap, pipeline geométrico e extrator unificado).

pub mod cmap;
pub mod native;
pub mod pipeline;

use std::path::PathBuf;

use crate::domain::ir::ExtractedBookPayload;
use crate::domain::metadata::BookMetadata;
use crate::domain::traits::ContentSourceExtractor;

pub use cmap::repair_broken_cmap_cid;
pub use native::extract_pdf_with_cover_and_figures;
pub use pipeline::PdfProsePipeline;

/// Extrator unificado para livros em formato PDF implementando [`ContentSourceExtractor`].
pub struct PdfExtractor {
    /// Caminho para o arquivo PDF local.
    pub pdf_path: PathBuf,
    /// Percentual de corte de margem superior/inferior.
    pub margin_cutoff: f32,
    /// Metadados editoriais base.
    pub default_metadata: BookMetadata,
}

impl ContentSourceExtractor for PdfExtractor {
    fn extract_payload(&self) -> Result<ExtractedBookPayload, Box<dyn std::error::Error>> {
        let (cover_jpeg_bytes, raw_pages, all_figures) =
            extract_pdf_with_cover_and_figures(&self.pdf_path)?;
        let pipeline = PdfProsePipeline::from_pages(&raw_pages, self.margin_cutoff);
        let chapters = pipeline.extract_chapters(&raw_pages);
        let mut meta = self.default_metadata.clone();
        meta.cover_jpeg_bytes = cover_jpeg_bytes;

        Ok(ExtractedBookPayload {
            metadata: meta,
            chapters,
            figures: all_figures,
        })
    }
}
