//! Extrator e parser de documentos Markdown locais (`LocalMarkdownExtractor`).

use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::ir::{
    insert_subtopic_node, CalloutKind, Chapter, ChapterElement, ExtractedBookPayload,
};
use crate::domain::metadata::BookMetadata;
use crate::domain::traits::ContentSourceExtractor;
use crate::utils::text::{escape_xhtml_fast, slugify_anchor_id};

/// Extrator para arquivos `.md` individuais ou diretórios de documentação Markdown.
pub struct LocalMarkdownExtractor {
    /// Caminho do arquivo ou diretório de entrada.
    pub input_path: PathBuf,
    /// Metadados base para o livro.
    pub default_metadata: BookMetadata,
}

impl LocalMarkdownExtractor {
    /// Processa o conteúdo de uma string Markdown em um [`Chapter`] estruturado.
    pub fn parse_markdown_to_chapter(
        content: &str,
        chapter_index: usize,
        filename: String,
    ) -> Chapter {
        let mut chapter_title = format!("Capítulo {}", chapter_index);
        let mut elements = Vec::new();
        let mut subtopics = Vec::new();

        let mut lines = content.lines().peekable();
        let mut h2_counter = 0;
        let mut h3_counter = 0;
        let mut h4_counter = 0;

        while let Some(line) = lines.next() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // 1. Títulos Markdown (# -> H1, ## -> H2, ### -> H3, #### -> H4)
            if let Some(heading_text) = trimmed.strip_prefix("# ") {
                chapter_title = heading_text.trim().to_string();
                continue;
            }

            if let Some(heading_text) = trimmed.strip_prefix("## ") {
                h2_counter += 1;
                h3_counter = 0;
                h4_counter = 0;
                let raw_text = heading_text.trim();
                let number_prefix = extract_or_build_prefix(raw_text, &format!("{}.{}", chapter_index, h2_counter));
                let clean_text = strip_leading_number_prefix(raw_text);
                let anchor_id = slugify_anchor_id(clean_text, h2_counter);
                let href = format!("{}#{}", filename, anchor_id);

                insert_subtopic_node(
                    &mut subtopics,
                    2,
                    number_prefix.clone(),
                    clean_text.to_string(),
                    href,
                );

                elements.push(ChapterElement::SubheadingH2 {
                    id: anchor_id,
                    number_prefix,
                    title: clean_text.to_string(),
                });
                continue;
            }

            if let Some(heading_text) = trimmed.strip_prefix("### ") {
                h3_counter += 1;
                h4_counter = 0;
                let raw_text = heading_text.trim();
                let number_prefix = extract_or_build_prefix(
                    raw_text,
                    &format!("{}.{}.{}", chapter_index, h2_counter, h3_counter),
                );
                let clean_text = strip_leading_number_prefix(raw_text);
                let anchor_id = slugify_anchor_id(clean_text, h3_counter);
                let href = format!("{}#{}", filename, anchor_id);

                insert_subtopic_node(
                    &mut subtopics,
                    3,
                    number_prefix.clone(),
                    clean_text.to_string(),
                    href,
                );

                elements.push(ChapterElement::SubheadingH3 {
                    id: anchor_id,
                    number_prefix,
                    title: clean_text.to_string(),
                });
                continue;
            }

            if let Some(heading_text) = trimmed.strip_prefix("#### ") {
                h4_counter += 1;
                let raw_text = heading_text.trim();
                let number_prefix = extract_or_build_prefix(
                    raw_text,
                    &format!("{}.{}.{}.{}", chapter_index, h2_counter, h3_counter, h4_counter),
                );
                let clean_text = strip_leading_number_prefix(raw_text);
                let anchor_id = slugify_anchor_id(clean_text, h4_counter);
                let href = format!("{}#{}", filename, anchor_id);

                insert_subtopic_node(
                    &mut subtopics,
                    4,
                    number_prefix.clone(),
                    clean_text.to_string(),
                    href,
                );

                elements.push(ChapterElement::SubheadingH4 {
                    id: anchor_id,
                    number_prefix,
                    title: clean_text.to_string(),
                });
                continue;
            }

            // 2. Blocos de Código (```lang ... ```)
            if trimmed.starts_with("```") {
                let mut code_lines = Vec::new();
                for code_line in lines.by_ref() {
                    if code_line.trim().starts_with("```") {
                        break;
                    }
                    code_lines.push(code_line);
                }
                elements.push(ChapterElement::CodeBlock(code_lines.join("\n")));
                continue;
            }

            // 3. Callout Boxes / Alertas (> [!NOTE], > [!TIP], > [!WARNING])
            if trimmed.starts_with("> [!") {
                let kind = if trimmed.contains("TIP") {
                    CalloutKind::Tip
                } else if trimmed.contains("WARNING") || trimmed.contains("CAUTION") {
                    CalloutKind::Warning
                } else {
                    CalloutKind::Note
                };

                let title = match kind {
                    CalloutKind::Tip => "Dica Importante",
                    CalloutKind::Warning => "Aviso",
                    CalloutKind::Note => "Nota Técnica",
                }
                .to_string();

                let mut body_lines = Vec::new();
                while let Some(next_line) = lines.peek() {
                    let next_trimmed = next_line.trim();
                    if next_trimmed.starts_with('>') {
                        let text = next_trimmed.trim_start_matches('>').trim();
                        body_lines.push(escape_xhtml_fast(text));
                        lines.next();
                    } else {
                        break;
                    }
                }

                elements.push(ChapterElement::CalloutBox {
                    kind,
                    title,
                    body_html: format!("<p>{}</p>", body_lines.join("<br/>")),
                });
                continue;
            }

            // 4. Blockquotes genéricos (> ...)
            if let Some(quote_text) = trimmed.strip_prefix("> ") {
                elements.push(ChapterElement::CalloutBox {
                    kind: CalloutKind::Note,
                    title: "Citação".to_string(),
                    body_html: format!("<p>{}</p>", escape_xhtml_fast(quote_text.trim())),
                });
                continue;
            }

            // 5. Itens de Lista (- item, * item, 1. item)
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                let item_text = &trimmed[2..];
                elements.push(ChapterElement::StandardParagraph(format!(
                    "• {}",
                    escape_xhtml_fast(item_text.trim())
                )));
                continue;
            }

            // 6. Parágrafos comuns de prosa
            elements.push(ChapterElement::StandardParagraph(escape_xhtml_fast(trimmed)));
        }

        Chapter {
            index: chapter_index,
            number_prefix: chapter_index.to_string(),
            title: chapter_title,
            filename,
            elements,
            footnotes: Vec::new(),
            subtopics,
        }
    }

    /// Coleta arquivos markdown a processar (suporta arquivo individual ou diretório recursivo).
    fn collect_markdown_files(&self) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut result = Vec::new();
        if self.input_path.is_file() {
            result.push(self.input_path.clone());
        } else if self.input_path.is_dir() {
            collect_md_recursive(&self.input_path, &mut result)?;
            result.sort();
        }
        Ok(result)
    }
}

impl ContentSourceExtractor for LocalMarkdownExtractor {
    fn extract_payload(&self) -> Result<ExtractedBookPayload, Box<dyn std::error::Error>> {
        let files = self.collect_markdown_files()?;
        if files.is_empty() {
            return Err(format!(
                "Nenhum arquivo Markdown encontrado em: {}",
                self.input_path.display()
            )
            .into());
        }

        let mut chapters = Vec::new();
        for (idx, file_path) in files.iter().enumerate() {
            let content = fs::read_to_string(file_path)?;
            let chapter_index = idx + 1;
            let filename = format!("chapter_{:03}.xhtml", chapter_index);
            let chapter = Self::parse_markdown_to_chapter(&content, chapter_index, filename);
            chapters.push(chapter);
        }

        Ok(ExtractedBookPayload {
            metadata: self.default_metadata.clone(),
            chapters,
            figures: Vec::new(),
        })
    }
}

fn collect_md_recursive(dir: &Path, acc: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_md_recursive(&path, acc)?;
        } else if let Some(ext) = path.extension() {
            if ext == "md" || ext == "markdown" {
                acc.push(path);
            }
        }
    }
    Ok(())
}

fn extract_or_build_prefix(title: &str, fallback: &str) -> String {
    if let Some(first_word) = title.split_whitespace().next() {
        if first_word.chars().all(|c| c.is_ascii_digit() || c == '.') && first_word.contains('.') {
            return first_word.trim_end_matches('.').to_string();
        }
    }
    fallback.to_string()
}

fn strip_leading_number_prefix(title: &str) -> &str {
    if let Some(first_word) = title.split_whitespace().next() {
        if first_word.chars().all(|c| c.is_ascii_digit() || c == '.') && first_word.contains('.') {
            return title[first_word.len()..].trim_start();
        }
    }
    title
}
