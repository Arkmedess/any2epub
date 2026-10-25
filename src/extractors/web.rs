//! Extrator calibrado para mdBook, Sphinx, Pro Git AsciiDoc, MDN Yari e artigos Web (`WebSiteExtractor`).

use std::collections::HashSet;

use rayon::prelude::*;
use scraper::{ElementRef, Html, Selector};

use crate::domain::ir::{
    insert_subtopic_node, CalloutKind, Chapter, ChapterElement, ExtractedBookPayload, TocNode,
};
use crate::domain::metadata::BookMetadata;
use crate::domain::traits::ContentSourceExtractor;
use crate::extractors::mediawiki::MediaWikiExtractor;
use crate::utils::text::{escape_xhtml_fast, normalize_whitespace, slugify_anchor_id};

/// Extrator profundo de portais de documentação técnica e artigos Web.
pub struct WebSiteExtractor {
    /// URL inicial do livro, guia ou artigo.
    pub start_url: String,
    /// Indica se deve varrer recursivamente o sumário de capítulos (TOC).
    pub crawl_docs_toc: bool,
    /// Seletor CSS customizado para descoberta de links do sumário.
    pub toc_selector: String,
    /// Metadados editoriais base.
    pub default_metadata: BookMetadata,
}

impl WebSiteExtractor {
    /// Normaliza uma URL base garantindo barra final `/` e removendo fragmentos ou nomes de arquivo `.html`.
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

    /// Detecta automaticamente o perfil de árvore DOM com base na URL de entrada.
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

    /// Retorna o seletor CSS composto para varredura profunda da árvore de capítulos (TOC)
    /// adaptado ao motor de documentação detectado na raiz.
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

    /// Verifica se a URL candidata pertence ao mesmo escopo hierárquico do livro/guia.
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

    /// Resolve `href` (absoluto, relativo ao diretório `./` ou relativo à raiz `/`) segundo a RFC 3986.
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

    /// Remove prefixos de botões de toolbar de código do MDN Yari (`js Copy`, `html Copy`, etc.) e Sphinx.
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

    /// Extrai o prefixo numérico do link da barra lateral (ex: `"4.1."` -> `"4.1"`) ou a partir do slug da URL.
    pub fn extract_chapter_number_prefix(a_text: &str, url: &str, fallback_idx: usize) -> String {
        let first_tok = a_text.split_whitespace().next().unwrap_or("");
        if first_tok.chars().next().is_some_and(|c| c.is_ascii_digit()) {
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

    /// Extrai metadados OpenGraph (`og:title`, `og:description`, `author`, `og:image`) da página inicial.
    pub fn extract_opengraph_metadata(&self, html_doc: &Html) -> BookMetadata {
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

    /// Extrai blocos de código do mdBook preservando avisos do Ferris e omitindo linhas ocultas (`.boring`).
    pub fn extract_mdbook_code_block(pre_node: ElementRef) -> String {
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
            if let Some(el) = ElementRef::wrap(child) {
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

    /// Verifica se um nó `<li>`, `<dd>` ou `<blockquote>` possui sub-blocos internos (`<p>`, `<pre>`, `<dl>`, `<table>`).
    pub fn has_nested_block_children(node: ElementRef) -> bool {
        let inner_sel = Selector::parse("p, pre, table, dl, ul, ol, blockquote").unwrap();
        node.select(&inner_sel).next().is_some()
    }

    /// Verifica se um nó `<p>` ou `<li>` está dentro de uma tabela ou caixa de destaque (`notecard` / `admonitionblock`).
    pub fn is_inside_callout_or_table(node: ElementRef) -> bool {
        let mut curr = node.parent();
        while let Some(parent) = curr {
            if let Some(el) = ElementRef::wrap(parent) {
                let tag = el.value().name();
                if matches!(tag, "table" | "thead" | "tbody" | "tr" | "td" | "th") {
                    return true;
                }
                if tag == "div" || tag == "aside" {
                    let cl = el.value().attr("class").unwrap_or("");
                    if cl.contains("notecard")
                        || cl.contains("admonitionblock")
                        || cl.contains("admonition")
                        || cl
                            .split_whitespace()
                            .any(|c| c == "note" || c == "warning" || c == "tip")
                    {
                        return true;
                    }
                }
            }
            curr = parent.parent();
        }
        false
    }

    /// Extrai todos os elementos na raiz semântica do capítulo preservando `<p>`, `<li>`, `<dt>`, `<dd>`,
    /// `<pre>`, `<table>`, `CalloutBox` e subtópicos hierárquicos (`H2 -> H3 -> H4`).
    pub fn extract_clean_article_elements(
        &self,
        html_doc: &Html,
        chapter_index: usize,
        chapter_prefix: &str,
        chapter_filename: &str,
    ) -> (String, Vec<ChapterElement>, Vec<TocNode>) {
        let title_sel = Selector::parse(
            "#content main h1, article.main-page-content h1, div#main div#content h1, \
             div.body[role=\"main\"] h1, article h1, main h1, h1",
        )
        .unwrap();
        let raw_h1 = html_doc
            .select(&title_sel)
            .next()
            .map(|el| {
                el.text()
                    .collect::<String>()
                    .replace('¶', "")
                    .trim()
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("Chapter {}", chapter_index));

        let chapter_title = if !chapter_prefix.is_empty()
            && !raw_h1.starts_with(chapter_prefix)
            && chapter_prefix
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
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
                    || classes
                        .split_whitespace()
                        .any(|c| c == "note" || c == "warning" || c == "tip")
                {
                    let callout_txt = normalize_whitespace(&node.text().collect::<String>());
                    if !callout_txt.is_empty() {
                        let (kind, title) =
                            if classes.contains("warning") || classes.contains("caution") {
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

            if matches!(tag, "p" | "li" | "dt" | "dd" | "blockquote")
                && Self::is_inside_callout_or_table(node)
            {
                continue;
            }

            if matches!(tag, "li" | "dd" | "blockquote") && Self::has_nested_block_children(node) {
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
                    let num_prefix = format!("{}.{}.{}.{}", chapter_prefix, h2_idx, h3_idx, h4_idx);
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
                "figcaption" | "span" => elements.push(ChapterElement::FigureCaption(
                    escape_xhtml_fast(&normalize_whitespace(trimmed)),
                )),
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

    /// Descobre recursivamente (BFS até 2 níveis de índice) todas as URLs de capítulos e subcapítulos
    /// da documentação (mdBook, Sphinx, Pro Git AsciiDoc, MDN Yari).
    pub fn crawl_docs_toc_bfs(
        &self,
        client: &reqwest::blocking::Client,
        base_dir_url: &str,
        root_doc: &Html,
    ) -> Vec<(String, String)> {
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
