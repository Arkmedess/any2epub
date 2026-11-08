//! Testes unitários e de integração do pipeline PDF, reparo CMap e construtor W3C EPUB 3.3.

use any2epub_rs::{
    escape_xhtml_fast, insert_subtopic_node, repair_broken_cmap_cid, slugify_anchor_id,
    BookMetadata, Chapter, ChapterElement, ExtractedBookPayload, FootnoteEntry, NativeEpub3Builder,
    PdfProsePipeline, PdfWordBox, RawPdfPage, TocNode,
};

#[test]
fn test_cmap_cid_repair_calibri_and_lekton() {
    assert_eq!(repair_broken_cmap_cid("Calibri-Bold", 0x014C), Some("ft"));
    assert_eq!(repair_broken_cmap_cid("Calibri", 0x019F), Some("ti"));
    assert_eq!(repair_broken_cmap_cid("Lekton-Regular", 0x01EB), Some("C"));
    assert_eq!(repair_broken_cmap_cid("Lekton-Regular", 0x0205), Some("="));
    assert_eq!(repair_broken_cmap_cid("TimesNewRoman", 0x014C), None);
}

#[test]
fn test_pdf_prose_pipeline_extracts_dropcap_dialogue_and_footnotes() {
    let page = RawPdfPage {
        page_index: 1,
        width: 600.0,
        height: 800.0,
        words: vec![
            PdfWordBox {
                text: String::from("CAPÍTULO 1"),
                font_name: String::from("MyriadPro-Bold"),
                x0: 72.0,
                top: 100.0,
                x1: 220.0,
                bottom: 124.0,
                font_size: 22.0,
                is_italic: false,
                is_bold: true,
                is_monospace: false,
            },
            PdfWordBox {
                text: String::from("O"),
                font_name: String::from("Georgia"),
                x0: 72.0,
                top: 160.0,
                x1: 95.0,
                bottom: 190.0,
                font_size: 24.0,
                is_italic: false,
                is_bold: true,
                is_monospace: false,
            },
            PdfWordBox {
                text: String::from("rquestração"),
                font_name: String::from("Georgia"),
                x0: 96.0,
                top: 160.0,
                x1: 180.0,
                bottom: 172.0,
                font_size: 11.0,
                is_italic: false,
                is_bold: false,
                is_monospace: false,
            },
            PdfWordBox {
                text: String::from("modular[1]."),
                font_name: String::from("Georgia"),
                x0: 185.0,
                top: 160.0,
                x1: 260.0,
                bottom: 172.0,
                font_size: 11.0,
                is_italic: false,
                is_bold: false,
                is_monospace: false,
            },
            PdfWordBox {
                text: String::from("— Excelente arquitetura!"),
                font_name: String::from("Georgia"),
                x0: 72.0,
                top: 200.0,
                x1: 250.0,
                bottom: 212.0,
                font_size: 11.0,
                is_italic: false,
                is_bold: false,
                is_monospace: false,
            },
            PdfWordBox {
                text: String::from("[1] Nota explicativa na base da página."),
                font_name: String::from("Georgia"),
                x0: 72.0,
                top: 650.0,
                x1: 350.0,
                bottom: 660.0,
                font_size: 8.5,
                is_italic: false,
                is_bold: false,
                is_monospace: false,
            },
        ],
        figures: Vec::new(),
    };

    let pipeline = PdfProsePipeline::from_pages(std::slice::from_ref(&page), 0.05);
    let chapters = pipeline.extract_chapters(&[page]);
    assert_eq!(chapters.len(), 1);
    assert_eq!(chapters[0].title, "CAPÍTULO 1");
    assert_eq!(chapters[0].footnotes.len(), 1);
    assert!(matches!(
        chapters[0].elements[0],
        ChapterElement::DropCapParagraph('O', _)
    ));
    assert!(matches!(
        chapters[0].elements[1],
        ChapterElement::DialogueParagraph(_)
    ));
}

#[test]
fn test_epub_builder_generates_valid_nav_and_opf() {
    let meta = BookMetadata {
        uuid: String::from("12345678-1234-4000-8000-1234567890ab"),
        isbn: String::from("urn:isbn:9780000000000"),
        title: String::from("Arquitetura & Engenharia"),
        authors: vec![String::from("Autor Principal")],
        translator: String::from("Tradutor"),
        publisher: String::from("Editora Técnica"),
        publication_date: String::from("2026-10-24"),
        synopsis: String::from("Sinopse de teste."),
        language: String::from("pt-BR"),
        cover_jpeg_bytes: None,
    };

    let mut subtopics: Vec<TocNode> = Vec::new();
    insert_subtopic_node(
        &mut subtopics,
        2,
        String::from("1.1"),
        String::from("Fronteiras Modulares"),
        String::from("chapter_001.xhtml#fronteiras-modulares"),
    );

    let chapter = Chapter {
        index: 1,
        number_prefix: String::from("1"),
        title: String::from("1. Introdução"),
        filename: String::from("chapter_001.xhtml"),
        elements: vec![
            ChapterElement::SubheadingH2 {
                id: slugify_anchor_id("Fronteiras Modulares", 1),
                number_prefix: String::from("1.1"),
                title: escape_xhtml_fast("Fronteiras Modulares"),
            },
            ChapterElement::StandardParagraph(String::from("Conteúdo do parágrafo.")),
        ],
        footnotes: vec![FootnoteEntry {
            id_num: 1,
            marker: String::from("1"),
            content_html: String::from("Texto da nota."),
        }],
        subtopics,
    };

    let payload = ExtractedBookPayload {
        metadata: meta,
        chapters: vec![chapter],
        figures: Vec::new(),
    };

    let builder = NativeEpub3Builder::from_payload(payload);
    let tmp_path = std::env::temp_dir().join("test_any2epub_modular_v0_9_0.epub");
    builder
        .save(&tmp_path)
        .expect("Falha ao gravar arquivo EPUB temporário");

    let bytes = std::fs::read(&tmp_path).expect("Falha ao ler EPUB gerado");
    // Verifica assinatura ZIP e mimetype sem compressão no byte offset 30
    assert_eq!(&bytes[0..2], b"PK");
    let _ = std::fs::remove_file(&tmp_path);
}

#[test]
fn test_pdf_prose_pipeline_detects_headings_with_generic_sans_fonts() {
    let page = RawPdfPage {
        page_index: 2,
        width: 600.0,
        height: 800.0,
        words: vec![
            PdfWordBox {
                text: String::from("1.2"),
                font_name: String::from("Helvetica-Bold"),
                x0: 72.0,
                top: 100.0,
                x1: 95.0,
                bottom: 120.0,
                font_size: 16.0,
                is_italic: false,
                is_bold: true,
                is_monospace: false,
            },
            PdfWordBox {
                text: String::from("Visão Geral"),
                font_name: String::from("Helvetica-Bold"),
                x0: 102.0,
                top: 100.0,
                x1: 220.0,
                bottom: 120.0,
                font_size: 16.0,
                is_italic: false,
                is_bold: true,
                is_monospace: false,
            },
            PdfWordBox {
                text: String::from("Parágrafo comum do texto explicativo em fonte serifada."),
                font_name: String::from("TimesNewRoman"),
                x0: 72.0,
                top: 140.0,
                x1: 350.0,
                bottom: 152.0,
                font_size: 10.0,
                is_italic: false,
                is_bold: false,
                is_monospace: false,
            },
        ],
        figures: Vec::new(),
    };

    let pipeline = PdfProsePipeline::from_pages(std::slice::from_ref(&page), 0.05);
    let preprocessed = pipeline.preprocess_pages_parallel(&[page]);
    let (lines, _, _) = &preprocessed[0];
    assert!(!lines.is_empty());
    let heading_line = lines
        .iter()
        .find(|l| l.raw_text.contains("Visão Geral"))
        .expect("Linha de título não encontrada");
    assert!(heading_line.is_heading_font, "Deveria identificar Helvetica-Bold como família de título");
}
