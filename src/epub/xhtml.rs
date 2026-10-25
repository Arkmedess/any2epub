//! Serialização de páginas XHTML5 (`cover.xhtml`, `sinopse.xhtml` e `chapter_XXX.xhtml`).

use crate::domain::ir::{CalloutKind, Chapter, ChapterElement};
use crate::domain::metadata::BookMetadata;
use crate::utils::text::escape_xhtml_fast;

/// Gera o documento XHTML5 da capa (`OEBPS/cover.xhtml`).
pub fn build_cover_xhtml(metadata: &BookMetadata) -> String {
    let safe_title = escape_xhtml_fast(&metadata.title);
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
        lang = escape_xhtml_fast(&metadata.language),
        title = safe_title
    )
}

/// Gera o documento XHTML5 de sinopse e ficha editorial (`OEBPS/sinopse.xhtml`).
pub fn build_synopsis_xhtml(metadata: &BookMetadata) -> String {
    let safe_title = escape_xhtml_fast(&metadata.title);
    let safe_authors = escape_xhtml_fast(&metadata.authors.join(", "));
    let safe_translator = escape_xhtml_fast(&metadata.translator);
    let safe_publisher = escape_xhtml_fast(&metadata.publisher);
    let safe_isbn = escape_xhtml_fast(&metadata.isbn);
    let safe_synopsis = escape_xhtml_fast(&metadata.synopsis);

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
        lang = escape_xhtml_fast(&metadata.language),
        title = safe_title,
        authors = safe_authors,
        translator = safe_translator,
        publisher = safe_publisher,
        isbn = safe_isbn,
        synopsis = safe_synopsis
    )
}

/// Gera o documento XHTML5 completo de um capítulo (`OEBPS/chapter_XXX.xhtml`).
pub fn build_chapter_xhtml(metadata: &BookMetadata, chapter: &Chapter) -> String {
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
                let class_attr = if idx == 0 {
                    " class=\"first-para\""
                } else {
                    ""
                };
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
        lang = escape_xhtml_fast(&metadata.language),
        title = safe_title,
        body = body_buf
    )
}
