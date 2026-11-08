//! Testes TDD do extrator de arquivos Markdown locais (`LocalMarkdownExtractor`).

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use any2epub_rs::epub::validator::NativeEpubValidator;
use any2epub_rs::epub::NativeEpub3Builder;
use any2epub_rs::extractors::LocalMarkdownExtractor;
use any2epub_rs::{BookMetadata, CalloutKind, ChapterElement, ContentSourceExtractor, SourceKind};

fn dummy_meta() -> BookMetadata {
    BookMetadata {
        uuid: String::from("44444444-5555-6666-7777-888888888888"),
        isbn: String::from("urn:isbn:978-3-16-148410-0"),
        title: String::from("Guia de Arquitetura em Rust"),
        authors: vec![String::from("Engenheiro Sênior")],
        translator: String::new(),
        publisher: String::from("Editora Técnica"),
        publication_date: String::from("2026-10-30"),
        synopsis: String::from("Guia prático de engenharia de software."),
        language: String::from("pt-BR"),
        cover_jpeg_bytes: None,
    }
}

fn create_sample_markdown_file() -> PathBuf {
    let temp_path = std::env::temp_dir().join(format!("test_chapter_{}.md", uuid::Uuid::new_v4()));
    let mut file = File::create(&temp_path).expect("Falha ao criar arquivo markdown de teste");

    let content = r#"# Capítulo 1: Arquitetura e Princípios SOLID

Este é o primeiro parágrafo introduzindo o capítulo de arquitetura.

## 1.1 Inversão de Dependência (DIP)

O domínio de negócio deve depender exclusivamente de portas e abstrações, nunca de bibliotecas concretas de I/O.

> [!NOTE]
> Mantenha sempre contratos enxutos e orientados estritamente ao consumidor (*ISP*).

```rust
pub trait ContentSourceExtractor {
    fn extract_payload(&self) -> Result<ExtractedBookPayload, Box<dyn std::error::Error>>;
}
```

- Simplicidade primeiro (YAGNI estrito)
- Cirurgia de diff em todas as entregas
- Zero dependências desnecessárias
"#;

    file.write_all(content.as_bytes())
        .expect("Falha ao escrever markdown");
    temp_path
}

#[test]
fn test_source_kind_auto_detects_markdown() {
    assert_eq!(
        SourceKind::Auto.resolve_effective_kind("docs/architecture.md"),
        SourceKind::Markdown
    );
    assert_eq!(
        SourceKind::Auto.resolve_effective_kind("/home/user/notes/book.markdown"),
        SourceKind::Markdown
    );
}

#[test]
fn test_local_markdown_extractor_extracts_elements_and_generates_valid_epub() {
    let md_path = create_sample_markdown_file();

    let extractor = LocalMarkdownExtractor {
        input_path: md_path.clone(),
        default_metadata: dummy_meta(),
    };

    let payload = extractor
        .extract_payload()
        .expect("Falha ao extrair payload do arquivo markdown");

    assert_eq!(payload.chapters.len(), 1);
    let chapter = &payload.chapters[0];
    assert_eq!(chapter.title, "Capítulo 1: Arquitetura e Princípios SOLID");
    assert_eq!(chapter.subtopics.len(), 1);
    assert_eq!(chapter.subtopics[0].number_prefix, "1.1");
    assert_eq!(chapter.subtopics[0].title, "Inversão de Dependência (DIP)");

    // Valida elementos extraídos
    let has_callout = chapter.elements.iter().any(|el| {
        matches!(
            el,
            ChapterElement::CalloutBox {
                kind: CalloutKind::Note,
                ..
            }
        )
    });
    assert!(has_callout, "Deveria extrair a caixa de destaque [!NOTE]");

    let has_code = chapter
        .elements
        .iter()
        .any(|el| matches!(el, ChapterElement::CodeBlock(_)));
    assert!(has_code, "Deveria extrair o bloco de código Rust");

    // Empacota e valida com o NativeEpubValidator
    let builder = NativeEpub3Builder::from_payload(payload);
    let epub_path = std::env::temp_dir().join(format!("test_md_{}.epub", uuid::Uuid::new_v4()));
    builder.save(&epub_path).expect("Falha ao salvar EPUB");

    let report = NativeEpubValidator::validate_file(&epub_path)
        .expect("Falha ao validar EPUB gerado a partir de Markdown");

    assert!(
        report.is_valid,
        "EPUB gerado a partir de Markdown deve ser 100% válido: {:?}",
        report.errors
    );

    let _ = std::fs::remove_file(&md_path);
    let _ = std::fs::remove_file(&epub_path);
}
