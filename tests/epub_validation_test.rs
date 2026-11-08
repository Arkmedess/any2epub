//! Testes TDD do validador de conformidade W3C EPUB 3.3 (`NativeEpubValidator`).

use std::fs::File;
use std::io::Write;

use any2epub_rs::epub::validator::NativeEpubValidator;
use any2epub_rs::{
    BookMetadata, Chapter, ChapterElement, ExtractedBookPayload, NativeEpub3Builder,
};

fn sample_payload() -> ExtractedBookPayload {
    let metadata = BookMetadata {
        uuid: String::from("11111111-2222-3333-4444-555555555555"),
        isbn: String::from("urn:isbn:978-0-000000-0-0"),
        title: String::from("Livro de Teste de Validação"),
        authors: vec![String::from("Autor de Teste")],
        translator: String::new(),
        publisher: String::from("Editora Teste"),
        publication_date: String::from("2026-10-30"),
        synopsis: String::from("Sinopse para teste."),
        language: String::from("pt-BR"),
        cover_jpeg_bytes: None,
    };

    let chapters = vec![Chapter {
        index: 1,
        number_prefix: String::from("1"),
        title: String::from("Fundações"),
        filename: String::from("chapter_001.xhtml"),
        elements: vec![ChapterElement::StandardParagraph(String::from(
            "Parágrafo de teste com conteúdo seguro.",
        ))],
        subtopics: Vec::new(),
        footnotes: Vec::new(),
    }];

    ExtractedBookPayload {
        metadata,
        chapters,
        figures: Vec::new(),
    }
}

#[test]
fn test_generated_epub_passes_w3c_validation() {
    let payload = sample_payload();
    let builder = NativeEpub3Builder::from_payload(payload);

    let temp_path =
        std::env::temp_dir().join(format!("test_valid_epub_{}.epub", uuid::Uuid::new_v4()));

    builder.save(&temp_path).expect("Falha ao salvar EPUB");

    let report = NativeEpubValidator::validate_file(&temp_path)
        .expect("Falha ao executar validação no arquivo EPUB");

    let _ = std::fs::remove_file(&temp_path);

    assert!(
        report.is_valid,
        "O EPUB gerado deve ser válido! Erros encontrados: {:?}",
        report.errors
    );
    assert!(report.errors.is_empty());
    assert_eq!(report.spine_items, 2); // sinopse + 1 capítulo
    assert!(report.total_files >= 5); // mimetype, container.xml, content.opf, nav.xhtml, style.css, chapter_001.xhtml
}

#[test]
fn test_corrupted_mimetype_fails_validation() {
    let temp_path =
        std::env::temp_dir().join(format!("test_corrupted_epub_{}.epub", uuid::Uuid::new_v4()));

    // Cria um arquivo ZIP inválido sem mimetype no offset 0
    let file = File::create(&temp_path).expect("Falha ao criar arquivo");
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("some_file.txt", options).unwrap();
    zip.write_all(b"dados aleatorios").unwrap();
    zip.finish().unwrap();

    let report = NativeEpubValidator::validate_file(&temp_path).expect("Falha ao validar arquivo");

    let _ = std::fs::remove_file(&temp_path);

    assert!(!report.is_valid);
    assert!(
        report.errors.iter().any(|e| e.contains("mimetype")),
        "Deveria reportar erro de mimetype ausente no offset 0"
    );
}
