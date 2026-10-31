//! Testes TDD do leitor interativo e da barra de status KOReader.

use std::path::PathBuf;

use any2epub_rs::epub::NativeEpub3Builder;
use any2epub_rs::viewer::{
    create_reader_engine, ViewerBackendKind, ViewerColorTheme, ViewerRenderOptions,
};
use any2epub_rs::{
    BookMetadata, Chapter, ChapterElement, ExtractedBookPayload,
};

fn generate_interactive_epub() -> PathBuf {
    let metadata = BookMetadata {
        uuid: String::from("55555555-6666-7777-8888-999999999999"),
        isbn: String::from("urn:isbn:978-0-123456-7-8"),
        title: String::from("Livro de Leitura Interativa"),
        authors: vec![String::from("Autor do Livro")],
        translator: String::new(),
        publisher: String::from("Editora"),
        publication_date: String::from("2026-10-30"),
        synopsis: String::from("Sinopse interativa."),
        language: String::from("pt-BR"),
        cover_jpeg_bytes: None,
    };

    let chapters = vec![
        Chapter {
            index: 1,
            number_prefix: String::from("1"),
            title: String::from("Capítulo 1: Fundações"),
            filename: String::from("chapter_001.xhtml"),
            elements: vec![
                ChapterElement::SubheadingH2 {
                    id: String::from("fund"),
                    number_prefix: String::from("1.1"),
                    title: String::from("Conceitos"),
                },
                ChapterElement::StandardParagraph(String::from("Texto do parágrafo 1.")),
                ChapterElement::StandardParagraph(String::from("Texto do parágrafo 2.")),
            ],
            subtopics: Vec::new(),
            footnotes: Vec::new(),
        },
    ];

    let payload = ExtractedBookPayload {
        metadata,
        chapters,
        figures: Vec::new(),
    };

    let builder = NativeEpub3Builder::from_payload(payload);
    let temp_path = std::env::temp_dir().join(format!("test_interactive_{}.epub", uuid::Uuid::new_v4()));
    builder.save(&temp_path).expect("Falha ao salvar EPUB");
    temp_path
}

#[test]
fn test_reader_engine_backend_switching_preserves_page_state() {
    let epub_path = generate_interactive_epub();

    // 1. Inicia com o motor nativo em Rust
    let mut native_engine = create_reader_engine(ViewerBackendKind::NativeRust)
        .expect("Falha ao criar native-rust");
    let details = native_engine
        .open_book(&epub_path)
        .expect("Falha ao abrir livro no native");
    assert_eq!(details.title, "Livro de Leitura Interativa");

    let options = ViewerRenderOptions {
        theme: ViewerColorTheme::Sepia,
        ..Default::default()
    };
    native_engine.apply_render_options(&options).unwrap();
    let total_pages_native = native_engine.total_pages();
    assert!(total_pages_native >= 1);

    // 2. Alterna em tempo real para o motor KOReader crengine
    let mut koreader_engine = create_reader_engine(ViewerBackendKind::KoreaderCrengine)
        .expect("Falha ao criar koreader-crengine");
    koreader_engine.open_book(&epub_path).unwrap();
    koreader_engine.apply_render_options(&options).unwrap();
    let total_pages_koreader = koreader_engine.total_pages();

    // Ambos os motores devem manter a consistência de contagem de páginas da obra
    assert_eq!(total_pages_native, total_pages_koreader);

    let _ = std::fs::remove_file(&epub_path);
}
