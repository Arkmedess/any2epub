//! Pipeline de reconstrução anatômica e heurísticas tipográficas para livros em PDF (`PdfProsePipeline`).

use std::collections::HashMap;

use rayon::prelude::*;

use crate::domain::ir::{
    insert_subtopic_node, Chapter, ChapterElement, FootnoteEntry, PdfWordBox, RawPdfPage,
    ReconstructedLine, TocNode,
};
use crate::domain::metadata::ExtractedFigure;
use crate::utils::regexes::{
    caption_regex, chapter_heading_regex, footnote_line_regex, inline_footnote_ref_regex,
    terminal_punctuation_regex,
};
use crate::utils::text::{
    escape_xhtml_fast, normalize_whitespace, slugify_anchor_id, strip_leading_tags,
};

/// Estrutura intermediária de uma página PDF pré-processada: `(linhas_reconstruidas, notas_de_rodape, figuras)`.
pub type PreprocessedPageData = (
    Vec<ReconstructedLine>,
    Vec<(String, String)>,
    Vec<ExtractedFigure>,
);

/// Pipeline paralelo de extração geométrica, detecção de moda estatística e costura de páginas PDF.
pub struct PdfProsePipeline {
    /// Tamanho de fonte predominante do corpo de texto (em pontos).
    pub body_font_size: f32,
    /// Percentual de corte para expurgo de cabeçalhos e rodapés (ex: `0.05` = 5%).
    pub margin_cutoff: f32,
}

impl PdfProsePipeline {
    /// Instancia o pipeline calculando automaticamente a moda do tamanho de fonte a partir das páginas.
    pub fn from_pages(pages: &[RawPdfPage], margin_cutoff: f32) -> Self {
        let body_font_size = Self::detect_body_font_size(pages);
        Self {
            body_font_size,
            margin_cutoff,
        }
    }

    /// Calcula em paralelo a moda estatística do tamanho de fonte ponderada pelo número de caracteres.
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

    /// Pré-processa todas as páginas em paralelo via `rayon`, filtrando margens, agrupando linhas
    /// e separando notas de rodapé na base da página.
    pub fn preprocess_pages_parallel(&self, pages: &[RawPdfPage]) -> Vec<PreprocessedPageData> {
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
                            let marker = caps[1].trim_matches(|c| c == '[' || c == ']').to_string();
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

    fn group_words_into_lines(
        words: &[&PdfWordBox],
        body_font_size: f32,
    ) -> Vec<ReconstructedLine> {
        if words.is_empty() {
            return Vec::new();
        }

        let mut sorted = words.to_vec();
        sorted.sort_by(|a, b| {
            let y_cmp = a
                .top
                .partial_cmp(&b.top)
                .unwrap_or(std::cmp::Ordering::Equal);
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
                html_str = format!(
                    "{}{}",
                    escape_xhtml_fast(&dc.to_string()),
                    html_str.trim_start()
                );
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

    /// Costura sequencialmente as páginas pré-processadas em capítulos estruturados,
    /// aplicando des-hifenização consciente de caixa e vinculando notas de rodapé.
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
