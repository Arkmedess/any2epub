//! Geração do documento de navegação hierárquico `OEBPS/nav.xhtml` (W3C EPUB 3.3 Navigation Document).

use crate::domain::ir::{Chapter, TocNode};
use crate::domain::metadata::BookMetadata;
use crate::utils::text::escape_xhtml_fast;

/// Renderiza recursivamente a árvore de [`TocNode`] como listas `<ol class="toc-sublist">` aninhadas.
pub fn render_toc_nodes_recursive(nodes: &[TocNode], indent: usize) -> String {
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
                render_toc_nodes_recursive(&node.children, indent + 4),
                pad = pad
            ));
        }
    }
    out.push_str(&format!("{}</ol>\n", pad));
    out
}

/// Constrói o arquivo `OEBPS/nav.xhtml` com sumário multinível completo.
pub fn build_nav_xhtml(metadata: &BookMetadata, chapters: &[Chapter]) -> String {
    let mut items = String::new();
    if metadata.cover_jpeg_bytes.is_some() {
        items.push_str("      <li><a href=\"cover.xhtml\">Capa</a></li>\n");
    }
    items.push_str("      <li><a href=\"sinopse.xhtml\">Sinopse e Ficha Editorial</a></li>\n");
    for ch in chapters {
        if ch.subtopics.is_empty() {
            items.push_str(&format!(
                "      <li><a href=\"{}\">{}</a></li>\n",
                ch.filename,
                escape_xhtml_fast(&ch.title)
            ));
        } else {
            let nested_ol = render_toc_nodes_recursive(&ch.subtopics, 8);
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
        lang = escape_xhtml_fast(&metadata.language),
        items = items
    )
}
