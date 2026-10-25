//! Estruturas de metadados editoriais Dublin Core e figuras extraídas.

/// Metadados editoriais completos da publicação EPUB 3.3 (Dublin Core + Capa).
#[derive(Debug, Clone)]
pub struct BookMetadata {
    /// Identificador único universal (UUID v4) da edição.
    pub uuid: String,
    /// Identificador ISBN ou URI editorial (`<dc:identifier>`).
    pub isbn: String,
    /// Título principal da obra (`<dc:title>`).
    pub title: String,
    /// Lista de autores da obra (`<dc:creator>`).
    pub authors: Vec<String>,
    /// Colaborador ou tradutor (`<dc:contributor>`).
    pub translator: String,
    /// Editora da publicação (`<dc:publisher>`).
    pub publisher: String,
    /// Data de publicação no padrão ISO 8601 (`<dc:date>`).
    pub publication_date: String,
    /// Sinopse editorial (`<dc:description>`).
    pub synopsis: String,
    /// Código de idioma BCP 47 (`<dc:language>`).
    pub language: String,
    /// Bytes JPEG da imagem de capa opcional.
    pub cover_jpeg_bytes: Option<Vec<u8>>,
}

/// Imagem extraída (diagrama de página PDF ou figura baixada da Web).
#[derive(Debug, Clone)]
pub struct ExtractedFigure {
    /// Identificador único no manifesto OPF (ex: `"fig_001"`).
    pub id: String,
    /// Caminho relativo dentro do pacote OEBPS (ex: `"images/fig_001_p20.jpg"`).
    pub href: String,
    /// Posição vertical normalizada no eixo Y da página de origem.
    pub top: f32,
    /// Conteúdo binário da imagem codificado em JPEG.
    pub jpeg_bytes: Vec<u8>,
}
