//! Testes TDD do motor de visualização de EPUB (`any2epub_rs::viewer`).

use std::path::PathBuf;

use any2epub_rs::epub::NativeEpub3Builder;
use any2epub_rs::viewer::{
    create_reader_engine, ViewerBackendKind, ViewerColorTheme, ViewerRenderOptions,
};
use any2epub_rs::{BookMetadata, Chapter, ChapterElement, ExtractedBookPayload};

fn generate_test_epub() -> PathBuf {
    let metadata = BookMetadata {
        uuid: String::from("11111111-2222-3333-4444-555555555555"),
        isbn: String::from("urn:isbn:978-0-000000-0-0"),
        title: String::from("Livro de Teste do Visualizador"),
        authors: vec![String::from("Autor de Teste")],
        translator: String::new(),
        publisher: String::from("Editora Teste"),
        publication_date: String::from("2026-10-30"),
        synopsis: String::from("Sinopse para teste de visualização."),
        language: String::from("pt-BR"),
        cover_jpeg_bytes: None,
    };

    let chapters = vec![Chapter {
        index: 1,
        number_prefix: String::from("1"),
        title: String::from("Primeiro Capítulo"),
        filename: String::from("chapter_001.xhtml"),
        elements: vec![
            ChapterElement::SubheadingH2 {
                id: String::from("sec1"),
                number_prefix: String::from("1.1"),
                title: String::from("Seção Inicial"),
            },
            ChapterElement::StandardParagraph(String::from(
                "Este é o primeiro parágrafo de teste renderizado pelo motor de leitura.",
            )),
            ChapterElement::StandardParagraph(String::from(
                "Este é o segundo parágrafo de teste com mais conteúdo para validar paginação.",
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
    let temp_path =
        std::env::temp_dir().join(format!("test_viewer_book_{}.epub", uuid::Uuid::new_v4()));
    builder.save(&temp_path).expect("Falha ao salvar EPUB");
    temp_path
}

#[test]
fn test_native_rust_reader_engine_lifecycle() {
    let epub_path = generate_test_epub();

    let mut engine = create_reader_engine(ViewerBackendKind::NativeRust)
        .expect("Falha ao instanciar motor de leitura Nativo em Rust");

    assert_eq!(engine.engine_name(), "native-rust");

    let details = engine
        .open_book(&epub_path)
        .expect("Falha ao abrir livro EPUB no motor");

    assert_eq!(details.title, "Livro de Teste do Visualizador");
    assert_eq!(details.spine_documents_count, 2); // sinopse + chap_001

    // Configura viewport padrão e opções
    engine
        .set_viewport(800, 1200)
        .expect("Falha ao definir viewport");

    let options = ViewerRenderOptions {
        font_size_pt: 12.0,
        line_height_multiplier: 1.4,
        margin_px: 40,
        hyphenation_enabled: true,
        theme: ViewerColorTheme::Sepia,
    };
    engine
        .apply_render_options(&options)
        .expect("Falha ao aplicar opções");

    let total_pages = engine.total_pages();
    assert!(total_pages >= 1, "Deve paginar em pelo menos 1 página");

    // Renderiza a página 0
    let bitmap = engine
        .render_page(0)
        .expect("Falha ao renderizar primeira página");
    assert_eq!(bitmap.width, 800);
    assert_eq!(bitmap.height, 1200);
    assert_eq!(bitmap.rgba_pixels.len(), 800 * 1200 * 4);

    let argb = bitmap.to_argb_u32();
    assert_eq!(argb.len(), 800 * 1200);

    // Valida que o renderizador desenha glifos legíveis com padrões verticais de letras
    // (um traço contínuo puro teria proporção diferente de uma renderização com espaçamento entre letras/colunas)
    let text_color = ViewerColorTheme::Sepia.text_rgba();
    let drawn_text_pixels = bitmap
        .rgba_pixels
        .chunks_exact(4)
        .filter(|chunk| chunk == &text_color)
        .count();
    assert!(
        drawn_text_pixels > 50,
        "Deve conter pixels reais renderizados para o texto legível (encontrados: {})",
        drawn_text_pixels
    );

    // Valida que caracteres produzem altura de glifo e não apenas 1 linha horizontal
    // Inspeciona se há pixels de texto em múltiplos Ys dentro de uma mesma linha
    let margin_x = 40u32;
    let mut distinct_ys_with_text = 0;
    for y in margin_x..(margin_x + 30) {
        let has_text_in_row = (margin_x..(margin_x + 300)).any(|x| {
            let offset = ((y * 800 + x) as usize) * 4;
            bitmap.rgba_pixels[offset..offset + 4] == text_color
        });
        if has_text_in_row {
            distinct_ys_with_text += 1;
        }
    }
    assert!(
        distinct_ys_with_text >= 5,
        "Glifos tipográficos devem ter altura vertical de pelo menos 5 pixels na linha (encontrados: {})",
        distinct_ys_with_text
    );

    let _ = std::fs::remove_file(&epub_path);
}

#[test]
fn test_koreader_crengine_backend_descriptor() {
    let engine_res = create_reader_engine(ViewerBackendKind::KoreaderCrengine);
    assert!(
        engine_res.is_ok(),
        "Deve permitir instanciar descritor KOReader"
    );
    let engine = engine_res.unwrap();
    assert_eq!(engine.engine_name(), "koreader-crengine");
}
