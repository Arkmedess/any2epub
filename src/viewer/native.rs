//! Motor nativo de leitura e paginação em Rust puro (`NativeRustReaderEngine`).

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use zip::ZipArchive;

use scraper::{Html, Selector};

use crate::domain::ir::TocNode;
use crate::viewer::bitmap::PageBitmap;
use crate::viewer::engine::{BookDetails, ReaderEngine, ViewerError};
use crate::viewer::options::ViewerRenderOptions;

/// Linha de texto paginada pronta para renderização geométrica.
#[derive(Debug, Clone)]
struct LayoutLine {
    text: String,
    pub is_heading: bool,
}

/// Página discreta de leitura contendo linhas paginadas para as dimensões do viewport.
#[derive(Debug, Clone)]
struct PageContent {
    lines: Vec<LayoutLine>,
}

/// Seção ou capítulo extraído do `<spine>` do EPUB.
#[derive(Debug, Clone)]
struct SpineDocument {
    _href: String,
    raw_blocks: Vec<(bool, String)>, // (is_heading, text)
}

/// Implementação do motor de leitura e layout 100% nativo em Rust.
pub struct NativeRustReaderEngine {
    book_details: Option<BookDetails>,
    documents: Vec<SpineDocument>,
    toc: Vec<TocNode>,
    viewport_width: u32,
    viewport_height: u32,
    options: ViewerRenderOptions,
    pages: Vec<PageContent>,
}

impl NativeRustReaderEngine {
    /// Instancia um novo motor nativo com dimensões padrão de viewport (800x1200).
    pub fn new() -> Self {
        Self {
            book_details: None,
            documents: Vec::new(),
            toc: Vec::new(),
            viewport_width: 800,
            viewport_height: 1200,
            options: ViewerRenderOptions::default(),
            pages: Vec::new(),
        }
    }

    /// Recalcula a quebra de linhas e a divisão de páginas para o viewport e opções atuais.
    fn repaginate(&mut self) {
        self.pages.clear();

        if self.documents.is_empty() {
            return;
        }

        let margin = self.options.margin_px;
        let content_width = self.viewport_width.saturating_sub(margin * 2).max(100);
        let content_height = self.viewport_height.saturating_sub(margin * 2).max(100);

        let font_px = (self.options.font_size_pt * 1.333).max(8.0);
        let line_height_px = (font_px * self.options.line_height_multiplier) as u32;
        let lines_per_page = (content_height / line_height_px.max(1)).max(1);

        let avg_char_width = (font_px * 0.55) as u32;
        let chars_per_line = (content_width / avg_char_width.max(1)).max(10) as usize;

        for doc in &self.documents {
            let mut current_page_lines = Vec::new();

            for (is_heading, block_text) in &doc.raw_blocks {
                let wrapped_lines = wrap_text_to_lines(block_text, chars_per_line);

                for line_text in wrapped_lines {
                    if current_page_lines.len() >= lines_per_page as usize {
                        self.pages.push(PageContent {
                            lines: std::mem::take(&mut current_page_lines),
                        });
                    }
                    current_page_lines.push(LayoutLine {
                        text: line_text,
                        is_heading: *is_heading,
                    });
                }

                // Espaçamento entre parágrafos
                if !current_page_lines.is_empty()
                    && current_page_lines.len() < lines_per_page as usize
                {
                    current_page_lines.push(LayoutLine {
                        text: String::new(),
                        is_heading: false,
                    });
                }
            }

            if !current_page_lines.is_empty() {
                self.pages.push(PageContent {
                    lines: current_page_lines,
                });
            }
        }

        if self.pages.is_empty() {
            self.pages.push(PageContent { lines: Vec::new() });
        }
    }
}

impl Default for NativeRustReaderEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReaderEngine for NativeRustReaderEngine {
    fn engine_name(&self) -> &'static str {
        "native-rust"
    }

    fn open_book(&mut self, path: &Path) -> Result<BookDetails, ViewerError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut archive = ZipArchive::new(reader)
            .map_err(|e| ViewerError::ZipError(format!("Erro ao desempacotar EPUB: {}", e)))?;

        // 1. Localiza o OPF no META-INF/container.xml
        let opf_path = {
            let mut container_file = archive.by_name("META-INF/container.xml").map_err(|_| {
                ViewerError::XmlError("META-INF/container.xml ausente no pacote".into())
            })?;
            let mut content = String::new();
            container_file.read_to_string(&mut content)?;
            extract_attr_val(&content, "full-path")
                .ok_or_else(|| ViewerError::XmlError("Atributo full-path ausente no container.xml".into()))?
        };

        let opf_base_dir = if let Some(slash) = opf_path.rfind('/') {
            format!("{}/", &opf_path[..slash])
        } else {
            String::new()
        };

        // 2. Lê o arquivo OPF
        let (title, authors, language, spine_hrefs) = {
            let mut opf_file = archive.by_name(&opf_path).map_err(|_| {
                ViewerError::XmlError(format!("Arquivo OPF '{}' ausente no pacote", opf_path))
            })?;
            let mut opf_content = String::new();
            opf_file.read_to_string(&mut opf_content)?;

            let title = extract_tag_content(&opf_content, "dc:title")
                .unwrap_or_else(|| String::from("Título Desconhecido"));

            let mut authors = Vec::new();
            let mut cursor = 0;
            while let Some(start) = opf_content[cursor..].find("<dc:creator") {
                let actual_start = cursor + start;
                if let Some(tag_end) = opf_content[actual_start..].find('>') {
                    let body_start = actual_start + tag_end + 1;
                    if let Some(close_tag) = opf_content[body_start..].find("</dc:creator>") {
                        let author_name = opf_content[body_start..body_start + close_tag].trim();
                        if !author_name.is_empty() {
                            authors.push(author_name.to_string());
                        }
                        cursor = body_start + close_tag;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            let language = extract_tag_content(&opf_content, "dc:language")
                .unwrap_or_else(|| String::from("pt-BR"));

            // Mapeia manifesto: id -> href
            let mut manifest = HashMap::new();
            let mut man_cursor = 0;
            while let Some(start) = opf_content[man_cursor..].find("<item ") {
                let actual = man_cursor + start;
                if let Some(end) = opf_content[actual..].find('>') {
                    let tag = &opf_content[actual..actual + end];
                    if let (Some(id), Some(href)) = (extract_attr_val(tag, "id"), extract_attr_val(tag, "href")) {
                        manifest.insert(id, href);
                    }
                    man_cursor = actual + end;
                } else {
                    break;
                }
            }

            // Mapeia spine: idref -> hrefs ordenados
            let mut spine_hrefs = Vec::new();
            let mut spine_cursor = 0;
            while let Some(start) = opf_content[spine_cursor..].find("<itemref ") {
                let actual = spine_cursor + start;
                if let Some(end) = opf_content[actual..].find('>') {
                    let tag = &opf_content[actual..actual + end];
                    if let Some(idref) = extract_attr_val(tag, "idref") {
                        if let Some(href) = manifest.get(&idref) {
                            spine_hrefs.push(href.clone());
                        }
                    }
                    spine_cursor = actual + end;
                } else {
                    break;
                }
            }

            (title, authors, language, spine_hrefs)
        };

        // 3. Extrai o conteúdo dos documentos XHTML listados no spine
        let block_selector = Selector::parse("h1, h2, h3, h4, p, li, blockquote, pre")
            .map_err(|e| ViewerError::XmlError(format!("Erro ao compilar seletor CSS: {:?}", e)))?;

        let mut documents = Vec::new();
        for href in &spine_hrefs {
            let zip_resource_path = format!("{}{}", opf_base_dir, href);
            if let Ok(mut xhtml_file) = archive.by_name(&zip_resource_path) {
                let mut xhtml_text = String::new();
                xhtml_file.read_to_string(&mut xhtml_text)?;

                let parsed_doc = Html::parse_document(&xhtml_text);
                let mut raw_blocks = Vec::new();

                for element in parsed_doc.select(&block_selector) {
                    let tag_name = element.value().name();
                    let is_heading = tag_name.starts_with('h');
                    let text = element.text().collect::<String>().trim().to_string();
                    if !text.is_empty() {
                        raw_blocks.push((is_heading, text));
                    }
                }

                documents.push(SpineDocument {
                    _href: href.clone(),
                    raw_blocks,
                });
            }
        }

        let spine_documents_count = spine_hrefs.len();
        let details = BookDetails {
            title,
            authors,
            spine_documents_count,
            language,
        };

        self.book_details = Some(details.clone());
        self.documents = documents;

        self.repaginate();

        Ok(details)
    }

    fn set_viewport(&mut self, width: u32, height: u32) -> Result<(), ViewerError> {
        self.viewport_width = width.max(100);
        self.viewport_height = height.max(100);
        self.repaginate();
        Ok(())
    }

    fn apply_render_options(&mut self, options: &ViewerRenderOptions) -> Result<(), ViewerError> {
        self.options = options.clone();
        self.repaginate();
        Ok(())
    }

    fn total_pages(&self) -> usize {
        self.pages.len()
    }

    fn render_page(&mut self, page_index: usize) -> Result<PageBitmap, ViewerError> {
        if self.pages.is_empty() {
            return Err(ViewerError::NotInitialized);
        }
        if page_index >= self.pages.len() {
            return Err(ViewerError::PageOutOfBounds {
                requested: page_index,
                total: self.pages.len(),
            });
        }

        let bg_color = self.options.theme.background_rgba();
        let mut bitmap = PageBitmap::new_blank(self.viewport_width, self.viewport_height, bg_color);

        // Desenha uma borda estética sutil ou marcas tipográficas
        let text_color = self.options.theme.text_rgba();
        let page = &self.pages[page_index];

        // Marca de progresso na base da página (como no KOReader)
        let bottom_y = self.viewport_height.saturating_sub(15);
        if bottom_y < self.viewport_height {
            let progress_width = (self.viewport_width as f32
                * ((page_index + 1) as f32 / self.pages.len() as f32))
                as u32;
            for x in 0..progress_width.min(self.viewport_width) {
                bitmap.set_pixel(x, bottom_y, text_color);
            }
        }

        // Renderização simbólica das linhas para o bitmap
        let margin_x = self.options.margin_px;
        let mut current_y = self.options.margin_px;
        let font_px = (self.options.font_size_pt * 1.333) as u32;
        let line_height = (font_px as f32 * self.options.line_height_multiplier) as u32;

        for line in &page.lines {
            if current_y + font_px >= self.viewport_height.saturating_sub(margin_x) {
                break;
            }
            if !line.text.is_empty() {
                // Desenha linha no buffer bitmap com peso tipográfico diferenciado para títulos
                let text_len_px = (line.text.len() as u32 * (font_px / 2))
                    .min(self.viewport_width.saturating_sub(margin_x * 2));
                for px in 0..text_len_px {
                    let x = margin_x + px;
                    bitmap.set_pixel(x, current_y + font_px, text_color);
                    if line.is_heading && current_y + font_px + 1 < self.viewport_height {
                        bitmap.set_pixel(x, current_y + font_px + 1, text_color);
                    }
                }
            }
            current_y += line_height;
        }

        Ok(bitmap)
    }

    fn table_of_contents(&self) -> &[TocNode] {
        &self.toc
    }
}

fn wrap_text_to_lines(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= max_chars {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn extract_attr_val(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    let start = tag.find(&pattern)? + pattern.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_tag_content(xml: &str, tag_name: &str) -> Option<String> {
    let open_pattern = format!("<{}", tag_name);
    let start = xml.find(&open_pattern)?;
    let tag_close = xml[start..].find('>')? + start + 1;
    let close_pattern = format!("</{}>", tag_name);
    let end = xml[tag_close..].find(&close_pattern)? + tag_close;
    Some(xml[tag_close..end].trim().to_string())
}
