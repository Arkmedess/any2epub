//! Testes de extração profunda na raiz do DOM (`WebSiteExtractor`, `MediaWikiExtractor`, `SinglePageSpecExtractor`).

use scraper::Html;

use any2epub_rs::{
    BookMetadata, CalloutKind, ChapterElement, MediaWikiExtractor, SinglePageSpecExtractor,
    WebSiteExtractor,
};

fn dummy_meta() -> BookMetadata {
    BookMetadata {
        uuid: String::from("00000000-0000-4000-8000-000000000000"),
        isbn: String::from("urn:test:v0.9.0"),
        title: String::from("Test Book"),
        authors: vec![String::from("Test Author")],
        translator: String::from("Test"),
        publisher: String::from("Test"),
        publication_date: String::from("2026-10-24"),
        synopsis: String::from("Test"),
        language: String::from("en"),
        cover_jpeg_bytes: None,
    }
}

#[test]
fn test_leaf_dom_extraction_preserves_li_dt_dd_and_callouts() {
    let html = r#"
    <html><body>
      <article class="main-page-content">
        <h1>Control flow and error handling</h1>
        <p>JavaScript supports a compact set of control flow statements.</p>
        <h2 id="conditional-statements">Conditional statements</h2>
        <p>A conditional statement executes if a specified condition is true.</p>
        <ul>
          <li>false literal</li>
          <li>undefined primitive</li>
          <li>null reference</li>
          <li>0 and -0 numeric values</li>
          <li>NaN (Not-a-Number)</li>
          <li>empty string ("")</li>
        </ul>
        <dl>
          <dt>unhandledrejection</dt>
          <dd>Sent when a promise is rejected but there is no rejection handler available.</dd>
          <dt>rejectionhandled</dt>
          <dd>Sent when a handler is attached to a rejected promise.</dd>
        </dl>
        <div class="notecard note">
          <p><strong>Note:</strong> Do not confuse primitive boolean values with Boolean objects.</p>
        </div>
        <pre><code>js Copy
if (condition) {
  statement1;
}</code></pre>
      </article>
    </body></html>
    "#;
    let doc = Html::parse_document(html);
    let extractor = WebSiteExtractor {
        start_url: String::from(
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Control_flow_and_error_handling",
        ),
        crawl_docs_toc: true,
        toc_selector: String::new(),
        default_metadata: dummy_meta(),
    };
    let (title, elements, subtopics) =
        extractor.extract_clean_article_elements(&doc, 3, "3", "chapter_003.xhtml");
    assert_eq!(title, "3. Control flow and error handling");
    assert_eq!(subtopics.len(), 1);
    assert_eq!(subtopics[0].number_prefix, "3.1");
    assert_eq!(subtopics[0].title, "Conditional statements");
    assert_eq!(
        subtopics[0].href,
        "chapter_003.xhtml#conditional-statements"
    );
    // 2 <p> + 1 <h2> + 6 <li> + 2 <dt> + 2 <dd> + 1 CalloutBox + 1 CodeBlock = 15 elementos!
    assert_eq!(elements.len(), 15);
}

#[test]
fn test_mediawiki_article_parsing_and_hierarchy() {
    let html = r#"
    <html><body>
      <h1 id="firstHeading">Systemd</h1>
      <div id="mw-content-text">
        <div class="mw-parser-output">
          <div class="archwiki-template-box archwiki-template-box-tip">Use systemctl to manage units.</div>
          <p>systemd is a suite of basic building blocks for a Linux system.</p>
          <h2><span class="mw-headline" id="Basic_systemctl_usage">Basic systemctl usage</span></h2>
          <p>The main command used to introspect and control systemd is systemctl.</p>
          <h3><span class="mw-headline" id="Analyzing_the_system_state">Analyzing the system state</span></h3>
          <h4><span class="mw-headline" id="Unit_status">Unit status</span></h4>
          <pre># systemctl status <em>unit</em></pre>
          <table class="wikitable">
            <tr><th>Action</th><th>Command</th></tr>
            <tr><td>Start unit</td><td>systemctl start</td></tr>
          </table>
          <a href="/title/Arch_boot_process">Arch boot process</a>
          <a href="/title/Special:RecentChanges">Ignored special page</a>
        </div>
      </div>
    </body></html>
    "#;
    let doc = Html::parse_document(html);
    let (title, elements, subtopics, links) = MediaWikiExtractor::parse_mediawiki_article(
        "https://wiki.archlinux.org",
        &doc,
        1,
        "chapter_001.xhtml",
    );
    assert_eq!(title, "Systemd");
    assert_eq!(
        links,
        vec![String::from(
            "https://wiki.archlinux.org/title/Arch_boot_process"
        )]
    );
    assert_eq!(subtopics.len(), 1);
    assert_eq!(subtopics[0].number_prefix, "1.1");
    assert_eq!(subtopics[0].children.len(), 1);
    assert_eq!(subtopics[0].children[0].number_prefix, "1.1.1");
    assert_eq!(subtopics[0].children[0].children.len(), 1);
    assert_eq!(
        subtopics[0].children[0].children[0].number_prefix,
        "1.1.1.1"
    );
    assert!(matches!(
        elements[0],
        ChapterElement::CalloutBox {
            kind: CalloutKind::Tip,
            ..
        }
    ));
}

#[test]
fn test_single_page_spec_segmentation() {
    let html = r#"
    <html><body>
      <article class="Doc">
        <h2 id="introduction">Introduction</h2>
        <p>Go is a new language.</p>
        <h3 id="examples">Examples</h3>
        <p>The Go package sources are intended to serve not only as the core library.</p>
        <h2 id="formatting">Formatting</h2>
        <p>Formatting issues are the most contentious.</p>
        <pre>gofmt -w src.go</pre>
      </article>
    </body></html>
    "#;
    let doc = Html::parse_document(html);
    let extractor = SinglePageSpecExtractor {
        url: String::from("https://go.dev/doc/effective_go"),
        default_metadata: dummy_meta(),
    };
    let chapters = extractor.parse_spec_document(&doc);
    assert_eq!(chapters.len(), 2);
    assert_eq!(chapters[0].title, "1. Introduction");
    assert_eq!(chapters[0].subtopics.len(), 1);
    assert_eq!(chapters[0].subtopics[0].number_prefix, "1.1");
    assert_eq!(chapters[1].title, "2. Formatting");
}
