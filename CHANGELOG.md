# Changelog

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [0.10.0] - 2026-10-30

### Adicionado
- Binário principal `any2epub` no manifesto `Cargo.toml`, mantendo `pdf2epub` como alias retrocompatível via `src/bin/pdf2epub.rs`.
- Validador nativo de conformidade estrutural W3C EPUB 3.3 (`NativeEpubValidator`) inspecionando OCF (mimetype descompactado no byte 0, container.xml), OPF Dublin Core e integridade referencial do manifest/spine.
- Subcomando `any2epub validate <path> [--json]` e flag `--validate` para auditoria automática pós-geração.
- Módulo de visualização e motor de paginação (`src/viewer/`) com padrão Strategy (`ReaderEngine`):
  - `NativeRustReaderEngine`: layout de parágrafos, quebra de página por altura de linha e rasterização de bitmap RGBA.
  - `KoreaderCrengineEngine`: adaptador para o motor oficial `crengine` do KOReader via FFI com suporte a bridge.
  - Temas cromáticos editoriais: `PaperWhite`, `Sepia`, `Night` e `EInkMono`.
- Subcomando `any2epub view <path> [--backend <koreader|native>] [--theme <sepia|white|night|eink>]` e flag `--view`.
- Extrator de documentos Markdown locais (`LocalMarkdownExtractor`) com suporte a arquivos `.md` e diretórios de documentação, extração de subtópicos (`#` a `####`), blocos de código com highlight e callouts (`> [!NOTE]`, `> [!TIP]`, `> [!WARNING]`).
- Sessão de leitura interativa estilo KOReader no terminal (`run_interactive_viewer`) acionada via `any2epub view <livro.epub> -i`, com barra de progresso gráfica, tempo restante estimado, alternância de motor (`native-rust` / `koreader-crengine`) e ciclos de tema em tempo real.
- Suíte expandida para 16 testes automatizados de integração em `tests/` cobrindo o pipeline completo de ponta a ponta.

## [0.9.0] - 2026-10-24

### Adicionado
- Arquitetura modular em estágios (`src/cli/`, `src/domain/`, `src/extractors/`, `src/epub/`, `src/utils/`) com separação entre biblioteca (`any2epub_rs`) e binário CLI (`pdf2epub`).
- Implementação de `PdfExtractor` aderente ao contrato polimórfico `ContentSourceExtractor` e fábrica `build_extractor`.
- Varredura recursiva BFS de sumários multinível (`WebSiteExtractor::crawl_docs_toc_bfs`) para mdBook, Sphinx, Pro Git AsciiDoc e MDN Yari.
- Extração de nós folha individuais (`<li>`, `<dt>`, `<dd>`, `<p>`, `<pre>`, `<table>`, `CalloutBox`) e prevenção de duplicação via `is_inside_callout_or_table`.
- Suíte de testes automatizados de integração e unidade em `tests/` (`web_extraction_test`, `url_resolution_test`, `epub_pipeline_test`).
