//! Contratos de abstração, erros e fábrica polimórfica de motores de leitura (`ReaderEngine`).

use std::fmt;
use std::path::Path;

use crate::domain::ir::TocNode;
use crate::viewer::bitmap::PageBitmap;
use crate::viewer::koreader::KoreaderCrengineEngine;
use crate::viewer::native::NativeRustReaderEngine;
use crate::viewer::options::ViewerRenderOptions;

/// Detalhes cadastrais de uma obra carregada pelo motor de leitura.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookDetails {
    /// Título principal do livro.
    pub title: String,
    /// Autores da obra.
    pub authors: Vec<String>,
    /// Quantidade de documentos XHTML ordenados na lista de leitura (`<spine>`).
    pub spine_documents_count: usize,
    /// Código de idioma da obra.
    pub language: String,
}

/// Erros decorrentes da operação de motores de visualização de EPUB.
#[derive(Debug)]
pub enum ViewerError {
    /// Falha de entrada/saída no sistema de arquivos.
    Io(std::io::Error),
    /// Pacote ZIP com estrutura corrompida ou inválida.
    ZipError(String),
    /// Estrutura de metadados OPF/XML ausente ou inválida.
    XmlError(String),
    /// Tentativa de renderizar página além do limite paginado.
    PageOutOfBounds { requested: usize, total: usize },
    /// Operação invocada antes de carregar um livro no motor.
    NotInitialized,
    /// Backend ou dependência nativa externa não disponível no ambiente atual.
    BackendUnavailable(String),
}

impl fmt::Display for ViewerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "Erro de I/O no visualizador: {}", err),
            Self::ZipError(msg) => write!(f, "Erro no contêiner ZIP do EPUB: {}", msg),
            Self::XmlError(msg) => write!(f, "Erro na sintaxe XML do pacote: {}", msg),
            Self::PageOutOfBounds { requested, total } => {
                write!(
                    f,
                    "Página {} fora dos limites (total paginado: {})",
                    requested, total
                )
            }
            Self::NotInitialized => write!(f, "Nenhum livro foi carregado no motor de leitura"),
            Self::BackendUnavailable(msg) => write!(f, "Backend indisponível: {}", msg),
        }
    }
}

impl std::error::Error for ViewerError {}

impl From<std::io::Error> for ViewerError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// Motores de renderização selecionáveis no backend do visualizador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerBackendKind {
    /// Motor nativo 100% Rust (layout determinístico, sem dependências externas C++).
    NativeRust,
    /// Motor C++ oficial do KOReader (`crengine` / CoolReader Engine via FFI).
    KoreaderCrengine,
}

/// Contrato polimórfico de abstração (Strategy / Port) para motores de visualização e paginação.
pub trait ReaderEngine: Send {
    /// Retorna o nome canônico do motor em execução (ex: `"native-rust"`, `"koreader-crengine"`).
    fn engine_name(&self) -> &'static str;

    /// Abre o pacote EPUB, processa metadados OPF, spine e carrega a estrutura do sumário.
    fn open_book(&mut self, path: &Path) -> Result<BookDetails, ViewerError>;

    /// Define as dimensões do viewport de leitura (largura x altura em pixels da tela).
    fn set_viewport(&mut self, width: u32, height: u32) -> Result<(), ViewerError>;

    /// Aplica configurações editoriais de tipografia, tema e margens, disparando repaginação se necessário.
    fn apply_render_options(&mut self, options: &ViewerRenderOptions) -> Result<(), ViewerError>;

    /// Retorna a contagem total de páginas discretas geradas para o viewport ativo.
    fn total_pages(&self) -> usize;

    /// Renderiza a página correspondente ao índice `page_index` (base 0) retornando seu buffer de pixels RGBA.
    fn render_page(&mut self, page_index: usize) -> Result<PageBitmap, ViewerError>;

    /// Retorna a árvore hierárquica do Sumário (TOC) da obra.
    fn table_of_contents(&self) -> &[TocNode];
}

/// Fábrica para instanciar o motor de visualização solicitado pelo usuário.
pub fn create_reader_engine(kind: ViewerBackendKind) -> Result<Box<dyn ReaderEngine>, ViewerError> {
    match kind {
        ViewerBackendKind::NativeRust => Ok(Box::new(NativeRustReaderEngine::new())),
        ViewerBackendKind::KoreaderCrengine => Ok(Box::new(KoreaderCrengineEngine::new())),
    }
}
