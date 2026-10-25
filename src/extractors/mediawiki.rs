//! Extrator em grafo BFS para portais MediaWiki / ArchWiki (`MediaWikiExtractor`).

use std::collections::{HashSet, VecDeque};

use scraper::{ElementRef, Html, Selector};

use crate::domain::ir::{
    insert_subtopic_node, CalloutKind, Chapter, ChapterElement, ExtractedBookPayload, TocNode,
};
use crate::domain::metadata::BookMetadata;
use crate::domain::traits::ContentSourceExtractor;
use crate::utils::text::{escape_xhtml_fast, normalize_whitespace, slugify_anchor_id};

/// Extrator em largura (BFS) para wikis baseadas no motor MediaWiki (ex: Arch Linux Wiki).
pub struct MediaWikiExtractor {
    /// URL raiz a partir da qual a busca em largura (BFS) inicia.
    pub root_url: String,
    /// Profundidade máxima de navegação em links internos.
    pub max_depth: usize,
    /// Limite máximo de capítulos (artigos) a coletar.
    pub max_chapters: usize,
    /// Metadados editoriais padrão para o pacote EPUB.
    pub default_metadata: BookMetadata,
}

impl MediaWikiExtractor {
    /// Extrai o esquema e domínio de origem (origin) de uma URL HTTP/HTTPS.
    pub fn extract_origin(url: &str) -> String {
        if let Some(scheme_end) = url.find("://") {
            let rest = &url[scheme_end + 3..];
            if let Some(slash_idx) = rest.find('/') {
                return url[..scheme_end + 3 + slash_idx].to_string();
            }
        }
        url.trim_end_matches('/').to_string()
    }

    /// Valida e resolve um atributo `href` de artigo MediaWiki ignorando namespaces especiais e páginas de discussão.
    pub fn resolve_valid_wiki_article(origin: &str, href: &str) -> Option<String> {
        let clean = href.split('#').next()?.trim();
        if clean.is_empty() || clean.contains('?') {
            return None;
        }

        let path = if let Some(stripped) = clean.strip_prefix(origin) {
            stripped
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

    /// Extrai o conteúdo de um bloco `<pre>` da ArchWiki preservando marcações inline (`<em>`, `<strong>`).
    pub fn extract_archwiki_pre_block(pre_node: ElementRef) -> String {
        let mut out = String::new();
        for child in pre_node.children() {
            if let Some(el) = ElementRef::wrap(child) {
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

    /// Extrai uma tabela HTML `.wikitable` convertendo cabeçalhos `<th>` e células `<td>` em [`ChapterElement::DataTable`].
    pub fn extract_wikitable(table_node: ElementRef) -> Option<ChapterElement> {
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

    /// Extrai o ID de âncora nativo do MediaWiki (`<span class="mw-headline" id="...">` ou atributo `id` no `<hN>`).
    fn extract_heading_anchor_id(el: ElementRef, raw_title: &str, seq: usize) -> String {
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

    /// Extrai elementos semânticos e constrói a árvore recursiva de subtópicos (`1.1`, `1.1.1`, `1.1.1.1`).
    pub fn parse_mediawiki_article(
        origin: &str,
        html_doc: &Html,
        chapter_index: usize,
        chapter_filename: &str,
    ) -> (String, Vec<ChapterElement>, Vec<TocNode>, Vec<String>) {
        let title_sel = Selector::parse("h1#firstHeading, .mw-page-title-main, h1").unwrap();
        let title = html_doc
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("Article {}", chapter_index));

        let parser_output_sel = Selector::parse("#mw-content-text .mw-parser-output").unwrap();
        let fallback_content_sel = Selector::parse("#mw-content-text").unwrap();
        let link_sel = Selector::parse("a[href]").unwrap();

        let mut discovered_links = Vec::new();
        let mut elements = Vec::new();
        let mut subtopics: Vec<TocNode> = Vec::new();

        let mut h2_num = 0usize;
        let mut h3_num = 0usize;
        let mut h4_num = 0usize;
        let mut anchor_seq = 1usize;

        let content_root_opt = html_doc
            .select(&parser_output_sel)
            .next()
            .or_else(|| html_doc.select(&fallback_content_sel).next());

        if let Some(content_root) = content_root_opt {
            for a_el in content_root.select(&link_sel) {
                if let Some(href) = a_el.value().attr("href") {
                    if let Some(valid_url) = Self::resolve_valid_wiki_article(origin, href) {
                        discovered_links.push(valid_url);
                    }
                }
            }

            for child in content_root.children() {
                let Some(el) = ElementRef::wrap(child) else {
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
                    let body =
                        escape_xhtml_fast(&normalize_whitespace(&el.text().collect::<String>()));
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
                            elements
                                .push(ChapterElement::StandardParagraph(escape_xhtml_fast(&txt)));
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
        let client = reqwest::blocking::Client::builder()
            .user_agent("pdf2epub-rs/0.9.0 (+https://www.w3.org/TR/epub-33/)")
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
