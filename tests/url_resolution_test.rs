//! Testes de resolução de URLs, escopo canônico e detecção de motores (`SourceKind`).

use any2epub_rs::{SourceKind, WebSiteExtractor};

#[test]
fn test_progit_mdn_sphinx_url_resolution_and_scope() {
    let mdn_base = WebSiteExtractor::normalize_base_url(
        "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide",
    );
    assert_eq!(
        WebSiteExtractor::resolve_chapter_url(
            &mdn_base,
            "/en-US/docs/Web/JavaScript/Guide/Using_promises#chaining"
        ),
        Some(String::from(
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Using_promises"
        ))
    );
    let progit_base = WebSiteExtractor::normalize_base_url("https://git-scm.com/book/en/v2");
    assert_eq!(
        WebSiteExtractor::resolve_chapter_url(
            &progit_base,
            "/book/en/v2/Git-Internals-Git-Objects"
        ),
        Some(String::from(
            "https://git-scm.com/book/en/v2/Git-Internals-Git-Objects"
        ))
    );
}

#[test]
fn test_source_kind_auto_detection() {
    assert_eq!(
        SourceKind::Auto.resolve_effective_kind("https://wiki.archlinux.org/title/Main_page"),
        SourceKind::MediaWiki
    );
    assert_eq!(
        SourceKind::Auto.resolve_effective_kind("https://docs.python.org/3/tutorial/index.html"),
        SourceKind::SphinxDocs
    );
    assert_eq!(
        SourceKind::Auto.resolve_effective_kind("https://git-scm.com/book/en/v2"),
        SourceKind::AsciiDocBook
    );
    assert_eq!(
        SourceKind::Auto.resolve_effective_kind("https://go.dev/doc/effective_go"),
        SourceKind::SinglePageSpec
    );
    assert_eq!(
        SourceKind::Auto.resolve_effective_kind(
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide"
        ),
        SourceKind::MdnGuide
    );
    assert_eq!(
        SourceKind::Auto.resolve_effective_kind("https://doc.rust-lang.org/book/"),
        SourceKind::WebDocs
    );
    assert_eq!(
        SourceKind::Auto.resolve_effective_kind("livro_local.pdf"),
        SourceKind::Pdf
    );
}
