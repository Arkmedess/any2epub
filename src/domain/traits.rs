//! Contratos e portas de abstração para extratores de conteúdo multi-fonte.

use crate::domain::ir::ExtractedBookPayload;

/// Trait de contrato para fontes de conteúdo (Arquitetura em Estágios / Pipeline).
///
/// Cada motor de entrada (PDF, mdBook, Sphinx, AsciiDoc, MDN, MediaWiki ou SinglePageSpec)
/// implementa este contrato para produzir um [`ExtractedBookPayload`] desacoplado do emissor EPUB.
pub trait ContentSourceExtractor {
    /// Executa a extração completa da fonte configurada e retorna a Representação Intermediária.
    fn extract_payload(&self) -> Result<ExtractedBookPayload, Box<dyn std::error::Error>>;
}
