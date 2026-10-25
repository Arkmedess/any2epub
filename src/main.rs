//! # pdf2epub-rs (v0.9.0) — Motor Autoral Multi-Fonte (PDF + mdBook + MediaWiki/ArchWiki -> EPUB 3.3)
//!
//! Conversor nativo em Rust projetado para converter:
//! 1. **Livros em PDF de centenas de páginas** (com reparo de CMap, capa e figuras),
//! 2. **Documentações Lineares mdBook** (calibrado para `https://doc.rust-lang.org/book/` com extração de subtópicos `4.1.1`, `4.1.5.1`, etc.),
//! 3. **Portais em Grafo MediaWiki / ArchWiki** (calibrado para `https://wiki.archlinux.org/title/Main_page` com subtópicos até 4º nível `1.1.1.1`),
//! 4. **Artigos Web Únicos**, gerando pacotes W3C EPUB 3.3 estritos com Sumário Hierárquico Aninhado (`<ol>` recursivo em `nav.xhtml`).

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{BufWriter, Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use clap::{Parser, ValueEnum};
use rayon::prelude::*;
use regex::Regex;
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Tipo de fonte de entrada suportada pelo motor multi-fonte
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// Detecta automaticamente PDF, mdBook, Sphinx, AsciiDoc, MediaWiki, SinglePageSpec ou MDN
    Auto,
    /// Livro ou documento em formato PDF local (com reparo CMap e figuras)
    Pdf,
    /// Artigo Web único (extração estilo Readability + OpenGraph)
    WebArticle,
    /// Site de Documentação linear / mdBook (ex: https://doc.rust-lang.org/book/)
    WebDocs,
    /// Documentação técnica gerada por Sphinx / ReadTheDocs (ex: https://docs.python.org/3/tutorial/)
    SphinxDocs,
    /// Livro técnico em AsciiDoc / Pro Git (ex: https://git-scm.com/book/en/v2)
    AsciiDocBook,
    /// Especificação monolítica segmentada por H2 (ex: https://go.dev/doc/effective_go)
    SinglePageSpec,
    /// Guia técnico estruturado no motor Yari / MDN (ex: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide)
    MdnGuide,
    /// Portal ou Wiki em Grafo MediaWiki (ex: https://wiki.archlinux.org/title/Main_page)
    MediaWiki,
}

/// Argumentos de linha de comando (CLI)
#[derive(Parser, Debug, Clone)]
#[command(
    name = "pdf2epub",
    version = "0.9.0",
    about = "Conversor autoral multi-fonte (PDF, mdBook, MediaWiki/ArchWiki, Sites) -> EPUB 3.3 com subtópicos hierárquicos (4.1.1, 4.1.5.1)"
)]
pub struct CliArgs {
    /// Caminho do arquivo PDF local ou URL inicial (ex: https://doc.rust-lang.org/book/)
    #[arg(short, long)]
    pub input: String,

    /// Caminho do arquivo EPUB 3.3 de saída
    #[arg(short, long, default_value = "output_book.epub")]
    pub output: PathBuf,

    /// Modo de extração da fonte (auto, pdf, web-article, web-docs, media-wiki)
    #[arg(long, value_enum, default_value_t = SourceKind::Auto)]
    pub source_kind: SourceKind,

    /// Seletor CSS para capturar a árvore de capítulos e subcapítulos no modo web-docs
    #[arg(
        long,
        default_value = "nav#sidebar ol.chapter li a, div.toctree-wrapper li.toctree-l1 > a, ol.book-toc li.chapter ol li a, nav.sidebar a, aside a, .sidebar a"
    )]
    pub toc_selector: String,

    /// Profundidade máxima de busca em largura (BFS) no modo media-wiki (1 = artigos linkados na página raiz)
    #[arg(long, default_value_t = 1)]
    pub max_depth: usize,

    /// Limite máximo de capítulos coletados em grafos MediaWiki
    #[arg(long, default_value_t = 20)]
    pub max_chapters: usize,

    /// Título principal da obra para o manifesto OPF (<dc:title>)
    #[arg(short, long, default_value = "The Rust Programming Language (2024 Edition)")]
    pub title: String,

    /// Autores da obra separados por vírgula (<dc:creator>)
    #[arg(short, long, default_value = "Steve Klabnik, Carol Nichols, Chris Krycho")]
    pub author: String,

    /// Colaborador / Tradutor (<dc:contributor>)
    #[arg(long, default_value = "Rust Community")]
    pub translator: String,

    /// Editora da publicação (<dc:publisher>)
    #[arg(long, default_value = "No Starch Press / Rust Project")]
    pub publisher: String,

    /// Identificador ISBN ou URI (<dc:identifier>)
    #[arg(long, default_value = "urn:rust-book:2024-edition")]
    pub isbn: String,

    /// Sinopse editorial (<dc:description>)
    #[arg(
        long,
        default_value = "Edição completa em EPUB 3.3 com sumário hierárquico multinível (capítulos, subcapítulos X.Y e subtópicos X.Y.Z / X.Y.Z.W ancorados)."
    )]
    pub synopsis: String,

    /// Código de idioma BCP 47 (padrão: en ou pt-BR)
    #[arg(short, long, default_value = "en")]
    pub lang: String,

    /// Percentual de corte de margem superior/inferior para cabeçalhos e rodapés no PDF
    #[arg(long, default_value_t = 0.05)]
    pub margin_cutoff: f32,
}

#[derive(Debug, Clone)]
pub struct TocNode {
    pub number_prefix: String,
    pub title: String,
    pub href: String,
    pub children: Vec<TocNode>,
}

#[derive(Debug, Clone)]
pub struct ExtractedBookPayload {
    pub metadata: BookMetadata,
    pub chapters: Vec<Chapter>,
    pub figures: Vec<ExtractedFigure>,
}

pub trait ContentSourceExtractor {
    fn extract_payload(&self) -> Result<ExtractedBookPayload, Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
pub struct BookMetadata {
    pub uuid: String,
    pub isbn: String,
    pub title: String,
    pub authors: Vec<String>,
    pub translator: String,
    pub publisher: String,
    pub publication_date: String,
    pub synopsis: String,
    pub language: String,
    pub cover_jpeg_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ExtractedFigure {
    pub id: String,
    pub href: String,
    pub top: f32,
    pub jpeg_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PdfWordBox {
    pub text: String,
    pub font_name: String,
    pub x0: f32,
    pub top: f32,
    pub x1: f32,
    pub bottom: f32,
    pub font_size: f32,
    pub is_italic: bool,
    pub is_bold: bool,
    pub is_monospace: bool,
}

#[derive(Debug, Clone)]
pub struct RawPdfPage {
    pub page_index: usize,
    pub width: f32,
    pub height: f32,
    pub words: Vec<PdfWordBox>,
    pub figures: Vec<ExtractedFigure>,
}

#[derive(Debug, Clone)]
pub struct ReconstructedLine {
    pub raw_text: String,
    pub html_text: String,
    pub x0: f32,
    pub top: f32,
    pub max_font_size: f32,
    pub starts_with_dropcap: bool,
    pub is_heading_font: bool,
    pub is_all_monospace: bool,
}

#[derive(Debug, Clone)]
pub struct FootnoteEntry {
    pub id_num: usize,
    pub marker: String,
    pub content_html: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalloutKind {
    Note,
    Tip,
    Warning,
}

#[derive(Debug, Clone)]
pub enum ChapterElement {
    SubheadingH2 {
        id: String,
        number_prefix: String,
        title: String,
    },
    SubheadingH3 {
        id: String,
        number_prefix: String,
        title: String,
    },
    SubheadingH4 {
        id: String,
        number_prefix: String,
        title: String,
    },
    StandardParagraph(String),
    DialogueParagraph(String),
    DropCapParagraph(char, String),
    CodeBlock(String),
    CalloutBox {
        kind: CalloutKind,
        title: String,
        body_html: String,
    },
    DataTable {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    FigureImage(String),
    FigureCaption(String),
}

#[derive(Debug, Clone)]
pub struct Chapter {
    pub index: usize,
    pub number_prefix: String,
    pub title: String,
    pub filename: String,
    pub elements: Vec<ChapterElement>,
    pub footnotes: Vec<FootnoteEntry>,
    pub subtopics: Vec<TocNode>,
}

pub fn slugify_anchor_id(raw_title: &str, fallback_idx: usize) -> String {
    let mut slug = String::with_capacity(raw_title.len());
    let mut last_was_dash = false;

    for ch in raw_title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if (ch.is_whitespace() || ch == '-' || ch == '_' || ch == '/')
            && !slug.is_empty()
            && !last_was_dash
        {
            slug.push('-');
            last_was_dash = true;
        }
    }

    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        format!("sec-{}", fallback_idx)
    } else if trimmed.chars().next().unwrap().is_ascii_digit() {
        format!("s-{}", trimmed)
    } else {
        trimmed
    }
}

pub fn insert_subtopic_node(
    tree: &mut Vec<TocNode>,
    level: u8,
    number_prefix: String,
    title: String,
    href: String,
) {
    let new_node = TocNode {
        number_prefix,
        title,
        href,
        children: Vec::new(),
    };

    match level {
        2 => tree.push(new_node),
        3 => {
            if let Some(last_h2) = tree.last_mut() {
                last_h2.children.push(new_node);
            } else {
                tree.push(new_node);
            }
        }
        _ => {
            if let Some(last_h2) = tree.last_mut() {
                if let Some(last_h3) = last_h2.children.last_mut() {
                    last_h3.children.push(new_node);
                } else {
                    last_h2.children.push(new_node);
                }
            } else {
                tree.push(new_node);
            }
        }
    }
}

pub struct MediaWikiExtractor {
    pub root_url: String,
    pub max_depth: usize,
    pub max_chapters: usize,
    pub default_metadata: BookMetadata,
}

impl MediaWikiExtractor {
    pub fn extract_origin(url: &str) -> String {
        if let Some(scheme_end) = url.find("://") {
            let rest = &url[scheme_end + 3..];
            if let Some(slash_idx) = rest.find('/') {
                return url[..scheme_end + 3 + slash_idx].to_string();
            }
        }
        url.trim_end_matches('/').to_string()
    }

    pub fn resolve_valid_wiki_article(origin: &str, href: &str) -> Option<String> {
        let clean = href.split('#').next()?.trim();
        if clean.is_empty() || clean.contains('?') {
            return None;
        }

        let path = if clean.starts_with(origin) {
            &clean[origin.len()..]
        } else if clean.starts_with("/title/") || clean.starts_with("/wiki/") {
            clean
        } else {
            return None;
        };

        let article_slug = path
            .strip_prefix("/title/")
            .or_else(|| path.strip_prefix("/wiki/"))?;

        if article_slug.is_empty() {
            return None;
        }

        let blocked_prefixes = [
            "Special:",
            "Talk:",
            "User:",
            "User_talk:",
            "ArchWiki:",
            "ArchWiki_talk:",
            "Help:",
            "File:",
            "Template:",
            "Category:",
            "MediaWiki:",
        ];
        if blocked_prefixes.iter().any(|p| article_slug.starts_with(p)) {
            return None;
        }

        if article_slug.contains("_(") && article_slug.ends_with(')') {
            return None;
        }

        Some(format!("{}{}", origin, path))
    }

    pub fn extract_archwiki_pre_block(pre_node: scraper::ElementRef) -> String {
        let mut out = String::new();
        for child in pre_node.children() {
            if let Some(el) = scraper::ElementRef::wrap(child) {
                let tag = el.value().name();
                let inner_txt = escape_xhtml_fast(&el.text().collect::<String>());
                match tag {
                    "i" | "em" => out.push_str(&format!("<em>{}</em>", inner_txt)),
                    "b" | "strong" => out.push_str(&format!("<strong>{}</strong>", inner_txt)),
                    _ => out.push_str(&inner_txt),
                }
            } else if let Some(txt) = child.value().as_text() {
                out.push_str(&escape_xhtml_fast(txt));
            }
        }
        out.trim().to_string()
    }

    pub fn extract_wikitable(table_node: scraper::ElementRef) -> Option<ChapterElement> {
        use scraper::Selector;
        let tr_sel = Selector::parse("tr").ok()?;
        let th_sel = Selector::parse("th").ok()?;
        let td_sel = Selector::parse("td").ok()?;

        let mut headers = Vec::new();
        let mut rows = Vec::new();

        for tr in table_node.select(&tr_sel) {
            let ths: Vec<String> = tr
                .select(&th_sel)
                .map(|th| escape_xhtml_fast(&normalize_whitespace(&th.text().collect::<String>())))
                .collect();
            let tds: Vec<String> = tr
                .select(&td_sel)
                .map(|td| escape_xhtml_fast(&normalize_whitespace(&td.text().collect::<String>())))
                .collect();

            if !ths.is_empty() && tds.is_empty() && headers.is_empty() {
                headers = ths;
            } else if !tds.is_empty() {
                let mut row = Vec::new();
                if !ths.is_empty() {
                    row.extend(ths);
                }
                row.extend(tds);
                rows.push(row);
            }
        }

        if headers.is_empty() && rows.is_empty() {
            None
        } else {
            Some(ChapterElement::DataTable { headers, rows })
        }
    }

    fn extract_heading_anchor_id(el: scraper::ElementRef, raw_title: &str, seq: usize) -> String {
        use scraper::Selector;
        if let Some(id_attr) = el.value().attr("id") {
            let s = slugify_anchor_id(id_attr, seq);
            if !s.is_empty() {
                return s;
            }
        }
        if let Ok(span_sel) = Selector::parse("span.mw-headline[id], span[id]") {
            if let Some(span) = el.select(&span_sel).next() {
                if let Some(id_attr) = span.value().attr("id") {
                    return slugify_anchor_id(id_attr, seq);
                }
            }
        }
        slugify_anchor_id(raw_title, seq)
    }

    pub fn parse_mediawiki_article(
        origin: &str,
        html_doc: &scraper::Html,
        chapter_index: usize,
        chapter_filename: &str,
    ) -> (String, Vec<ChapterElement>, Vec<TocNode>, Vec<String>) {
        use scraper::Selector;

        let title_sel = Selector::parse("h1#firstHeading, .mw-page-title-main, h1").unwrap();
        let title = html_doc
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("Article {}", chapter_index));

        let content_sel =
            Selector::parse("#mw-content-text .mw-parser-output, #mw-content-text").unwrap();
        let link_sel = Selector::parse("a[href]").unwrap();

        let mut discovered_links = Vec::new();
        let mut elements = Vec::new();
        let mut subtopics: Vec<TocNode> = Vec::new();

        let mut h2_num = 0usize;
        let mut h3_num = 0usize;
        let mut h4_num = 0usize;
        let mut anchor_seq = 1usize;

        if let Some(content_root) = html_doc.select(&content_sel).next() {
            for a_el in content_root.select(&link_sel) {
                if let Some(href) = a_el.value().attr("href") {
                    if let Some(valid_url) = Self::resolve_valid_wiki_article(origin, href) {
                        discovered_links.push(valid_url);
                    }
                }
            }

            for child in content_root.children() {
                let Some(el) = scraper::ElementRef::wrap(child) else {
                    continue;
                };
                let tag = el.value().name();
                let classes = el.value().attr("class").unwrap_or("");

                if classes.contains("toc")
                    || classes.contains("mw-indicators")
                    || classes.contains("noprint")
                    || el.value().attr("id") == Some("toc")
                {
                    continue;
                }

                if classes.contains("archwiki-template-box") {
                    let kind = if classes.contains("archwiki-template-box-warning") {
                        CalloutKind::Warning
                    } else if classes.contains("archwiki-template-box-tip") {
                        CalloutKind::Tip
                    } else {
                        CalloutKind::Note
                    };
                    let box_title = match kind {
                        CalloutKind::Warning => "Warning",
                        CalloutKind::Tip => "Tip",
                        CalloutKind::Note => "Note",
                    };
                    let body = escape_xhtml_fast(&normalize_whitespace(
                        &el.text().collect::<String>(),
                    ));
                    if !body.is_empty() {
                        elements.push(ChapterElement::CalloutBox {
                            kind,
                            title: String::from(box_title),
                            body_html: body,
                        });
                    }
                    continue;
                }

                match tag {
                    "h2" => {
                        let h_txt = el
                            .text()
                            .collect::<String>()
                            .replace("[edit]", "")
                            .trim()
                            .to_string();
                        if !h_txt.is_empty() && h_txt != "Contents" && h_txt != "Navigation" {
                            h2_num += 1;
                            h3_num = 0;
                            h4_num = 0;
                            let num_prefix = format!("{}.{}", chapter_index, h2_num);
                            let id = Self::extract_heading_anchor_id(el, &h_txt, anchor_seq);
                            anchor_seq += 1;
                            let safe_t = escape_xhtml_fast(&h_txt);

                            insert_subtopic_node(
                                &mut subtopics,
                                2,
                                num_prefix.clone(),
                                h_txt,
                                format!("{}#{}", chapter_filename, id),
                            );
                            elements.push(ChapterElement::SubheadingH2 {
                                id,
                                number_prefix: num_prefix,
                                title: safe_t,
                            });
                        }
                    }
                    "h3" => {
                        let h_txt = el
                            .text()
                            .collect::<String>()
                            .replace("[edit]", "")
                            .trim()
                            .to_string();
                        if !h_txt.is_empty() {
                            if h2_num == 0 {
                                h2_num = 1;
                            }
                            h3_num += 1;
                            h4_num = 0;
                            let num_prefix = format!("{}.{}.{}", chapter_index, h2_num, h3_num);
                            let id = Self::extract_heading_anchor_id(el, &h_txt, anchor_seq);
                            anchor_seq += 1;
                            let safe_t = escape_xhtml_fast(&h_txt);

                            insert_subtopic_node(
                                &mut subtopics,
                                3,
                                num_prefix.clone(),
                                h_txt,
                                format!("{}#{}", chapter_filename, id),
                            );
                            elements.push(ChapterElement::SubheadingH3 {
                                id,
                                number_prefix: num_prefix,
                                title: safe_t,
                            });
                        }
                    }
                    "h4" | "h5" => {
                        let h_txt = el
                            .text()
                            .collect::<String>()
                            .replace("[edit]", "")
                            .trim()
                            .to_string();
                        if !h_txt.is_empty() {
                            if h2_num == 0 {
                                h2_num = 1;
                            }
                            if h3_num == 0 {
                                h3_num = 1;
                            }
                            h4_num += 1;
                            let num_prefix =
                                format!("{}.{}.{}.{}", chapter_index, h2_num, h3_num, h4_num);
                            let id = Self::extract_heading_anchor_id(el, &h_txt, anchor_seq);
                            anchor_seq += 1;
                            let safe_t = escape_xhtml_fast(&h_txt);

                            insert_subtopic_node(
                                &mut subtopics,
                                4,
                                num_prefix.clone(),
                                h_txt,
                                format!("{}#{}", chapter_filename, id),
                            );
                            elements.push(ChapterElement::SubheadingH4 {
                                id,
                                number_prefix: num_prefix,
                                title: safe_t,
                            });
                        }
                    }
                    "pre" => {
                        let code_html = Self::extract_archwiki_pre_block(el);
                        if !code_html.is_empty() {
                            elements.push(ChapterElement::CodeBlock(code_html));
                        }
                    }
                    "table" if classes.contains("wikitable") => {
                        if let Some(tbl) = Self::extract_wikitable(el) {
                            elements.push(tbl);
                        }
                    }
                    "p" | "ul" | "ol" | "dl" => {
                        let txt = normalize_whitespace(&el.text().collect::<String>());
                        if !txt.is_empty() {
                            elements.push(ChapterElement::StandardParagraph(escape_xhtml_fast(
                                &txt,
                            )));
                        }
                    }
                    _ => {}
                }
            }
        }

        (title, elements, subtopics, discovered_links)
    }
}

impl ContentSourceExtractor for MediaWikiExtractor {
    fn extract_payload(&self) -> Result<ExtractedBookPayload, Box<dyn std::error::Error>> {
        use scraper::Html;

        let client = reqwest::blocking::Client::builder()
            .user_agent("pdf2epub-rs/0.7.0 (+https://www.w3.org/TR/epub-33/)")
            .build()?;

        let clean_root = self
            .root_url
            .split('#')
            .next()
            .unwrap_or(&self.root_url)
            .to_string();
        let origin = Self::extract_origin(&clean_root);

        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        let mut visited: HashSet<String> = HashSet::new();

        queue.push_back((clean_root.clone(), 0));
        visited.insert(clean_root);

        let mut chapters = Vec::new();

        while let Some((current_url, depth)) = queue.pop_front() {
            if chapters.len() >= self.max_chapters {
                break;
            }

            let Ok(resp) = client.get(&current_url).send() else {
                continue;
            };
            let Ok(html_str) = resp.text() else {
                continue;
            };

            let doc = Html::parse_document(&html_str);
            let ch_idx = chapters.len() + 1;
            let fname = format!("chapter_{:03}.xhtml", ch_idx);
            let (title, elements, subtopics, child_links) =
                Self::parse_mediawiki_article(&origin, &doc, ch_idx, &fname);

            if !elements.is_empty() {
                chapters.push(Chapter {
                    index: ch_idx,
                    number_prefix: format!("{}", ch_idx),
                    title,
                    filename: fname,
                    elements,
                    footnotes: Vec::new(),
                    subtopics,
                });
            }

            if depth < self.max_depth {
                for next_url in child_links {
                    if visited.len() >= self.max_chapters * 2 {
                        break;
                    }
                    if visited.insert(next_url.clone()) {
                        queue.push_back((next_url, depth + 1));
                    }
                }
            }
        }

        Ok(ExtractedBookPayload {
            metadata: self.default_metadata.clone(),
            chapters,
            figures: Vec::new(),
        })
    }
}

pub struct WebSiteExtractor {
    pub start_url: String,
    pub crawl_docs_toc: bool,
    pub toc_selector: String,
    pub default_metadata: BookMetadata,
}

impl WebSiteExtractor {
    pub fn normalize_base_url(raw_url: &str) -> String {
        let without_fragment = raw_url.split('#').next().unwrap_or(raw_url).trim();
        if without_fragment.ends_with(".html") || without_fragment.ends_with(".htm") {
            if let Some(last_slash) = without_fragment.rfind('/') {
                return without_fragment[..=last_slash].to_string();
            }
        }
        if without_fragment.ends_with('/') {
            without_fragment.to_string()
        } else {
            format!("{}/", without_fragment)
        }
    }

    pub fn detect_dom_profile(url: &str) -> &'static str {
        if url.contains("developer.mozilla.org") {
            "mdn-yari"
        } else if url.contains("git-scm.com/book") {
            "progit-asciidoc"
        } else if url.contains("docs.python.org") || url.contains("readthedocs") {
            "sphinx-docs"
        } else if url.contains("go.dev/doc/effective_go") {
            "godev-spec"
        } else {
            "mdbook-default"
        }
    }

    pub fn effective_toc_selector(&self) -> String {
        let profile = Self::detect_dom_profile(&self.start_url);
        let engine_sel = match profile {
            "mdn-yari" => {
                "nav.sidebar ol li a, aside.sidebar ol li a, details ol li a, \
                 article.main-page-content ul li a, .sidebar-inner a[href*=\"/Guide/\"]"
            }
            "progit-asciidoc" => {
                "ol.book-toc li a, div#book-chapters ol li a, #content ol.book-toc a, \
                 ul.book-toc li a, a[href*=\"/book/\"]"
            }
            "sphinx-docs" => {
                "div.toctree-wrapper li.toctree-l1 > a, div.toctree-wrapper li.toctree-l2 > a, \
                 div.toctree-wrapper li.toctree-l3 > a, div.sphinxsidebar a.reference.internal"
            }
            _ => "nav#sidebar ol.chapter li a, ol.chapter li a, nav.sidebar a, aside a",
        };
        if self.toc_selector.trim().is_empty() {
            engine_sel.to_string()
        } else {
            format!("{}, {}", engine_sel, self.toc_selector)
        }
    }

    pub fn is_within_book_scope(base_dir_url: &str, candidate_url: &str) -> bool {
        let base_clean = base_dir_url.trim_end_matches('/');
        if candidate_url.starts_with(base_clean) {
            return true;
        }
        if base_clean.contains("developer.mozilla.org/en-US/docs/Web/JavaScript")
            && candidate_url.starts_with("https://developer.mozilla.org/en-US/docs/Web/JavaScript/")
        {
            return true;
        }
        false
    }

    pub fn resolve_chapter_url(base_dir_url: &str, href: &str) -> Option<String> {
        let clean_href = href.split('#').next()?.trim();
        if clean_href.is_empty()
            || clean_href.starts_with("javascript:")
            || clean_href.starts_with("mailto:")
            || clean_href.ends_with("print.html")
            || clean_href.ends_with(".png")
            || clean_href.ends_with(".jpg")
            || clean_href.ends_with(".svg")
            || clean_href.ends_with(".pdf")
            || clean_href.ends_with(".epub")
        {
            return None;
        }

        if clean_href.starts_with("http://") || clean_href.starts_with("https://") {
            if Self::is_within_book_scope(base_dir_url, clean_href) {
                return Some(clean_href.to_string());
            }
            return None;
        }

        if clean_href.starts_with('/') {
            if let Some(scheme_idx) = base_dir_url.find("://") {
                let after_scheme = &base_dir_url[scheme_idx + 3..];
                let origin = if let Some(slash_idx) = after_scheme.find('/') {
                    &base_dir_url[..scheme_idx + 3 + slash_idx]
                } else {
                    base_dir_url
                };
                let resolved = format!("{}{}", origin, clean_href);
                if Self::is_within_book_scope(base_dir_url, &resolved) {
                    return Some(resolved);
                }
            }
            return None;
        }

        let trimmed_rel = clean_href.trim_start_matches("./");
        let resolved = format!("{}{}", base_dir_url, trimmed_rel);
        if Self::is_within_book_scope(base_dir_url, &resolved) {
            Some(resolved)
        } else {
            None
        }
    }

    pub fn clean_code_toolbar_prefix(raw_code: &str) -> String {
        let mut lines: Vec<&str> = raw_code.lines().collect();
        if let Some(first) = lines.first() {
            let f = first.trim();
            if matches!(
                f,
                "js Copy"
                    | "html Copy"
                    | "css Copy"
                    | "bash Copy"
                    | "json Copy"
                    | "diff Copy"
                    | "js Copy Play"
                    | "html Copy Play"
                    | "css Copy Play"
                    | "Copy"
            ) {
                lines.remove(0);
            }
        }
        lines.join("\n").trim().to_string()
    }

    pub fn extract_chapter_number_prefix(a_text: &str, url: &str, fallback_idx: usize) -> String {
        let first_tok = a_text.split_whitespace().next().unwrap_or("");
        if first_tok.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            let cleaned = first_tok.trim_end_matches('.');
            if !cleaned.is_empty() {
                return cleaned.to_string();
            }
        }

        if let Some(fname) = url.rsplit('/').next() {
            if let Some(rest) = fname.strip_prefix("ch") {
                let parts: Vec<&str> = rest.split('-').collect();
                if parts.len() >= 2 {
                    if let (Ok(ch_num), Ok(sub_num)) =
                        (parts[0].parse::<usize>(), parts[1].parse::<usize>())
                    {
                        if sub_num == 0 {
                            return format!("{}", ch_num);
                        } else {
                            return format!("{}.{}", ch_num, sub_num);
                        }
                    }
                }
            }
        }

        format!("{}", fallback_idx)
    }

    pub fn extract_opengraph_metadata(&self, html_doc: &scraper::Html) -> BookMetadata {
        use scraper::Selector;
        let mut meta = self.default_metadata.clone();

        let select_meta_content = |selector_str: &str| -> Option<String> {
            let sel = Selector::parse(selector_str).ok()?;
            html_doc
                .select(&sel)
                .next()?
                .value()
                .attr("content")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        };

        if let Some(og_title) = select_meta_content(r#"meta[property="og:title"]"#).or_else(|| {
            let menu_sel = Selector::parse("h1.menu-title, title").ok()?;
            Some(
                html_doc
                    .select(&menu_sel)
                    .next()?
                    .text()
                    .collect::<String>()
                    .trim()
                    .to_string(),
            )
        }) {
            meta.title = og_title;
        }

        if let Some(og_desc) = select_meta_content(r#"meta[property="og:description"]"#)
            .or_else(|| select_meta_content(r#"meta[name="description"]"#))
        {
            meta.synopsis = og_desc;
        }

        if let Some(author) = select_meta_content(r#"meta[name="author"]"#)
            .or_else(|| select_meta_content(r#"meta[property="article:author"]"#))
        {
            meta.authors = vec![author];
        }

        if let Some(og_image_url) = select_meta_content(r#"meta[property="og:image"]"#) {
            if let Ok(resp) = reqwest::blocking::get(&og_image_url) {
                if let Ok(bytes) = resp.bytes() {
                    meta.cover_jpeg_bytes = Some(bytes.to_vec());
                }
            }
        }

        meta
    }

    pub fn extract_mdbook_code_block(pre_node: scraper::ElementRef) -> String {
        use scraper::Selector;
        let code_sel = Selector::parse("code").unwrap();

        let mut ferris_badge = "";
        if let Some(code_el) = pre_node.select(&code_sel).next() {
            let classes = code_el.value().attr("class").unwrap_or("");
            if classes.contains("does_not_compile") {
                ferris_badge = "// [Ferris: This code does not compile!]\n";
            } else if classes.contains("panics") {
                ferris_badge = "// [Ferris: This code panics!]\n";
            } else if classes.contains("not_desired_behavior") {
                ferris_badge = "// [Ferris: This code does not produce the desired behavior.]\n";
            }
        }

        let mut clean_code = String::from(ferris_badge);
        let target_node = pre_node.select(&code_sel).next().unwrap_or(pre_node);

        for child in target_node.children() {
            if let Some(el) = scraper::ElementRef::wrap(child) {
                let cl = el.value().attr("class").unwrap_or("");
                if cl.split_whitespace().any(|c| c == "boring") {
                    continue;
                }
                clean_code.push_str(&el.text().collect::<String>());
            } else if let Some(txt) = child.value().as_text() {
                clean_code.push_str(txt);
            }
        }

        clean_code.trim().to_string()
    }

    pub fn has_nested_block_children(node: scraper::ElementRef) -> bool {
        use scraper::Selector;
        let inner_sel = Selector::parse("p, pre, table, dl, ul, ol, blockquote").unwrap();
        node.select(&inner_sel).next().is_some()
    }

    pub fn is_inside_callout_or_table(node: scraper::ElementRef) -> bool {
        let mut curr = node.parent();
        while let Some(parent) = curr {
            if let Some(el) = scraper::ElementRef::wrap(parent) {
                let tag = el.value().name();
                if tag == "table" || tag == "thead" || tag == "tbody" || tag == "tr" || tag == "td" || tag == "th" {
                    return true;
                }
                if tag == "div" || tag == "aside" {
                    let cl = el.value().attr("class").unwrap_or("");
                    if cl.contains("notecard")
                        || cl.contains("admonitionblock")
                        || cl.contains("admonition")
                        || cl.split_whitespace().any(|c| c == "note" || c == "warning" || c == "tip")
                    {
                        return true;
                    }
                }
            }
            curr = parent.parent();
        }
        false
    }

    pub fn extract_clean_article_elements(
        &self,
        html_doc: &scraper::Html,
        chapter_index: usize,
        chapter_prefix: &str,
        chapter_filename: &str,
    ) -> (String, Vec<ChapterElement>, Vec<TocNode>) {
        use scraper::Selector;

        let title_sel = Selector::parse(
            "#content main h1, article.main-page-content h1, div#main div#content h1, \
             div.body[role=\"main\"] h1, article h1, main h1, h1",
        )
        .unwrap();
        let raw_h1 = html_doc
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().replace('¶', "").trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("Chapter {}", chapter_index));

        let chapter_title = if !chapter_prefix.is_empty()
            && !raw_h1.starts_with(chapter_prefix)
            && chapter_prefix.chars().next().map_or(false, |c| c.is_ascii_digit())
        {
            format!("{}. {}", chapter_prefix, raw_h1)
        } else {
            raw_h1
        };

        let block_sel = Selector::parse(
            "#content main h2, #content main h3, #content main h4, #content main p, \
             #content main li, #content main dt, #content main dd, #content main blockquote, \
             #content main pre, #content main table, #content main figcaption, #content main span.caption, \
             article.main-page-content h2, article.main-page-content h3, article.main-page-content h4, \
             article.main-page-content p, article.main-page-content li, article.main-page-content dt, article.main-page-content dd, \
             article.main-page-content blockquote, article.main-page-content pre, article.main-page-content table, \
             article.main-page-content div.notecard, article.main-page-content figcaption, \
             div#main div#content h2, div#main div#content h3, div#main div#content h4, \
             div#main div#content p, div#main div#content li, div#main div#content dt, div#main div#content dd, \
             div#main div#content blockquote, div#main div#content pre, div#main div#content table, \
             div#main div#content div.admonitionblock, div#main div#content div.imageblock div.title, \
             div.body[role=\"main\"] h2, div.body[role=\"main\"] h3, div.body[role=\"main\"] h4, \
             div.body[role=\"main\"] p, div.body[role=\"main\"] li, div.body[role=\"main\"] dt, div.body[role=\"main\"] dd, \
             div.body[role=\"main\"] blockquote, div.body[role=\"main\"] pre, div.body[role=\"main\"] table, \
             div.body[role=\"main\"] div.admonition, div.body[role=\"main\"] div.note, div.body[role=\"main\"] div.warning, \
             article h2, article h3, article h4, article p, article li, article dt, article dd, article blockquote, \
             article pre, article table, article figcaption, article div.notecard, article div.admonitionblock",
        )
        .unwrap();

        let mut elements = Vec::new();
        let mut subtopics: Vec<TocNode> = Vec::new();

        let mut h2_idx = 0usize;
        let mut h3_idx = 0usize;
        let mut h4_idx = 0usize;
        let mut anchor_seq = 1usize;

        for node in html_doc.select(&block_sel) {
            let tag = node.value().name();

            if tag == "pre" {
                if Self::is_inside_callout_or_table(node) {
                    continue;
                }
                let raw_code = Self::extract_mdbook_code_block(node);
                let code_txt = Self::clean_code_toolbar_prefix(&raw_code);
                if !code_txt.is_empty() {
                    elements.push(ChapterElement::CodeBlock(escape_xhtml_fast(&code_txt)));
                }
                continue;
            }

            if tag == "div" {
                let classes = node.value().attr("class").unwrap_or("");
                if classes.contains("notecard")
                    || classes.contains("admonitionblock")
                    || classes.contains("admonition")
                    || classes.split_whitespace().any(|c| c == "note" || c == "warning" || c == "tip")
                {
                    let callout_txt = normalize_whitespace(&node.text().collect::<String>());
                    if !callout_txt.is_empty() {
                        let (kind, title) = if classes.contains("warning") || classes.contains("caution") {
                            (CalloutKind::Warning, "Warning")
                        } else if classes.contains("tip") {
                            (CalloutKind::Tip, "Tip")
                        } else {
                            (CalloutKind::Note, "Note")
                        };
                        elements.push(ChapterElement::CalloutBox {
                            kind,
                            title: String::from(title),
                            body_html: escape_xhtml_fast(&callout_txt),
                        });
                    }
                    continue;
                }
                if classes.contains("title") {
                    let cap_txt = normalize_whitespace(&node.text().collect::<String>());
                    if !cap_txt.is_empty() {
                        elements.push(ChapterElement::FigureCaption(escape_xhtml_fast(&cap_txt)));
                    }
                    continue;
                }
                continue;
            }

            if tag == "table" {
                let classes = node.value().attr("class").unwrap_or("");
                if classes.contains("admonition") {
                    continue;
                }
                if let Some(tbl) = MediaWikiExtractor::extract_wikitable(node) {
                    elements.push(tbl);
                }
                continue;
            }

            if (tag == "p" || tag == "li" || tag == "dt" || tag == "dd" || tag == "blockquote")
                && Self::is_inside_callout_or_table(node)
            {
                continue;
            }

            if (tag == "li" || tag == "dd" || tag == "blockquote")
                && Self::has_nested_block_children(node)
            {
                continue;
            }

            let text = node
                .text()
                .collect::<String>()
                .replace('¶', "")
                .replace("[edit]", "");
            let trimmed = text.trim();
            if trimmed.is_empty() {
                continue;
            }

            let raw_id = node
                .value()
                .attr("id")
                .map(|s| slugify_anchor_id(s, anchor_seq))
                .unwrap_or_else(|| slugify_anchor_id(trimmed, anchor_seq));
            anchor_seq += 1;

            match tag {
                "h2" => {
                    h2_idx += 1;
                    h3_idx = 0;
                    h4_idx = 0;
                    let num_prefix = format!("{}.{}", chapter_prefix, h2_idx);
                    insert_subtopic_node(
                        &mut subtopics,
                        2,
                        num_prefix.clone(),
                        trimmed.to_string(),
                        format!("{}#{}", chapter_filename, raw_id),
                    );
                    elements.push(ChapterElement::SubheadingH2 {
                        id: raw_id,
                        number_prefix: num_prefix,
                        title: escape_xhtml_fast(trimmed),
                    });
                }
                "h3" => {
                    if h2_idx == 0 {
                        h2_idx = 1;
                    }
                    h3_idx += 1;
                    h4_idx = 0;
                    let num_prefix = format!("{}.{}.{}", chapter_prefix, h2_idx, h3_idx);
                    insert_subtopic_node(
                        &mut subtopics,
                        3,
                        num_prefix.clone(),
                        trimmed.to_string(),
                        format!("{}#{}", chapter_filename, raw_id),
                    );
                    elements.push(ChapterElement::SubheadingH3 {
                        id: raw_id,
                        number_prefix: num_prefix,
                        title: escape_xhtml_fast(trimmed),
                    });
                }
                "h4" => {
                    if h2_idx == 0 {
                        h2_idx = 1;
                    }
                    if h3_idx == 0 {
                        h3_idx = 1;
                    }
                    h4_idx += 1;
                    let num_prefix =
                        format!("{}.{}.{}.{}", chapter_prefix, h2_idx, h3_idx, h4_idx);
                    insert_subtopic_node(
                        &mut subtopics,
                        4,
                        num_prefix.clone(),
                        trimmed.to_string(),
                        format!("{}#{}", chapter_filename, raw_id),
                    );
                    elements.push(ChapterElement::SubheadingH4 {
                        id: raw_id,
                        number_prefix: num_prefix,
                        title: escape_xhtml_fast(trimmed),
                    });
                }
                "figcaption" | "span" => {
                    elements.push(ChapterElement::FigureCaption(escape_xhtml_fast(
                        &normalize_whitespace(trimmed),
                    )))
                }
                "dt" => {
                    elements.push(ChapterElement::StandardParagraph(format!(
                        "<strong><code>{}</code></strong>",
                        escape_xhtml_fast(&normalize_whitespace(trimmed))
                    )));
                }
                "dd" => {
                    elements.push(ChapterElement::StandardParagraph(format!(
                        "— {}",
                        escape_xhtml_fast(&normalize_whitespace(trimmed))
                    )));
                }
                "li" => {
                    elements.push(ChapterElement::StandardParagraph(format!(
                        "• {}",
                        escape_xhtml_fast(&normalize_whitespace(trimmed))
                    )));
                }
                _ => elements.push(ChapterElement::StandardParagraph(escape_xhtml_fast(
                    &normalize_whitespace(trimmed),
                ))),
            }
        }

        (chapter_title, elements, subtopics)
    }

    pub fn crawl_docs_toc_bfs(
        &self,
        client: &reqwest::blocking::Client,
        base_dir_url: &str,
        root_doc: &scraper::Html,
    ) -> Vec<(String, String)> {
        use scraper::{Html, Selector};

        let mut target_entries: Vec<(String, String)> = Vec::new();
        let mut seen_urls = HashSet::new();
        let combined_sel_str = self.effective_toc_selector();

        let Ok(toc_sel) = Selector::parse(&combined_sel_str) else {
            return vec![(base_dir_url.to_string(), String::from("1"))];
        };

        let mut level1_urls: Vec<String> = Vec::new();
        for a_el in root_doc.select(&toc_sel) {
            if let Some(href) = a_el.value().attr("href") {
                if let Some(resolved) = Self::resolve_chapter_url(base_dir_url, href) {
                    if seen_urls.insert(resolved.clone()) {
                        let a_text = a_el.text().collect::<String>();
                        let prefix = Self::extract_chapter_number_prefix(
                            &a_text,
                            &resolved,
                            target_entries.len() + 1,
                        );
                        target_entries.push((resolved.clone(), prefix));
                        level1_urls.push(resolved);
                    }
                }
            }
        }

        if level1_urls.len() <= 20 {
            for l1_url in level1_urls.iter().take(16) {
                if let Ok(resp) = client.get(l1_url).send() {
                    if let Ok(sub_html) = resp.text() {
                        let sub_doc = Html::parse_document(&sub_html);
                        let sub_base = Self::normalize_base_url(l1_url);
                        for a_el in sub_doc.select(&toc_sel) {
                            if let Some(href) = a_el.value().attr("href") {
                                if let Some(resolved) = Self::resolve_chapter_url(&sub_base, href) {
                                    if Self::is_within_book_scope(base_dir_url, &resolved)
                                        && seen_urls.insert(resolved.clone())
                                    {
                                        let a_text = a_el.text().collect::<String>();
                                        let prefix = Self::extract_chapter_number_prefix(
                                            &a_text,
                                            &resolved,
                                            target_entries.len() + 1,
                                        );
                                        target_entries.push((resolved, prefix));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if target_entries.is_empty() {
            target_entries.push((base_dir_url.to_string(), String::from("1")));
        }

        target_entries
    }
}

impl ContentSourceExtractor for WebSiteExtractor {
    fn extract_payload(&self) -> Result<ExtractedBookPayload, Box<dyn std::error::Error>> {
        use scraper::Html;

        let client = reqwest::blocking::Client::builder()
            .user_agent("pdf2epub-rs/0.9.0 (+https://www.w3.org/TR/epub-33/)")
            .build()?;

        let base_dir_url = Self::normalize_base_url(&self.start_url);
        let root_html_str = client.get(&self.start_url).send()?.text()?;
        let root_doc = Html::parse_document(&root_html_str);
        let metadata = self.extract_opengraph_metadata(&root_doc);

        let target_entries = if self.crawl_docs_toc {
            self.crawl_docs_toc_bfs(&client, &base_dir_url, &root_doc)
        } else {
            vec![(self.start_url.clone(), String::from("1"))]
        };

        let fetched_pages: Vec<(usize, String, String)> = target_entries
            .par_iter()
            .enumerate()
            .filter_map(|(idx, (url, prefix))| {
                let html = client.get(url).send().ok()?.text().ok()?;
                Some((idx + 1, prefix.clone(), html))
            })
            .collect();

        let mut chapters = Vec::with_capacity(fetched_pages.len());
        for (ch_num, ch_prefix, page_html) in fetched_pages {
            let doc = Html::parse_document(&page_html);
            let fname = format!("chapter_{:03}.xhtml", ch_num);
            let (title, elements, subtopics) =
                self.extract_clean_article_elements(&doc, ch_num, &ch_prefix, &fname);
            if !elements.is_empty() {
                chapters.push(Chapter {
                    index: ch_num,
                    number_prefix: ch_prefix,
                    title,
                    filename: fname,
                    elements,
                    footnotes: Vec::new(),
                    subtopics,
                });
            }
        }

        Ok(ExtractedBookPayload {
            metadata,
            chapters,
            figures: Vec::new(),
        })
    }
}

pub struct SinglePageSpecExtractor {
    pub url: String,
    pub default_metadata: BookMetadata,
}

impl SinglePageSpecExtractor {
    pub fn parse_spec_document(&self, doc: &scraper::Html) -> Vec<Chapter> {
        use scraper::Selector;

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
                let code_txt = WebSiteExtractor::clean_code_toolbar_prefix(&node.text().collect::<String>());
                if !code_txt.trim().is_empty() {
                    current_elements.push(ChapterElement::CodeBlock(escape_xhtml_fast(code_txt.trim())));
                }
                continue;
            }

            if tag == "table" {
                if let Some(tbl) = MediaWikiExtractor::extract_wikitable(node) {
                    current_elements.push(tbl);
                }
                continue;
            }

            if (tag == "li" || tag == "dd" || tag == "blockquote")
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
        use scraper::Html;

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

pub fn repair_broken_cmap_cid(font_name: &str, cid: u16) -> Option<&'static str> {
    let fname_lower = font_name.to_lowercase();
    if fname_lower.contains("calibri") {
        match cid {
            0x014C => Some("ft"),
            0x019F => Some("ti"),
            0x01A1 => Some("tí"),
            _ => None,
        }
    } else if fname_lower.contains("lekton") {
        match cid {
            0x01EB => Some("C"),
            0x01EE => Some("Q"),
            0x01EF => Some("V"),
            0x01F0 => Some("Z"),
            0x01F1 => Some("c"),
            0x01F2 => Some("f"),
            0x01F3 => Some("g"),
            0x01F4 => Some("i"),
            0x01F5 => Some("j"),
            0x01F6 => Some("k"),
            0x01F7 => Some("l"),
            0x01F8 => Some("r"),
            0x01F9 => Some("t"),
            0x01FA => Some("v"),
            0x01FB => Some("y"),
            0x0205 => Some("="),
            _ => None,
        }
    } else {
        None
    }
}

fn chapter_heading_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^(?i)(CAP[ÍI]TULO\s+[0-9IVXLCDM]+|PARTE\s+[0-9IVXLCDM]+|PR[ÓO]LOGO|PREF[ÁA]CIO|EP[ÍI]LOGO|AP[ÊE]NDICE)",
        )
        .expect("Regex de capítulo inválida")
    })
}

fn caption_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(Figura|Figure|Quadro|Tabela|Exemplo|Equa[çc][ãa]o)\s+\d+")
            .expect("Regex de legenda inválida")
    })
}

fn terminal_punctuation_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"[\.\!\?\:\;"”»…\)]$|\.\.\.$"#)
            .expect("Regex de pontuação terminal inválida")
    })
}

fn footnote_line_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(\[?\d{1,3}\]?|[¹²³⁴⁵⁶⁷⁸⁹⁰]+)\s+(.+)$")
            .expect("Regex de nota de rodapé inválida")
    })
}

fn inline_footnote_ref_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"([a-zA-ZÀ-ÿ,\.])(\[\d{1,2}\]|\d{1,2})(\s|$)")
            .expect("Regex de referência inline de nota inválida")
    })
}

pub struct PdfProsePipeline {
    pub body_font_size: f32,
    pub margin_cutoff: f32,
}

impl PdfProsePipeline {
    pub fn from_pages(pages: &[RawPdfPage], margin_cutoff: f32) -> Self {
        let body_font_size = Self::detect_body_font_size(pages);
        Self {
            body_font_size,
            margin_cutoff,
        }
    }

    pub fn detect_body_font_size(pages: &[RawPdfPage]) -> f32 {
        let sample_slice = if pages.len() > 35 {
            &pages[15..35]
        } else {
            pages
        };

        let histogram: HashMap<u32, usize> = sample_slice
            .par_iter()
            .fold(HashMap::new, |mut acc, page| {
                for w in &page.words {
                    let bucket = (w.font_size * 10.0).round() as u32;
                    *acc.entry(bucket).or_insert(0) += w.text.chars().count();
                }
                acc
            })
            .reduce(HashMap::new, |mut a, b| {
                for (k, v) in b {
                    *a.entry(k).or_insert(0) += v;
                }
                a
            });

        histogram
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(bucket, _)| (bucket as f32) / 10.0)
            .unwrap_or(11.1)
    }

    pub fn preprocess_pages_parallel(
        &self,
        pages: &[RawPdfPage],
    ) -> Vec<(
        Vec<ReconstructedLine>,
        Vec<(String, String)>,
        Vec<ExtractedFigure>,
    )> {
        let body_size = self.body_font_size;
        let cutoff = self.margin_cutoff;

        pages
            .par_iter()
            .map(|page| {
                let top_limit = page.height * cutoff;
                let bottom_limit = page.height * (1.0 - cutoff);
                let footnote_zone_start = page.height * 0.75;
                let footnote_max_font = body_size * 0.85;

                let usable_words: Vec<&PdfWordBox> = page
                    .words
                    .iter()
                    .filter(|w| w.top >= top_limit && w.bottom <= bottom_limit)
                    .collect();

                let raw_lines = Self::group_words_into_lines(&usable_words, body_size);

                let mut body_lines = Vec::with_capacity(raw_lines.len());
                let mut page_footnotes: Vec<(String, String)> = Vec::new();

                for line in raw_lines {
                    if line.top >= footnote_zone_start && line.max_font_size <= footnote_max_font {
                        if let Some(caps) = footnote_line_regex().captures(line.raw_text.trim()) {
                            let marker =
                                caps[1].trim_matches(|c| c == '[' || c == ']').to_string();
                            let content = caps[2].trim().to_string();
                            page_footnotes.push((marker, content));
                            continue;
                        } else if let Some(last_fn) = page_footnotes.last_mut() {
                            last_fn.1.push(' ');
                            last_fn.1.push_str(line.raw_text.trim());
                            continue;
                        }
                    }
                    body_lines.push(line);
                }

                (body_lines, page_footnotes, page.figures.clone())
            })
            .collect()
    }

    fn group_words_into_lines(words: &[&PdfWordBox], body_font_size: f32) -> Vec<ReconstructedLine> {
        if words.is_empty() {
            return Vec::new();
        }

        let mut sorted = words.to_vec();
        sorted.sort_by(|a, b| {
            let y_cmp = a.top.partial_cmp(&b.top).unwrap_or(std::cmp::Ordering::Equal);
            if (a.top - b.top).abs() <= 2.8 {
                a.x0.partial_cmp(&b.x0).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                y_cmp
            }
        });

        let mut clusters: Vec<Vec<&PdfWordBox>> = Vec::new();
        for w in sorted {
            if let Some(last_cluster) = clusters.last_mut() {
                let cluster_y = last_cluster[0].top;
                if (w.top - cluster_y).abs() <= 2.8 {
                    last_cluster.push(w);
                    continue;
                }
            }
            clusters.push(vec![w]);
        }

        let mut lines: Vec<ReconstructedLine> = Vec::with_capacity(clusters.len());
        let mut pending_dropcap: Option<char> = None;

        for mut cluster in clusters {
            cluster.sort_by(|a, b| a.x0.partial_cmp(&b.x0).unwrap_or(std::cmp::Ordering::Equal));

            if cluster.len() == 1
                && cluster[0].text.trim().chars().count() == 1
                && cluster[0].font_size >= body_font_size * 1.80
                && !cluster[0].font_name.contains("MyriadPro")
            {
                pending_dropcap = cluster[0].text.trim().chars().next();
                continue;
            }

            let mut starts_with_dropcap = false;
            let mut max_font_size: f32 = 0.0;
            let x0 = cluster[0].x0;
            let top = cluster[0].top;
            let is_heading_font = cluster.iter().any(|w| w.font_name.contains("MyriadPro"));
            let is_all_monospace = cluster.iter().all(|w| w.is_monospace);

            let first_is_inline_dropcap = cluster.len() > 1
                && cluster[0].text.trim().chars().count() == 1
                && cluster[0].font_size >= body_font_size * 1.80
                && cluster[1].font_size <= body_font_size * 1.25
                && !is_heading_font;

            let mut raw_str = String::new();
            let mut html_str = String::new();

            for (idx, w) in cluster.iter().enumerate() {
                let clean_word = w.text.replace('\0', "");
                if clean_word.trim().is_empty() {
                    continue;
                }

                if first_is_inline_dropcap && idx == 0 {
                    starts_with_dropcap = true;
                    raw_str.push_str(clean_word.trim());
                    html_str.push_str(&escape_xhtml_fast(clean_word.trim()));
                    let gap = cluster[1].x0 - w.x1;
                    if gap > body_font_size * 0.45 {
                        raw_str.push(' ');
                        html_str.push(' ');
                    }
                    continue;
                }

                if w.font_size > max_font_size {
                    max_font_size = w.font_size;
                }

                if !raw_str.is_empty() && !(first_is_inline_dropcap && idx == 1) {
                    raw_str.push(' ');
                    html_str.push(' ');
                }
                raw_str.push_str(clean_word.trim());
                html_str.push_str(&Self::format_inline_styles(w, clean_word.trim()));
            }

            if let Some(dc) = pending_dropcap.take() {
                starts_with_dropcap = true;
                raw_str = format!("{}{}", dc, raw_str.trim_start());
                html_str = format!("{}{}", escape_xhtml_fast(&dc.to_string()), html_str.trim_start());
            }

            if max_font_size == 0.0 {
                max_font_size = body_font_size;
            }

            lines.push(ReconstructedLine {
                raw_text: raw_str,
                html_text: html_str,
                x0,
                top,
                max_font_size,
                starts_with_dropcap,
                is_heading_font,
                is_all_monospace,
            });
        }

        lines
    }

    fn format_inline_styles(w: &PdfWordBox, clean_text: &str) -> String {
        let escaped = escape_xhtml_fast(clean_text);
        if w.is_monospace {
            match (w.is_bold, w.is_italic) {
                (true, _) => format!("<code><strong>{}</strong></code>", escaped),
                (false, true) => format!("<code><em>{}</em></code>", escaped),
                (false, false) => format!("<code>{}</code>", escaped),
            }
        } else {
            match (w.is_bold, w.is_italic) {
                (true, true) => format!("<strong><em>{}</strong></em>", escaped),
                (true, false) => format!("<strong>{}</strong>", escaped),
                (false, true) => format!("<em>{}</em>", escaped),
                (false, false) => escaped,
            }
        }
    }

    pub fn extract_chapters(&self, pages: &[RawPdfPage]) -> Vec<Chapter> {
        let processed_pages = self.preprocess_pages_parallel(pages);

        let mut chapters: Vec<Chapter> = Vec::new();
        let mut current_title = String::from("Elementos Pré-Textuais");
        let mut current_elements: Vec<ChapterElement> = Vec::new();
        let mut current_footnotes: Vec<FootnoteEntry> = Vec::new();
        let mut current_subtopics: Vec<TocNode> = Vec::new();
        let mut chapter_index = 1usize;
        let mut footnote_counter = 1usize;
        let mut h2_counter = 0usize;
        let mut h3_counter = 0usize;

        let mut buffer_text = String::new();
        let mut buffer_is_dialogue = false;
        let mut buffer_dropcap: Option<char> = None;
        let mut code_buffer: Vec<String> = Vec::new();

        for (lines, page_footnotes, page_figures) in processed_pages {
            let mut page_fn_map: HashMap<String, usize> = HashMap::new();
            for (marker, raw_fn_text) in page_footnotes {
                let fn_id = footnote_counter;
                footnote_counter += 1;
                page_fn_map.insert(marker.clone(), fn_id);
                current_footnotes.push(FootnoteEntry {
                    id_num: fn_id,
                    marker,
                    content_html: escape_xhtml_fast(&raw_fn_text),
                });
            }

            let mut fig_idx = 0usize;

            for line in lines {
                while fig_idx < page_figures.len() && page_figures[fig_idx].top <= line.top {
                    Self::flush_paragraph_buffer(
                        &mut buffer_text,
                        &mut buffer_is_dialogue,
                        &mut buffer_dropcap,
                        &mut current_elements,
                    );
                    Self::flush_code_buffer(&mut code_buffer, &mut current_elements);
                    current_elements.push(ChapterElement::FigureImage(
                        page_figures[fig_idx].href.clone(),
                    ));
                    fig_idx += 1;
                }

                let trimmed_raw = line.raw_text.trim();
                let trimmed_html = line.html_text.trim();
                if trimmed_raw.is_empty() {
                    continue;
                }

                let is_chapter_heading = !line.starts_with_dropcap
                    && (chapter_heading_regex().is_match(trimmed_raw)
                        || (line.is_heading_font && line.max_font_size >= 20.0));

                if is_chapter_heading {
                    Self::flush_paragraph_buffer(
                        &mut buffer_text,
                        &mut buffer_is_dialogue,
                        &mut buffer_dropcap,
                        &mut current_elements,
                    );
                    Self::flush_code_buffer(&mut code_buffer, &mut current_elements);

                    if !current_elements.is_empty() {
                        chapters.push(Chapter {
                            index: chapter_index,
                            number_prefix: format!("{}", chapter_index),
                            title: current_title,
                            filename: format!("chapter_{:03}.xhtml", chapter_index),
                            elements: std::mem::take(&mut current_elements),
                            footnotes: std::mem::take(&mut current_footnotes),
                            subtopics: std::mem::take(&mut current_subtopics),
                        });
                        chapter_index += 1;
                        h2_counter = 0;
                        h3_counter = 0;
                    }

                    current_title = trimmed_raw.to_string();
                    continue;
                }

                if line.is_heading_font && line.max_font_size >= 13.5 {
                    Self::flush_paragraph_buffer(
                        &mut buffer_text,
                        &mut buffer_is_dialogue,
                        &mut buffer_dropcap,
                        &mut current_elements,
                    );
                    Self::flush_code_buffer(&mut code_buffer, &mut current_elements);
                    let ch_fname = format!("chapter_{:03}.xhtml", chapter_index);
                    if line.max_font_size >= 16.5 {
                        h2_counter += 1;
                        h3_counter = 0;
                        let num_prefix = format!("{}.{}", chapter_index, h2_counter);
                        let id = slugify_anchor_id(trimmed_raw, h2_counter);
                        insert_subtopic_node(
                            &mut current_subtopics,
                            2,
                            num_prefix.clone(),
                            trimmed_raw.to_string(),
                            format!("{}#{}", ch_fname, id),
                        );
                        current_elements.push(ChapterElement::SubheadingH2 {
                            id,
                            number_prefix: num_prefix,
                            title: escape_xhtml_fast(trimmed_raw),
                        });
                    } else {
                        if h2_counter == 0 {
                            h2_counter = 1;
                        }
                        h3_counter += 1;
                        let num_prefix = format!("{}.{}.{}", chapter_index, h2_counter, h3_counter);
                        let id = slugify_anchor_id(trimmed_raw, h3_counter);
                        insert_subtopic_node(
                            &mut current_subtopics,
                            3,
                            num_prefix.clone(),
                            trimmed_raw.to_string(),
                            format!("{}#{}", ch_fname, id),
                        );
                        current_elements.push(ChapterElement::SubheadingH3 {
                            id,
                            number_prefix: num_prefix,
                            title: escape_xhtml_fast(trimmed_raw),
                        });
                    }
                    continue;
                }

                if line.is_all_monospace && trimmed_raw.len() > 2 {
                    Self::flush_paragraph_buffer(
                        &mut buffer_text,
                        &mut buffer_is_dialogue,
                        &mut buffer_dropcap,
                        &mut current_elements,
                    );
                    code_buffer.push(escape_xhtml_fast(trimmed_raw));
                    continue;
                } else {
                    Self::flush_code_buffer(&mut code_buffer, &mut current_elements);
                }

                if caption_regex().is_match(trimmed_raw) {
                    Self::flush_paragraph_buffer(
                        &mut buffer_text,
                        &mut buffer_is_dialogue,
                        &mut buffer_dropcap,
                        &mut current_elements,
                    );
                    current_elements.push(ChapterElement::FigureCaption(trimmed_html.to_string()));
                    continue;
                }

                let starts_with_dash = trimmed_raw.starts_with('—')
                    || trimmed_raw.starts_with('–')
                    || trimmed_raw.starts_with("- ");

                if starts_with_dash {
                    Self::flush_paragraph_buffer(
                        &mut buffer_text,
                        &mut buffer_is_dialogue,
                        &mut buffer_dropcap,
                        &mut current_elements,
                    );
                    buffer_is_dialogue = true;
                }

                if line.starts_with_dropcap && buffer_text.is_empty() {
                    let mut chars = trimmed_raw.chars();
                    if let Some(first_char) = chars.next() {
                        buffer_dropcap = Some(first_char);
                        buffer_text.push_str(chars.as_str());
                    }
                } else if buffer_text.is_empty() {
                    buffer_text.push_str(trimmed_html);
                } else {
                    Self::stitch_line_into_buffer(&mut buffer_text, trimmed_html);
                }

                if !page_fn_map.is_empty() {
                    buffer_text = Self::link_inline_footnotes(&buffer_text, &page_fn_map);
                }

                if terminal_punctuation_regex().is_match(trimmed_raw) {
                    Self::flush_paragraph_buffer(
                        &mut buffer_text,
                        &mut buffer_is_dialogue,
                        &mut buffer_dropcap,
                        &mut current_elements,
                    );
                }
            }

            while fig_idx < page_figures.len() {
                Self::flush_paragraph_buffer(
                    &mut buffer_text,
                    &mut buffer_is_dialogue,
                    &mut buffer_dropcap,
                    &mut current_elements,
                );
                Self::flush_code_buffer(&mut code_buffer, &mut current_elements);
                current_elements.push(ChapterElement::FigureImage(
                    page_figures[fig_idx].href.clone(),
                ));
                fig_idx += 1;
            }
        }

        Self::flush_paragraph_buffer(
            &mut buffer_text,
            &mut buffer_is_dialogue,
            &mut buffer_dropcap,
            &mut current_elements,
        );
        Self::flush_code_buffer(&mut code_buffer, &mut current_elements);

        if !current_elements.is_empty() {
            chapters.push(Chapter {
                index: chapter_index,
                number_prefix: format!("{}", chapter_index),
                title: current_title,
                filename: format!("chapter_{:03}.xhtml", chapter_index),
                elements: current_elements,
                footnotes: current_footnotes,
                subtopics: current_subtopics,
            });
        }

        chapters
    }

    fn stitch_line_into_buffer(buffer: &mut String, next_line: &str) {
        let ends_with_hyphen = buffer.ends_with('-') || buffer.ends_with('\u{00AD}');
        let ends_with_spaced_hyphen = buffer.ends_with(" -") || buffer.ends_with(" —");

        if ends_with_hyphen && !ends_with_spaced_hyphen {
            let clean_next = strip_leading_tags(next_line);
            if let Some(first_ch) = clean_next.chars().next() {
                if first_ch.is_lowercase() {
                    buffer.pop();
                    buffer.push_str(next_line.trim_start());
                    return;
                }
            }
        }

        buffer.push(' ');
        buffer.push_str(next_line.trim_start());
    }

    fn link_inline_footnotes(text: &str, page_fn_map: &HashMap<String, usize>) -> String {
        inline_footnote_ref_regex()
            .replace_all(text, |caps: &regex::Captures| {
                let prefix = &caps[1];
                let num_str = caps[2].trim_matches(|c| c == '[' || c == ']');
                let suffix = &caps[3];
                if let Some(&fn_id) = page_fn_map.get(num_str) {
                    format!(
                        r##"{}<a epub:type="noteref" id="ref-fn-{}" href="#fn-{}"><sup>[{}]</sup></a>{}"##,
                        prefix, fn_id, fn_id, fn_id, suffix
                    )
                } else {
                    caps[0].to_string()
                }
            })
            .into_owned()
    }

    fn flush_paragraph_buffer(
        buffer_text: &mut String,
        buffer_is_dialogue: &mut bool,
        buffer_dropcap: &mut Option<char>,
        elements: &mut Vec<ChapterElement>,
    ) {
        let cleaned = normalize_whitespace(buffer_text);
        if !cleaned.is_empty() {
            if let Some(dc) = buffer_dropcap.take() {
                elements.push(ChapterElement::DropCapParagraph(dc, cleaned));
            } else if *buffer_is_dialogue {
                elements.push(ChapterElement::DialogueParagraph(cleaned));
            } else {
                elements.push(ChapterElement::StandardParagraph(cleaned));
            }
        }
        buffer_text.clear();
        *buffer_is_dialogue = false;
        *buffer_dropcap = None;
    }

    fn flush_code_buffer(code_buffer: &mut Vec<String>, elements: &mut Vec<ChapterElement>) {
        if !code_buffer.is_empty() {
            elements.push(ChapterElement::CodeBlock(code_buffer.join("\n")));
            code_buffer.clear();
        }
    }
}

pub struct NativeEpub3Builder {
    pub metadata: BookMetadata,
    pub chapters: Vec<Chapter>,
    pub figures: Vec<ExtractedFigure>,
}

impl NativeEpub3Builder {
    pub fn from_payload(payload: ExtractedBookPayload) -> Self {
        Self {
            metadata: payload.metadata,
            chapters: payload.chapters,
            figures: payload.figures,
        }
    }

    pub fn save(&self, output_path: &Path) -> std::io::Result<()> {
        let file = File::create(output_path)?;
        let buf_writer = BufWriter::with_capacity(512 * 1024, file);
        let mut zip = ZipWriter::new(buf_writer);

        let stored_opts = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .unix_permissions(0o644);
        zip.start_file("mimetype", stored_opts)?;
        zip.write_all(b"application/epub+zip")?;

        let deflated_opts = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(6))
            .unix_permissions(0o644);

        zip.start_file("META-INF/container.xml", deflated_opts)?;
        zip.write_all(Self::build_container_xml().as_bytes())?;

        zip.start_file("OEBPS/style.css", deflated_opts)?;
        zip.write_all(Self::build_editorial_css().as_bytes())?;

        if let Some(ref cover_bytes) = self.metadata.cover_jpeg_bytes {
            zip.start_file("OEBPS/images/cover.jpg", deflated_opts)?;
            zip.write_all(cover_bytes)?;

            zip.start_file("OEBPS/cover.xhtml", deflated_opts)?;
            zip.write_all(self.build_cover_xhtml().as_bytes())?;
        }

        zip.start_file("OEBPS/sinopse.xhtml", deflated_opts)?;
        zip.write_all(self.build_synopsis_xhtml().as_bytes())?;

        for fig in &self.figures {
            let fig_path = format!("OEBPS/{}", fig.href);
            zip.start_file(fig_path, deflated_opts)?;
            zip.write_all(&fig.jpeg_bytes)?;
        }

        zip.start_file("OEBPS/nav.xhtml", deflated_opts)?;
        zip.write_all(self.build_nav_xhtml().as_bytes())?;

        zip.start_file("OEBPS/content.opf", deflated_opts)?;
        zip.write_all(self.build_content_opf().as_bytes())?;

        let rendered_chapters: Vec<(&str, String)> = self
            .chapters
            .par_iter()
            .map(|ch| (ch.filename.as_str(), self.build_chapter_xhtml(ch)))
            .collect();

        for (filename, xhtml_content) in rendered_chapters {
            let path_in_zip = format!("OEBPS/{}", filename);
            zip.start_file(path_in_zip, deflated_opts)?;
            zip.write_all(xhtml_content.as_bytes())?;
        }

        zip.finish()?;
        Ok(())
    }

    fn build_container_xml() -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#
    }

    fn build_editorial_css() -> &'static str {
        r#"@charset "UTF-8";
body {
  font-family: Georgia, "Bookerly", "Literata", serif;
  line-height: 1.55;
  margin: 4% 5%;
  color: #1a1a1a;
}
body.cover-body {
  margin: 0;
  padding: 0;
  text-align: center;
}
div.cover-container {
  margin: 0;
  padding: 0;
  height: 100vh;
}
img.cover-image {
  max-width: 100%;
  max-height: 98vh;
  object-fit: contain;
}
h1.chapter-title {
  text-align: right;
  font-size: 1.55em;
  font-weight: bold;
  border-bottom: 2px solid #1793d1;
  padding-bottom: 0.4em;
  margin-top: 1.5em;
  margin-bottom: 1.4em;
}
h2 {
  font-size: 1.25em;
  margin-top: 1.4em;
  margin-bottom: 0.5em;
}
h3 {
  font-size: 1.08em;
  margin-top: 1.1em;
  margin-bottom: 0.4em;
}
h4 {
  font-size: 0.98em;
  font-style: italic;
  margin-top: 0.9em;
  margin-bottom: 0.35em;
}
span.sec-num {
  font-family: "Courier New", monospace;
  font-weight: bold;
  color: #555;
  margin-right: 0.35em;
}
ol.toc-sublist {
  list-style-type: none;
  padding-left: 1.25em;
  margin-top: 0.2em;
  margin-bottom: 0.35em;
}
p {
  text-indent: 1.2em;
  margin-top: 0;
  margin-bottom: 0.45em;
  text-align: justify;
}
p.first-para, p.dropcap-para, p.meta-line {
  text-indent: 0;
}
span.dropcap {
  float: left;
  font-size: 3.1em;
  line-height: 0.85;
  padding-top: 0.04em;
  padding-right: 0.1em;
  font-weight: bold;
}
p.dialogue {
  text-indent: 1.2em;
  margin-top: 0;
  margin-bottom: 0.3em;
}
pre.code-block {
  font-family: "Courier New", monospace;
  font-size: 0.84em;
  background-color: #f5f5f5;
  border-left: 3px solid #1793d1;
  padding: 0.6em 0.8em;
  margin: 0.8em 0;
  white-space: pre-wrap;
  word-wrap: break-word;
}
code {
  font-family: "Courier New", monospace;
  font-size: 0.9em;
}
aside.callout {
  padding: 0.6em 0.9em;
  margin: 0.8em 0;
  border-left: 4px solid #555;
  background-color: #f8f9fa;
}
aside.callout-note { border-left-color: #1793d1; }
aside.callout-tip { border-left-color: #2e7d32; }
aside.callout-warning { border-left-color: #c62828; background-color: #fff8f8; }
table.wikitable {
  width: 100%;
  border-collapse: collapse;
  margin: 1em 0;
  font-size: 0.88em;
}
table.wikitable th, table.wikitable td {
  border: 1px solid #aaa;
  padding: 0.4em 0.6em;
  text-align: left;
}
table.wikitable th {
  background-color: #eaeff2;
  font-weight: bold;
}
figure.book-figure {
  margin: 1.2em auto;
  text-align: center;
  page-break-inside: avoid;
}
figure.book-figure img {
  max-width: 96%;
  height: auto;
}
p.fig-caption {
  text-indent: 0;
  font-size: 0.88em;
  font-style: italic;
  text-align: center;
  margin-bottom: 1.1em;
}
aside[epub|type~="footnote"] {
  font-size: 0.85em;
  line-height: 1.35;
  margin-bottom: 0.6em;
}
"#
    }

    fn build_cover_xhtml(&self) -> String {
        let safe_title = escape_xhtml_fast(&self.metadata.title);
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="{lang}" lang="{lang}">
<head>
  <meta charset="utf-8"/>
  <title>Capa — {title}</title>
  <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body class="cover-body" epub:type="cover">
  <div class="cover-container">
    <img src="images/cover.jpg" alt="Capa — {title}" class="cover-image"/>
  </div>
</body>
</html>"#,
            lang = escape_xhtml_fast(&self.metadata.language),
            title = safe_title
        )
    }

    fn build_synopsis_xhtml(&self) -> String {
        let safe_title = escape_xhtml_fast(&self.metadata.title);
        let safe_authors = escape_xhtml_fast(&self.metadata.authors.join(", "));
        let safe_translator = escape_xhtml_fast(&self.metadata.translator);
        let safe_publisher = escape_xhtml_fast(&self.metadata.publisher);
        let safe_isbn = escape_xhtml_fast(&self.metadata.isbn);
        let safe_synopsis = escape_xhtml_fast(&self.metadata.synopsis);

        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="{lang}" lang="{lang}">
<head>
  <meta charset="utf-8"/>
  <title>Sinopse e Metadados — {title}</title>
  <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body epub:type="frontmatter">
  <section>
    <h1 class="chapter-title">{title}</h1>
    <p class="meta-line"><strong>Autores:</strong> {authors}</p>
    <p class="meta-line"><strong>Colaboração / Tradução:</strong> {translator} | <strong>Editora:</strong> {publisher}</p>
    <p class="meta-line"><strong>Identificador:</strong> {isbn}</p>
    <h2>Sinopse</h2>
    <p class="first-para">{synopsis}</p>
  </section>
</body>
</html>"#,
            lang = escape_xhtml_fast(&self.metadata.language),
            title = safe_title,
            authors = safe_authors,
            translator = safe_translator,
            publisher = safe_publisher,
            isbn = safe_isbn,
            synopsis = safe_synopsis
        )
    }

    fn build_chapter_xhtml(&self, chapter: &Chapter) -> String {
        let mut body_buf = String::with_capacity(chapter.elements.len() * 180);

        for (idx, elem) in chapter.elements.iter().enumerate() {
            match elem {
                ChapterElement::SubheadingH2 {
                    id,
                    number_prefix,
                    title,
                } => {
                    body_buf.push_str(&format!(
                        "    <h2 id=\"{}\"><span class=\"sec-num\">{}</span> {}</h2>\n",
                        escape_xhtml_fast(id),
                        escape_xhtml_fast(number_prefix),
                        title
                    ));
                }
                ChapterElement::SubheadingH3 {
                    id,
                    number_prefix,
                    title,
                } => {
                    body_buf.push_str(&format!(
                        "    <h3 id=\"{}\"><span class=\"sec-num\">{}</span> {}</h3>\n",
                        escape_xhtml_fast(id),
                        escape_xhtml_fast(number_prefix),
                        title
                    ));
                }
                ChapterElement::SubheadingH4 {
                    id,
                    number_prefix,
                    title,
                } => {
                    body_buf.push_str(&format!(
                        "    <h4 id=\"{}\"><span class=\"sec-num\">{}</span> {}</h4>\n",
                        escape_xhtml_fast(id),
                        escape_xhtml_fast(number_prefix),
                        title
                    ));
                }
                ChapterElement::DropCapParagraph(dc, rest) => {
                    body_buf.push_str(&format!(
                        "    <p class=\"dropcap-para\"><span class=\"dropcap\">{}</span>{}</p>\n",
                        escape_xhtml_fast(&dc.to_string()),
                        rest
                    ));
                }
                ChapterElement::DialogueParagraph(text) => {
                    body_buf.push_str(&format!("    <p class=\"dialogue\">{}</p>\n", text));
                }
                ChapterElement::StandardParagraph(text) => {
                    let class_attr = if idx == 0 { " class=\"first-para\"" } else { "" };
                    body_buf.push_str(&format!("    <p{}>{}</p>\n", class_attr, text));
                }
                ChapterElement::CodeBlock(code) => {
                    body_buf.push_str(&format!(
                        "    <pre class=\"code-block\"><code>{}</code></pre>\n",
                        code
                    ));
                }
                ChapterElement::CalloutBox {
                    kind,
                    title,
                    body_html,
                } => {
                    let kind_class = match kind {
                        CalloutKind::Note => "callout-note",
                        CalloutKind::Tip => "callout-tip",
                        CalloutKind::Warning => "callout-warning",
                    };
                    body_buf.push_str(&format!(
                        "    <aside class=\"callout {}\"><p class=\"first-para\"><strong>{}:</strong> {}</p></aside>\n",
                        kind_class,
                        escape_xhtml_fast(title),
                        body_html
                    ));
                }
                ChapterElement::DataTable { headers, rows } => {
                    body_buf.push_str("    <table class=\"wikitable\">\n");
                    if !headers.is_empty() {
                        body_buf.push_str("      <thead><tr>");
                        for h in headers {
                            body_buf.push_str(&format!("<th>{}</th>", h));
                        }
                        body_buf.push_str("</tr></thead>\n");
                    }
                    body_buf.push_str("      <tbody>\n");
                    for r in rows {
                        body_buf.push_str("        <tr>");
                        for cell in r {
                            body_buf.push_str(&format!("<td>{}</td>", cell));
                        }
                        body_buf.push_str("</tr>\n");
                    }
                    body_buf.push_str("      </tbody>\n    </table>\n");
                }
                ChapterElement::FigureImage(href) => {
                    body_buf.push_str(&format!(
                        "    <figure class=\"book-figure\"><img src=\"{}\" alt=\"Figura técnica\"/></figure>\n",
                        href
                    ));
                }
                ChapterElement::FigureCaption(cap) => {
                    body_buf.push_str(&format!("    <p class=\"fig-caption\">{}</p>\n", cap));
                }
            }
        }

        if !chapter.footnotes.is_empty() {
            body_buf.push_str("    <hr/>\n    <section epub:type=\"footnotes\">\n");
            for fn_entry in &chapter.footnotes {
                body_buf.push_str(&format!(
                    "      <aside epub:type=\"footnote\" id=\"fn-{id}\">\n        <p><strong>[{id}]</strong> {content} <a href=\"#ref-fn-{id}\">↩</a></p>\n      </aside>\n",
                    id = fn_entry.id_num,
                    content = fn_entry.content_html
                ));
            }
            body_buf.push_str("    </section>\n");
        }

        let safe_title = escape_xhtml_fast(&chapter.title);
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="{lang}" lang="{lang}">
<head>
  <meta charset="utf-8"/>
  <title>{title}</title>
  <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body epub:type="bodymatter">
  <section epub:type="chapter">
    <h1 class="chapter-title">{title}</h1>
{body}  </section>
</body>
</html>"#,
            lang = escape_xhtml_fast(&self.metadata.language),
            title = safe_title,
            body = body_buf
        )
    }

    fn render_toc_nodes_recursive(nodes: &[TocNode], indent: usize) -> String {
        if nodes.is_empty() {
            return String::new();
        }
        let pad = " ".repeat(indent);
        let mut out = format!("{}<ol class=\"toc-sublist\">\n", pad);
        for node in nodes {
            let label = if node.number_prefix.is_empty() {
                escape_xhtml_fast(&node.title)
            } else {
                format!(
                    "{} {}",
                    escape_xhtml_fast(&node.number_prefix),
                    escape_xhtml_fast(&node.title)
                )
            };
            if node.children.is_empty() {
                out.push_str(&format!(
                    "{}  <li><a href=\"{}\">{}</a></li>\n",
                    pad,
                    escape_xhtml_fast(&node.href),
                    label
                ));
            } else {
                out.push_str(&format!(
                    "{}  <li>\n{}    <a href=\"{}\">{}</a>\n{}{pad}  </li>\n",
                    pad,
                    pad,
                    escape_xhtml_fast(&node.href),
                    label,
                    Self::render_toc_nodes_recursive(&node.children, indent + 4),
                    pad = pad
                ));
            }
        }
        out.push_str(&format!("{}</ol>\n", pad));
        out
    }

    fn build_nav_xhtml(&self) -> String {
        let mut items = String::new();
        if self.metadata.cover_jpeg_bytes.is_some() {
            items.push_str("      <li><a href=\"cover.xhtml\">Capa</a></li>\n");
        }
        items.push_str("      <li><a href=\"sinopse.xhtml\">Sinopse e Ficha Editorial</a></li>\n");
        for ch in &self.chapters {
            if ch.subtopics.is_empty() {
                items.push_str(&format!(
                    "      <li><a href=\"{}\">{}</a></li>\n",
                    ch.filename,
                    escape_xhtml_fast(&ch.title)
                ));
            } else {
                let nested_ol = Self::render_toc_nodes_recursive(&ch.subtopics, 8);
                items.push_str(&format!(
                    "      <li>\n        <a href=\"{}\">{}</a>\n{}      </li>\n",
                    ch.filename,
                    escape_xhtml_fast(&ch.title),
                    nested_ol
                ));
            }
        }

        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="{lang}" lang="{lang}">
<head>
  <meta charset="utf-8"/>
  <title>Sumário</title>
  <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
  <nav epub:type="toc" id="toc">
    <h1>Sumário</h1>
    <ol>
{items}    </ol>
  </nav>
</body>
</html>"#,
            lang = escape_xhtml_fast(&self.metadata.language),
            items = items
        )
    }

    fn build_content_opf(&self) -> String {
        let mut manifest_items = String::from(
            "    <item id=\"style\" href=\"style.css\" media-type=\"text/css\"/>\n    <item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n",
        );
        let mut spine_items = String::new();

        if self.metadata.cover_jpeg_bytes.is_some() {
            manifest_items.push_str(
                "    <item id=\"cover-img\" href=\"images/cover.jpg\" media-type=\"image/jpeg\" properties=\"cover-image\"/>\n",
            );
            manifest_items.push_str(
                "    <item id=\"cover\" href=\"cover.xhtml\" media-type=\"application/xhtml+xml\"/>\n",
            );
            spine_items.push_str("    <itemref idref=\"cover\"/>\n");
        }

        manifest_items.push_str(
            "    <item id=\"sinopse\" href=\"sinopse.xhtml\" media-type=\"application/xhtml+xml\"/>\n",
        );
        spine_items.push_str("    <itemref idref=\"sinopse\"/>\n");

        for fig in &self.figures {
            manifest_items.push_str(&format!(
                "    <item id=\"{}\" href=\"{}\" media-type=\"image/jpeg\"/>\n",
                fig.id, fig.href
            ));
        }

        for ch in &self.chapters {
            let item_id = format!("chap_{:03}", ch.index);
            manifest_items.push_str(&format!(
                "    <item id=\"{}\" href=\"{}\" media-type=\"application/xhtml+xml\"/>\n",
                item_id, ch.filename
            ));
            spine_items.push_str(&format!("    <itemref idref=\"{}\"/>\n", item_id));
        }

        let creators_xml: String = self
            .metadata
            .authors
            .iter()
            .enumerate()
            .map(|(i, a)| {
                format!(
                    "    <dc:creator id=\"creator-{}\">{}</dc:creator>\n",
                    i + 1,
                    escape_xhtml_fast(a)
                )
            })
            .collect();

        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id" xml:lang="{lang}">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="pub-id">{isbn}</dc:identifier>
    <dc:identifier id="uuid-id">urn:uuid:{uuid}</dc:identifier>
    <dc:title>{title}</dc:title>
{creators}    <dc:contributor>{translator}</dc:contributor>
    <dc:publisher>{publisher}</dc:publisher>
    <dc:date>{date}</dc:date>
    <dc:language>{lang}</dc:language>
    <dc:description>{synopsis}</dc:description>
    <meta name="cover" content="cover-img"/>
    <meta property="dcterms:modified">2026-10-16T00:05:00Z</meta>
  </metadata>
  <manifest>
{manifest}  </manifest>
  <spine>
{spine}  </spine>
</package>"#,
            lang = escape_xhtml_fast(&self.metadata.language),
            isbn = escape_xhtml_fast(&self.metadata.isbn),
            uuid = escape_xhtml_fast(&self.metadata.uuid),
            title = escape_xhtml_fast(&self.metadata.title),
            creators = creators_xml,
            translator = escape_xhtml_fast(&self.metadata.translator),
            publisher = escape_xhtml_fast(&self.metadata.publisher),
            date = escape_xhtml_fast(&self.metadata.publication_date),
            synopsis = escape_xhtml_fast(&self.metadata.synopsis),
            manifest = manifest_items,
            spine = spine_items
        )
    }
}

pub fn escape_xhtml_fast(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 16);
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

pub fn normalize_whitespace(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_was_space = false;
    for ch in input.trim().chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
        } else {
            out.push(ch);
            last_was_space = false;
        }
    }
    out
}

fn strip_leading_tags(mut input: &str) -> &str {
    input = input.trim_start();
    while input.starts_with('<') {
        if let Some(end_idx) = input.find('>') {
            input = input[end_idx + 1..].trim_start();
        } else {
            break;
        }
    }
    input
}

pub fn extract_pdf_with_cover_and_figures(
    pdf_path: &Path,
) -> Result<(Option<Vec<u8>>, Vec<RawPdfPage>, Vec<ExtractedFigure>), Box<dyn std::error::Error>> {
    use image::ImageFormat;
    use pdfium_render::prelude::*;

    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())?,
    );

    let document = pdfium.load_pdf_from_file(pdf_path, None)?;
    let total_pages = document.pages().len() as usize;
    let mut raw_pages = Vec::with_capacity(total_pages);
    let mut all_figures = Vec::new();
    let mut cover_jpeg_bytes: Option<Vec<u8>> = None;
    let mut fig_counter = 1usize;

    for (page_idx, page) in document.pages().iter().enumerate() {
        if page_idx == 0 {
            let render_cfg = PdfRenderConfig::new().set_target_width(1400);
            let bitmap = page.render_with_config(&render_cfg)?;
            let dyn_img = bitmap.as_image();
            let mut buf = Cursor::new(Vec::new());
            dyn_img.write_to(&mut buf, ImageFormat::Jpeg)?;
            cover_jpeg_bytes = Some(buf.into_inner());
            continue;
        }

        let width = page.width().value;
        let height = page.height().value;
        let mut words: Vec<PdfWordBox> = Vec::new();
        let mut page_figures: Vec<ExtractedFigure> = Vec::new();

        for object in page.objects().iter() {
            if let Some(text_obj) = object.as_text_object() {
                let text = text_obj.text();
                if text.trim().is_empty() {
                    continue;
                }
                if let Ok(bounds) = object.bounds() {
                    let font_size = text_obj.unscaled_font_size().value;
                    let font_name = text_obj.font().name();
                    let fn_lower = font_name.to_lowercase();
                    let is_italic = text_obj.font().is_italic()
                        || fn_lower.contains("italic")
                        || fn_lower.contains("oblique");
                    let is_bold = fn_lower.contains("bold") || fn_lower.contains("semibold");
                    let is_monospace = fn_lower.contains("lekton") || fn_lower.contains("mono");

                    let top = (height - bounds.top().value).max(0.0);
                    let bottom = (height - bounds.bottom().value).max(top);

                    words.push(PdfWordBox {
                        text,
                        font_name,
                        x0: bounds.left().value,
                        top,
                        x1: bounds.right().value,
                        bottom,
                        font_size,
                        is_italic,
                        is_bold,
                        is_monospace,
                    });
                }
            } else if let Some(img_obj) = object.as_image_object() {
                if let (Ok(bounds), Ok(raw_img)) = (object.bounds(), img_obj.get_raw_image()) {
                    let w_pt = bounds.right().value - bounds.left().value;
                    let h_pt = bounds.top().value - bounds.bottom().value;
                    if w_pt > 85.0 && h_pt > 55.0 && page_idx >= 19 {
                        let mut buf = Cursor::new(Vec::new());
                        if raw_img.write_to(&mut buf, ImageFormat::Jpeg).is_ok() {
                            let fig_id = format!("fig_{:03}", fig_counter);
                            let fig_href =
                                format!("images/fig_{:03}_p{}.jpg", fig_counter, page_idx + 1);
                            let top = (height - bounds.top().value).max(0.0);
                            let fig = ExtractedFigure {
                                id: fig_id,
                                href: fig_href,
                                top,
                                jpeg_bytes: buf.into_inner(),
                            };
                            page_figures.push(fig.clone());
                            all_figures.push(fig);
                            fig_counter += 1;
                        }
                    }
                }
            }
        }

        raw_pages.push(RawPdfPage {
            page_index: page_idx,
            width,
            height,
            words,
            figures: page_figures,
        });
    }

    Ok((cover_jpeg_bytes, raw_pages, all_figures))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    let authors: Vec<String> = args
        .author
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let base_metadata = BookMetadata {
        uuid: Uuid::new_v4().to_string(),
        isbn: args.isbn.clone(),
        title: args.title.clone(),
        authors,
        translator: args.translator.clone(),
        publisher: args.publisher.clone(),
        publication_date: String::from("2026-10-15"),
        synopsis: args.synopsis.clone(),
        language: args.lang.clone(),
        cover_jpeg_bytes: None,
    };

    let is_web_url = args.input.starts_with("http://") || args.input.starts_with("https://");
    let effective_kind = match args.source_kind {
        SourceKind::Auto => {
            if is_web_url {
                if args.input.contains("wiki.") || args.input.contains("/title/") {
                    SourceKind::MediaWiki
                } else if args.input.contains("docs.python.org") || args.input.contains("readthedocs") {
                    SourceKind::SphinxDocs
                } else if args.input.contains("git-scm.com/book") {
                    SourceKind::AsciiDocBook
                } else if args.input.contains("effective_go") || args.input.contains("go.dev/doc/") {
                    SourceKind::SinglePageSpec
                } else if args.input.contains("developer.mozilla.org") {
                    SourceKind::MdnGuide
                } else if args.input.contains("/book") || args.input.contains("docs") {
                    SourceKind::WebDocs
                } else {
                    SourceKind::WebArticle
                }
            } else {
                SourceKind::Pdf
            }
        }
        other => other,
    };

    let payload = match effective_kind {
        SourceKind::MediaWiki => {
            println!(
                "[*] Modo MediaWiki/ArchWiki ativado (BFS max_depth={}, max_chapters={}): {}",
                args.max_depth, args.max_chapters, args.input
            );
            let wiki_extractor = MediaWikiExtractor {
                root_url: args.input.clone(),
                max_depth: args.max_depth,
                max_chapters: args.max_chapters,
                default_metadata: base_metadata,
            };
            wiki_extractor.extract_payload()?
        }
        SourceKind::SinglePageSpec => {
            println!(
                "[*] Modo SinglePageSpec ativado (ex: Effective Go): segmentando capítulos H2 e subtópicos H3 de {}",
                args.input
            );
            let spec_extractor = SinglePageSpecExtractor {
                url: args.input.clone(),
                default_metadata: base_metadata,
            };
            spec_extractor.extract_payload()?
        }
        SourceKind::WebArticle
        | SourceKind::WebDocs
        | SourceKind::SphinxDocs
        | SourceKind::AsciiDocBook
        | SourceKind::MdnGuide => {
            println!(
                "[*] Modo Documentação Web ativado ({:?}): extraindo capítulos e subtópicos de {}",
                effective_kind, args.input
            );
            let web_extractor = WebSiteExtractor {
                start_url: args.input.clone(),
                crawl_docs_toc: effective_kind != SourceKind::WebArticle,
                toc_selector: args.toc_selector.clone(),
                default_metadata: base_metadata,
            };
            web_extractor.extract_payload()?
        }
        SourceKind::Pdf | SourceKind::Auto => {
            let pdf_path = Path::new(&args.input);
            println!(
                "[*] Modo PDF ativado: extraindo geometria e capa de {}",
                pdf_path.display()
            );
            let (cover_jpeg_bytes, raw_pages, all_figures) =
                extract_pdf_with_cover_and_figures(pdf_path)?;
            let pipeline = PdfProsePipeline::from_pages(&raw_pages, args.margin_cutoff);
            let chapters = pipeline.extract_chapters(&raw_pages);
            let mut meta = base_metadata;
            meta.cover_jpeg_bytes = cover_jpeg_bytes;
            ExtractedBookPayload {
                metadata: meta,
                chapters,
                figures: all_figures,
            }
        }
    };

    let builder = NativeEpub3Builder::from_payload(payload);
    builder.save(&args.output)?;
    println!(
        "[+] EPUB 3.3 gerado com sucesso em: {}",
        args.output.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    fn dummy_meta() -> BookMetadata {
        BookMetadata {
            uuid: String::from("00000000-0000-4000-8000-000000000000"),
            isbn: String::from("urn:test:v0.9.0"),
            title: String::from("Test Book"),
            authors: vec![String::from("Test Author")],
            translator: String::from("Test"),
            publisher: String::from("Test"),
            publication_date: String::from("2026-10-24"),
            synopsis: String::from("Test"),
            language: String::from("en"),
            cover_jpeg_bytes: None,
        }
    }

    #[test]
    fn test_leaf_dom_extraction_preserves_li_dt_dd_and_callouts() {
        let html = r#"
        <html><body>
          <article class="main-page-content">
            <h1>Control flow and error handling</h1>
            <p>JavaScript supports a compact set of control flow statements.</p>
            <h2 id="conditional-statements">Conditional statements</h2>
            <p>A conditional statement executes if a specified condition is true.</p>
            <ul>
              <li>false literal</li>
              <li>undefined primitive</li>
              <li>null reference</li>
              <li>0 and -0 numeric values</li>
              <li>NaN (Not-a-Number)</li>
              <li>empty string ("")</li>
            </ul>
            <dl>
              <dt>unhandledrejection</dt>
              <dd>Sent when a promise is rejected but there is no rejection handler available.</dd>
              <dt>rejectionhandled</dt>
              <dd>Sent when a handler is attached to a rejected promise.</dd>
            </dl>
            <div class="notecard note">
              <p><strong>Note:</strong> Do not confuse primitive boolean values with Boolean objects.</p>
            </div>
            <pre><code>js Copy
if (condition) {
  statement1;
}</code></pre>
          </article>
        </body></html>
        "#;
        let doc = Html::parse_document(html);
        let extractor = WebSiteExtractor {
            start_url: String::from("https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Control_flow_and_error_handling"),
            crawl_docs_toc: true,
            toc_selector: String::new(),
            default_metadata: dummy_meta(),
        };
        let (title, elements, subtopics) =
            extractor.extract_clean_article_elements(&doc, 3, "3", "chapter_003.xhtml");
        assert_eq!(title, "3. Control flow and error handling");
        assert_eq!(subtopics.len(), 1);
        assert_eq!(elements.len(), 15);
    }

    #[test]
    fn test_progit_mdn_sphinx_url_resolution_and_scope() {
        let mdn_base = WebSiteExtractor::normalize_base_url(
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide",
        );
        assert_eq!(
            WebSiteExtractor::resolve_chapter_url(
                &mdn_base,
                "/en-US/docs/Web/JavaScript/Guide/Using_promises#chaining"
            ),
            Some(String::from(
                "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Using_promises"
            ))
        );
        let progit_base = WebSiteExtractor::normalize_base_url("https://git-scm.com/book/en/v2");
        assert_eq!(
            WebSiteExtractor::resolve_chapter_url(
                &progit_base,
                "/book/en/v2/Git-Internals-Git-Objects"
            ),
            Some(String::from(
                "https://git-scm.com/book/en/v2/Git-Internals-Git-Objects"
            ))
        );
    }
}
