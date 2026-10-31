//! Testes dos comandos de CLI (`validate`, `view`) e contratos de automação.

use std::path::PathBuf;

use any2epub_rs::cli::{handle_validate_command, handle_view_command};
use any2epub_rs::epub::NativeEpub3Builder;
use any2epub_rs::{
    BookMetadata, Chapter, ChapterElement, ExtractedBookPayload,
};

fn generate_fixture_epub() -> PathBuf {
    let metadata = BookMetadata {
        uuid: String::from("33333333-4444-5555-6666-777777777777"),
        isbn: String::from("urn:isbn:978-1-234567-8-9"),
        title: String::from("CLI Test Book"),
        authors: vec![String::from("CLI Author")],
        translator: String::new(),
        publisher: String::from("CLI Publisher"),
        publication_date: String::from("2026-10-30"),
        synopsis: String::from("Sinopse para teste de CLI."),
        language: String::from("pt-BR"),
        cover_jpeg_bytes: None,
    };

    let chapters = vec![Chapter {
        index: 1,
        number_prefix: String::from("1"),
        title: String::from("Capítulo 1: Introdução"),
        filename: String::from("chapter_001.xhtml"),
        elements: vec![
            ChapterElement::SubheadingH2 {
                id: String::from("intro"),
                number_prefix: String::from("1.1"),
                title: String::from("Introdução"),
            },
            ChapterElement::StandardParagraph(String::from(
                "Linha de conteúdo para testes de validação e visualização.",
            )),
        ],
        subtopics: Vec::new(),
        footnotes: Vec::new(),
    }];

    let payload = ExtractedBookPayload {
        metadata,
        chapters,
        figures: Vec::new(),
    };

    let builder = NativeEpub3Builder::from_payload(payload);
    let temp_path = std::env::temp_dir().join(format!("cli_test_{}.epub", uuid::Uuid::new_v4()));
    builder.save(&temp_path).expect("Falha ao salvar fixture EPUB");
    temp_path
}

#[test]
fn test_cli_handle_validate_and_view_commands() {
    let epub_path = generate_fixture_epub();

    // 1. Testa validação em modo texto
    assert!(handle_validate_command(&epub_path, false).is_ok());

    // 2. Testa validação em modo JSON
    assert!(handle_validate_command(&epub_path, true).is_ok());

    // 3. Testa visualização no motor nativo Rust
    assert!(handle_view_command(&epub_path, "native", "sepia", 0, false).is_ok());

    // 4. Testa visualização no motor KOReader crengine
    assert!(handle_view_command(&epub_path, "koreader", "night", 0, false).is_ok());

    let _ = std::fs::remove_file(&epub_path);
}
