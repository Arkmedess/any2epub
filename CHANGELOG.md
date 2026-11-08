# Changelog

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [Unreleased]

## [0.11.0] - 2026-11-07

### Adicionado
- Rasterizador nativo de glifos tipográficos (`src/viewer/font.rs`) com matriz monocromática 5x7 e renderização de caracteres legíveis e escaláveis com reforço negrito para títulos no buffer RGBA de `NativeRustReaderEngine`.
- Pipelines automatizados de CI/CD via GitHub Actions:
  - `.github/workflows/ci.yml`: verificação contínua de formatação (`cargo fmt`), lint (`cargo clippy --all-targets --features gui`) e suíte de testes completa.
  - `.github/workflows/release.yml`: matriz de compilação multiplataforma (Linux x86_64, macOS Apple Silicon e Windows x86_64) com cálculo determinístico de `SHA256SUMS` e publicação via GitHub CLI.
- Manifesto `Dockerfile` multi-stage rootless para execução e conversão headless em contêineres e servidores com `libpdfium-dev`.
- Metadados formais de publicação e empacotamento no manifesto `Cargo.toml` (`repository`, `homepage`, `keywords`, `categories`, `exclude`).

### Modificado
- Auto-inferência inteligente de metadados na CLI (`src/cli/args.rs`): remoção dos valores padrão hardcoded do *Rust Book*; inferência do título a partir do arquivo de entrada e geração automática de UUID para identificador `urn:uuid:<uuid>`.
- Separação estrita de streams POSIX (`stdout` vs. `stderr`): logs de progresso e status direcionados a `stderr`; `stdout` reservado exclusivamente para dados e JSON de validação.
- Generalização algorítmica de heurísticas de PDF em `src/extractors/pdf/`:
  - `native.rs`: remoção da trava arbitrária `page_idx >= 19`, permitindo extração de figuras ilustrativas em qualquer página do documento; detecção ampliada de fontes monoespaçadas (`mono`, `courier`, `consolas`, `code`, `lekton`, `menlo`).
  - `pipeline.rs`: detecção universal de famílias sem serifa/display (`sans`, `helvetica`, `arial`, `display`, `myriad`) para linhas de títulos de capítulos e seções.

### Corrigido
- Formatação e integridade de streams de saída para integração com ferramentas de automação e piping (`jq`).

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
