# Changelog

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [0.9.0] - 2026-10-24

### Adicionado
- Arquitetura modular em estágios (`src/cli/`, `src/domain/`, `src/extractors/`, `src/epub/`, `src/utils/`) com separação entre biblioteca (`any2epub_rs`) e binário CLI (`pdf2epub`).
- Implementação de `PdfExtractor` aderente ao contrato polimórfico `ContentSourceExtractor` e fábrica `build_extractor`.
- Varredura recursiva BFS de sumários multinível (`WebSiteExtractor::crawl_docs_toc_bfs`) para mdBook, Sphinx, Pro Git AsciiDoc e MDN Yari.
- Extração de nós folha individuais (`<li>`, `<dt>`, `<dd>`, `<p>`, `<pre>`, `<table>`, `CalloutBox`) e prevenção de duplicação via `is_inside_callout_or_table`.
- Suíte de testes automatizados de integração e unidade em `tests/` (`web_extraction_test`, `url_resolution_test`, `epub_pipeline_test`).
