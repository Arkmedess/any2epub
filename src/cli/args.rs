//! Definição dos argumentos de linha de comando (`CliArgs`) e enumeração de motores de entrada (`SourceKind`).

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
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
}

impl SourceKind {
    /// Resolve o motor efetivo de extração com base na URL ou caminho do arquivo quando configurado como [`SourceKind::Auto`].
    pub fn resolve_effective_kind(self, input: &str) -> Self {
        match self {
            Self::Auto => {
                let is_web_url = input.starts_with("http://") || input.starts_with("https://");
                if !is_web_url {
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

/// Argumentos de linha de comando (CLI) do conversor `pdf2epub` (`any2epub-rs`).
#[derive(Parser, Debug, Clone)]
#[command(
    name = "pdf2epub",
    version = "0.9.0",
    about = "Conversor autoral multi-fonte (PDF, mdBook, MediaWiki/ArchWiki, Sites) -> EPUB 3.3 com subtópicos hierárquicos (4.1.1, 4.1.5.1)"
)]
pub struct CliArgs {
    /// Caminho do arquivo PDF local ou URL inicial (ex: `https://doc.rust-lang.org/book/`).
    #[arg(short, long)]
    pub input: String,

    /// Caminho do arquivo EPUB 3.3 de saída.
    #[arg(short, long, default_value = "output_book.epub")]
    pub output: PathBuf,

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

    /// Título principal da obra para o manifesto OPF (`<dc:title>`).
    #[arg(
        short,
        long,
        default_value = "The Rust Programming Language (2024 Edition)"
    )]
    pub title: String,

    /// Autores da obra separados por vírgula (`<dc:creator>`).
    #[arg(
        short,
        long,
        default_value = "Steve Klabnik, Carol Nichols, Chris Krycho"
    )]
    pub author: String,

    /// Colaborador / Tradutor (`<dc:contributor>`).
    #[arg(long, default_value = "Rust Community")]
    pub translator: String,

    /// Editora da publicação (`<dc:publisher>`).
    #[arg(long, default_value = "No Starch Press / Rust Project")]
    pub publisher: String,

    /// Identificador ISBN ou URI (`<dc:identifier>`).
    #[arg(long, default_value = "urn:rust-book:2024-edition")]
    pub isbn: String,

    /// Sinopse editorial (`<dc:description>`).
    #[arg(
        long,
        default_value = "Edição completa em EPUB 3.3 com sumário hierárquico multinível (capítulos, subcapítulos X.Y e subtópicos X.Y.Z / X.Y.Z.W ancorados)."
    )]
    pub synopsis: String,

    /// Código de idioma BCP 47 (padrão: `en` ou `pt-BR`).
    #[arg(short, long, default_value = "en")]
    pub lang: String,

    /// Percentual de corte de margem superior/inferior para cabeçalhos e rodapés no PDF.
    #[arg(long, default_value_t = 0.05)]
    pub margin_cutoff: f32,
}

impl CliArgs {
    /// Constrói a estrutura inicial de [`BookMetadata`] a partir dos argumentos fornecidos na CLI.
    pub fn build_base_metadata(&self) -> BookMetadata {
        let authors: Vec<String> = self
            .author
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        BookMetadata {
            uuid: Uuid::new_v4().to_string(),
            isbn: self.isbn.clone(),
            title: self.title.clone(),
            authors,
            translator: self.translator.clone(),
            publisher: self.publisher.clone(),
            publication_date: String::from("2026-10-15"),
            synopsis: self.synopsis.clone(),
            language: self.lang.clone(),
            cover_jpeg_bytes: None,
        }
    }
}
