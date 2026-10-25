//! Geração do Package Document `OEBPS/content.opf` com metadados Dublin Core, manifesto e spine.

use crate::domain::ir::Chapter;
use crate::domain::metadata::{BookMetadata, ExtractedFigure};
use crate::utils::text::escape_xhtml_fast;

/// Constrói o arquivo XML `OEBPS/content.opf` seguindo a especificação W3C EPUB 3.3.
pub fn build_content_opf(
    metadata: &BookMetadata,
    chapters: &[Chapter],
    figures: &[ExtractedFigure],
) -> String {
    let mut manifest_items = String::from(
        "    <item id=\"style\" href=\"style.css\" media-type=\"text/css\"/>\n    <item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n",
    );
    let mut spine_items = String::new();

    if metadata.cover_jpeg_bytes.is_some() {
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

    for fig in figures {
        manifest_items.push_str(&format!(
            "    <item id=\"{}\" href=\"{}\" media-type=\"image/jpeg\"/>\n",
            fig.id, fig.href
        ));
    }

    for ch in chapters {
        let item_id = format!("chap_{:03}", ch.index);
        manifest_items.push_str(&format!(
            "    <item id=\"{}\" href=\"{}\" media-type=\"application/xhtml+xml\"/>\n",
            item_id, ch.filename
        ));
        spine_items.push_str(&format!("    <itemref idref=\"{}\"/>\n", item_id));
    }

    let creators_xml: String = metadata
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
        lang = escape_xhtml_fast(&metadata.language),
        isbn = escape_xhtml_fast(&metadata.isbn),
        uuid = escape_xhtml_fast(&metadata.uuid),
        title = escape_xhtml_fast(&metadata.title),
        creators = creators_xml,
        translator = escape_xhtml_fast(&metadata.translator),
        publisher = escape_xhtml_fast(&metadata.publisher),
        date = escape_xhtml_fast(&metadata.publication_date),
        synopsis = escape_xhtml_fast(&metadata.synopsis),
        manifest = manifest_items,
        spine = spine_items
    )
}
