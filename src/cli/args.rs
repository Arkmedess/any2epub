//! Definição dos argumentos de linha de comando (`CliArgs`) e enumeração de motores de entrada (`SourceKind`).

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use uuid::Uuid;

use crate::domain::metadata::BookMetadata;

/// Tipo de fonte de entrada suportada pelo motor multi-fonte.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// Detecta automaticamente PDF, mdBook, Sphinx, AsciiDoc, MediaWiki, SinglePageSpec ou MDN.
    Auto,
    /// Livro ou documento em formato PDF local (com reparo CMap e figuras).
    Pdf,
    /// Artigo Web único (extração estilo Readability + OpenGraph).
    WebArticle,
    /// Site de Documentação linear / mdBook (ex: `https://doc.rust-lang.org/book/`).
    WebDocs,
    /// Documentação técnica gerada por Sphinx / ReadTheDocs (ex: `https://docs.python.org/3/tutorial/`).
    SphinxDocs,
    /// Livro técnico em AsciiDoc / Pro Git (ex: `https://git-scm.com/book/en/v2`).
    AsciiDocBook,
    /// Especificação monolítica segmentada por H2 (ex: `https://go.dev/doc/effective_go`).
    SinglePageSpec,
    /// Guia técnico estruturado no motor Yari / MDN (ex: `https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide`).
    MdnGuide,
    /// Portal ou Wiki em Grafo MediaWiki (ex: `https://wiki.archlinux.org/title/Main_page`).
    MediaWiki,
    /// Arquivo ou diretório contendo documentos Markdown locais (`.md`, `.markdown`).
    Markdown,
}

impl SourceKind {
    /// Resolve o motor efetivo de extração com base na URL ou caminho do arquivo quando configurado como [`SourceKind::Auto`].
    pub fn resolve_effective_kind(self, input: &str) -> Self {
        match self {
            Self::Auto => {
                let is_web_url = input.starts_with("http://") || input.starts_with("https://");
                if !is_web_url {
                    if input.ends_with(".md") || input.ends_with(".markdown") {
                        return Self::Markdown;
                    }
                    return Self::Pdf;
                }
                if input.contains("wiki.") || input.contains("/title/") {
                    Self::MediaWiki
                } else if input.contains("docs.python.org") || input.contains("readthedocs") {
                    Self::SphinxDocs
                } else if input.contains("git-scm.com/book") {
                    Self::AsciiDocBook
                } else if input.contains("effective_go") || input.contains("go.dev/doc/") {
                    Self::SinglePageSpec
                } else if input.contains("developer.mozilla.org") {
                    Self::MdnGuide
                } else if input.contains("/book") || input.contains("docs") {
                    Self::WebDocs
                } else {
                    Self::WebArticle
                }
            }
            other => other,
        }
    }
}

/// Subcomandos opcionais suportados pelo CLI `any2epub`.
#[derive(Subcommand, Debug, Clone)]
pub enum CliCommand {
    /// Valida estruturalmente um pacote EPUB existente contra a especificação W3C EPUB 3.3.
    Validate {
        /// Caminho do arquivo EPUB a validar.
        path: PathBuf,
        /// Emite resultado em formato JSON estruturado.
        #[arg(long)]
        json: bool,
    },
    /// Visualiza e inspeciona interativamente as páginas paginadas de um arquivo EPUB.
    View {
        /// Caminho do arquivo EPUB a visualizar.
        path: PathBuf,
        /// Backend de renderização (`native` ou `koreader`).
        #[arg(long, default_value = "native")]
        backend: String,
        /// Tema cromático inicial (`paper-white`, `sepia`, `night`, `e-ink-mono`).
        #[arg(long, default_value = "sepia")]
        theme: String,
        /// Página inicial (base 0).
        #[arg(short, long, default_value_t = 0)]
        page: usize,
        /// Inicia a sessão de leitura interativa estilo KOReader no terminal.
        #[arg(short, long)]
        interactive: bool,
        /// Abre a janela desktop nativa do visualizador de EPUB.
        #[arg(short, long)]
        gui: bool,
    },
}

/// Argumentos de linha de comando (CLI) do conversor `any2epub` (`any2epub-rs`).
#[derive(Parser, Debug, Clone)]
#[command(
    name = "any2epub",
    version = "0.11.0",
    about = "Conversor autoral multi-fonte (PDF, mdBook, MediaWiki/ArchWiki, Sites) -> EPUB 3.3 com validação W3C e suporte a visualizador KOReader/Rust"
)]
pub struct CliArgs {
    /// Subcomando opcional (`validate`).
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    /// Caminho do arquivo PDF local ou URL inicial (ex: `https://doc.rust-lang.org/book/`).
    #[arg(short, long)]
    pub input: Option<String>,

    /// Caminho do arquivo EPUB 3.3 de saída.
    #[arg(short, long, default_value = "output_book.epub")]
    pub output: PathBuf,

    /// Executa validação de conformidade estrutural W3C EPUB 3.3 após a geração.
    #[arg(long)]
    pub validate: bool,

    /// Abre o visualizador de leitura imediatamente após a conversão.
    #[arg(long)]
    pub view: bool,

    /// Abre a janela desktop nativa do visualizador de leitura imediatamente após a conversão.
    #[arg(short, long)]
    pub gui: bool,

    /// Emite relatórios ou dados em formato JSON estruturado (ideal para CI/automações).
    #[arg(long)]
    pub json: bool,

    /// Modo silencioso: oculta mensagens de progresso informativas em stderr.
    #[arg(short, long)]
    pub quiet: bool,

    /// Modo de extração da fonte (`auto`, `pdf`, `web-article`, `web-docs`, `media-wiki`, etc.).
    #[arg(long, value_enum, default_value_t = SourceKind::Auto)]
    pub source_kind: SourceKind,

    /// Seletor CSS para capturar a árvore de capítulos e subcapítulos no modo `web-docs`.
    #[arg(
        long,
        default_value = "nav#sidebar ol.chapter li a, div.toctree-wrapper li.toctree-l1 > a, ol.book-toc li.chapter ol li a, nav.sidebar a, aside a, .sidebar a"
    )]
    pub toc_selector: String,

    /// Profundidade máxima de busca em largura (BFS) no modo `media-wiki` (`1` = artigos linkados na página raiz).
    #[arg(long, default_value_t = 1)]
    pub max_depth: usize,

    /// Limite máximo de capítulos coletados em grafos MediaWiki.
    #[arg(long, default_value_t = 20)]
    pub max_chapters: usize,

    /// Título principal da obra para o manifesto OPF (`<dc:title>`). Se omitido, é inferido do nome do arquivo ou fonte.
    #[arg(short, long)]
    pub title: Option<String>,

    /// Autores da obra separados por vírgula (`<dc:creator>`). Se omitido, usa autor padrão genérico.
    #[arg(short, long)]
    pub author: Option<String>,

    /// Colaborador / Tradutor (`<dc:contributor>`).
    #[arg(long)]
    pub translator: Option<String>,

    /// Editora da publicação (`<dc:publisher>`).
    #[arg(long)]
    pub publisher: Option<String>,

    /// Identificador ISBN ou URI (`<dc:identifier>`). Se omitido, gera `urn:uuid:<uuid>`.
    #[arg(long)]
    pub isbn: Option<String>,

    /// Sinopse editorial (`<dc:description>`).
    #[arg(long)]
    pub synopsis: Option<String>,

    /// Código de idioma BCP 47 (padrão: `en` ou `pt-BR`).
    #[arg(short, long, default_value = "en")]
    pub lang: String,

    /// Percentual de corte de margem superior/inferior para cabeçalhos e rodapés no PDF.
    #[arg(long, default_value_t = 0.05)]
    pub margin_cutoff: f32,
}

impl CliArgs {
    /// Constrói a estrutura inicial de [`BookMetadata`] a partir dos argumentos fornecidos na CLI
    /// com inferência automática de metadados quando não especificados.
    pub fn build_base_metadata(&self) -> BookMetadata {
        let generated_uuid = Uuid::new_v4().to_string();

        let title = match &self.title {
            Some(t) if !t.trim().is_empty() => t.trim().to_string(),
            _ => {
                if let Some(inp) = &self.input {
                    let path = std::path::Path::new(inp);
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        stem.replace(['_', '-'], " ")
                    } else {
                        String::from("Untitled Publication")
                    }
                } else {
                    String::from("Untitled Publication")
                }
            }
        };

        let authors: Vec<String> = match &self.author {
            Some(a) if !a.trim().is_empty() => a
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            _ => vec![String::from("any2epub Author")],
        };

        let isbn = match &self.isbn {
            Some(i) if !i.trim().is_empty() => i.trim().to_string(),
            _ => format!("urn:uuid:{}", generated_uuid),
        };

        let publisher = match &self.publisher {
            Some(p) if !p.trim().is_empty() => p.trim().to_string(),
            _ => String::from("any2epub Universal Engine"),
        };

        let synopsis = match &self.synopsis {
            Some(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => format!("Publicação '{}' em formato EPUB 3.3 gerada por any2epub.", title),
        };

        let translator = self.translator.clone().unwrap_or_default();

        BookMetadata {
            uuid: generated_uuid,
            isbn,
            title,
            authors,
            translator,
            publisher,
            publication_date: String::from("2026-11-07"),
            synopsis,
            language: self.lang.clone(),
            cover_jpeg_bytes: None,
        }
    }
}
