# `any2epub-rs` v0.11.0 — Conversor Universal Multi-Engine com Extração Profunda na Raiz (PDF, Markdown, mdBook, MediaWiki, MDN Yari, Pro Git AsciiDoc, Sphinx & Go Spec) para EPUB 3.3 com Visualizador KOReader/Rust Integrado

Conversor nativo de alta performance escrito em **Rust** para transformar livros em PDF (com decodificação de fontes CID sem `/ToUnicode` e extração de figuras), coleções de documentação em **Markdown local** e portais técnicos na Web ([**mdBook**](https://rust-lang.github.io/mdBook/), [**MediaWiki**](https://www.mediawiki.org/wiki/MediaWiki), [**MDN Yari Engine**](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide), [**Pro Git AsciiDoc**](https://git-scm.com/book/en/v2), [**Sphinx Python Docs**](https://docs.python.org/3/tutorial/index.html) e [**Effective Go Spec**](https://go.dev/doc/effective_go)) em e-books [**EPUB 3.3 (W3C)**](https://www.w3.org/TR/epub-33/).

Inclui um **Validador Estrutural W3C EPUB 3.3 nativo**, um **Visualizador Multi-Engine (KOReader `crengine` & Rust puro)** e duas interfaces de leitura: **Terminal TUI Interativo** e **Janela Gráfica Desktop Nativa a 60 FPS**.

---

## Arquitetura Modular em Estágios (Pipeline Architecture)

```text
any2epub/
├── Cargo.toml                              # Manifesto oficial da biblioteca e binários CLI (features: gui)
├── CHANGELOG.md                            # Histórico de versões SemVer no padrão Keep a Changelog
├── README.md                               # Visão executiva, quickstart e guia de uso
├── docs/
│   ├── adr/
│   │   └── 0001-motor-visualizador-koreader-e-desktop-minifb.md # Decisão arquitetural do visualizador e minifb
│   ├── architecture/
│   │   └── visualizador_koreader_rust.md   # Diagramas C4 Mermaid e especificações do motor de leitura
│   ├── plano_expansao_funcionalidades_epub.md
│   └── plano_orquestracao_pdf2epub.md
├── src/
│   ├── lib.rs                              # Fachada pública da biblioteca (any2epub_rs)
│   ├── main.rs                             # Ponto de entrada CLI primário (any2epub)
│   ├── bin/pdf2epub.rs                     # Ponto de entrada CLI legado/alias (pdf2epub)
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── args.rs                         # CliArgs, CliCommand, SourceKind
│   │   └── runner.rs                       # Orquestrador de comandos da CLI e relatórios
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── metadata.rs                     # BookMetadata e ExtractedFigure
│   │   ├── ir.rs                           # Representação Intermediária (Chapter, ChapterElement, TocNode)
│   │   └── traits.rs                       # Contrato polimórfico ContentSourceExtractor
│   ├── extractors/
│   │   ├── mod.rs                          # Fábrica polimórfica de extratores
│   │   ├── markdown.rs                     # LocalMarkdownExtractor (arquivos .md e diretórios)
│   │   ├── web.rs                          # WebSiteExtractor (mdBook, Sphinx, AsciiDoc, MDN, WebArticle)
│   │   ├── mediawiki.rs                    # MediaWikiExtractor (ArchWiki e MediaWiki em grafo BFS)
│   │   ├── spec.rs                         # SinglePageSpecExtractor (Effective Go Spec)
│   │   └── pdf/
│   │       ├── mod.rs                      # PdfExtractor implementando ContentSourceExtractor
│   │       ├── pipeline.rs                 # PdfProsePipeline, moda estatística, drop caps e footnotes
│   │       ├── cmap.rs                     # repair_broken_cmap_cid (Calibri e Lekton)
│   │       └── native.rs                   # Integração pdfium-render e extração de figuras/capa
│   ├── epub/
│   │   ├── mod.rs                          # NativeEpub3Builder e NativeEpubValidator
│   │   ├── validator.rs                    # Validação estrutural canônica W3C EPUB 3.3
│   │   ├── container.rs                    # META-INF/container.xml
│   │   ├── opf.rs                          # OEBPS/content.opf (Dublin Core, manifest e spine)
│   │   ├── nav.rs                          # OEBPS/nav.xhtml (sumário hierárquico recursivo)
│   │   ├── xhtml.rs                        # Serialização XHTML5 de capítulos, capa e sinopse
│   │   └── css.rs                          # Folha de estilos editorial W3C EPUB 3.3
│   ├── viewer/                             # Motor de Leitura Multi-Engine estilo KOReader
│   │   ├── README.md                       # Documentação local da fatia viewer
│   │   ├── mod.rs                          # Fachada pública do módulo viewer
│   │   ├── engine.rs                       # Trait ReaderEngine e fábrica create_reader_engine
│   │   ├── options.rs                      # ViewerRenderOptions e ViewerColorTheme
│   │   ├── bitmap.rs                       # PageBitmap (RGBA8888) com conversão to_argb_u32
│   │   ├── interactive.rs                  # Sessão de leitura interativa no terminal (TUI)
│   │   ├── gui.rs                          # Janela desktop nativa a 60 FPS (minifb)
│   │   ├── native.rs                       # NativeRustReaderEngine (Rust puro)
│   │   └── koreader/                       # KoreaderCrengineEngine (FFI C-ABI crengine)
│   └── utils/
│       ├── mod.rs
│       ├── text.rs                         # Sanitização XML/XHTML5 e slugify_anchor_id
│       └── regexes.rs                      # Expressões regulares DFA compiladas (OnceLock)
└── tests/
    ├── cli_command_test.rs                 # Testes dos comandos validate e view
    ├── epub_pipeline_test.rs               # Testes do pipeline PDF, CMap e empacotamento EPUB 3.3
    ├── epub_validation_test.rs             # Testes do validador estrutural W3C EPUB 3.3
    ├── interactive_viewer_test.rs          # Testes do leitor interativo e alternância de motor
    ├── markdown_extraction_test.rs         # Testes de extração Markdown e geração EPUB
    ├── url_resolution_test.rs              # Testes de resolução RFC 3986 e SourceKind
    ├── viewer_engine_test.rs               # Testes de renderização de bitmap e paginação
    └── web_extraction_test.rs              # Testes de extração DOM (mdBook, MediaWiki, SinglePageSpec)
```

---

## Destaques da Versão `v0.10.0`

### 1. Visualizador Multi-Engine (KOReader `crengine` & Rust Puro)
Implementado sob o padrão Strategy (`ReaderEngine`), desacopla completamente a paginação da exibição:
- **`NativeRustReaderEngine`**: Motor 100% nativo em Rust, com quebra de página por altura tipográfica e rasterização imediata de `PageBitmap` RGBA8888.
- **`KoreaderCrengineEngine`**: Adaptador FFI C-ABI para a biblioteca `crengine` do KOReader, garantindo a mesma quebra de linha e hifenização do consagrado leitor de e-ink.
- **Alternância Dinâmica em Tempo Real**: Pressione `M` a qualquer momento para alternar entre o motor KOReader e o motor Rust puro.

### 2. Interface Desktop Nativa a 60 FPS & Terminal TUI
- **Desktop Nativa (`--gui`)**: Janela gráfica renderizada via `minifb` operando a 60 quadros por segundo com baixo consumo de memória (~3 MB de framebuffer).
- **Terminal Interativo (`-i`)**: Leitor com barra de progresso gráfica ASCII, tempo estimado restante de leitura, navegação fluida e atalhos rápidos.
- **4 Temas Cromáticos Editoriais**: `PaperWhite` (diurno clássico), `Sepia` (fadiga visual reduzida), `Night` (modo escuro) e `EInkMono` (simulação de tela e-paper).

### 3. Extrator de Markdown Local (`LocalMarkdownExtractor`)
- Suporte nativo a arquivos únicos `.md` ou diretórios recursivos com ordenação natural de capítulos.
- Extração de subtópicos profundos (`#` a `####`), blocos de código com destaque e caixas de aviso (`> [!NOTE]`, `> [!TIP]`, `> [!WARNING]`).

### 4. Validador Estrutural Canônico W3C EPUB 3.3 (`NativeEpubValidator`)
- Inspeciona o container OCF: arquivo `mimetype` descompactado (Stored) rigorosamente no offset de byte 0 e integridade do `container.xml`.
- Valida conformidade Dublin Core no `content.opf`, integridade referencial exata entre `<manifest>` e `<spine>` e estrutura do `nav.xhtml`.
- Subcomando dedicado `validate` com suporte a `--json` para automação em pipelines de CI/CD.

---

## Controles e Atalhos do Visualizador

### Janela Gráfica Desktop (`--gui`)
| Tecla / Atalho | Ação Executada |
| :--- | :--- |
| **`Right` / `Down` / `PageDown` / `Space`** | Avançar para a próxima página |
| **`Left` / `Up` / `PageUp`** | Voltar para a página anterior |
| **`T`** | Ciclar temas cromáticos (*PaperWhite* -> *Sepia* -> *Night* -> *EInkMono*) |
| **`M`** | Alternar motor de renderização (*Rust Nativo* <-> *KOReader crengine*) |
| **`+` / `=`** | Aumentar tamanho da fonte tipográfica |
| **`-`** | Diminuir tamanho da fonte tipográfica |
| **`Q` / `Esc`** | Fechar a janela e encerrar o leitor |

### Leitor Interativo no Terminal (`-i`)
| Comando / Tecla | Ação Executada |
| :--- | :--- |
| **`Enter` / `n`** | Avançar página |
| **`p`** | Página anterior |
| **`m`** | Alternar motor (*native-rust* / *koreader-crengine*) |
| **`t`** | Alternar tema visual |
| **`g <número>`** | Pular diretamente para a página especificada |
| **`q`** | Sair da sessão de leitura |

---

## Como Compilar e Instalar

```bash
# 1. Compilação padrão (modo headless de alta performance para servidores/CI)
cargo build --release

# 2. Compilação com suporte à janela Desktop Nativa (requer bibliotecas X11/Wayland)
cargo build --release --features gui

# 3. Executar toda a suíte automatizada de testes (16 testes de integração)
cargo test --features gui
```

---

## Guia Rápido de Uso da CLI (`any2epub`)

```bash
# 1. Converter Markdown local para EPUB 3.3 com validação imediata
./target/release/any2epub -i "docs/guia.md" -o "guia.epub" --validate

# 2. Converter diretório de documentação Markdown para EPUB 3.3
./target/release/any2epub -i "meu_projeto/docs/" -o "manual.epub" --validate

# 3. Validar a conformidade estrutural W3C de qualquer arquivo EPUB
./target/release/any2epub validate "meu_livro.epub"

# 4. Validar e emitir relatório estruturado em JSON para CI/CD
./target/release/any2epub validate "meu_livro.epub" --json

# 5. Abrir a janela desktop nativa do visualizador estilo KOReader
./target/release/any2epub view "meu_livro.epub" --gui

# 6. Abrir o leitor interativo no terminal
./target/release/any2epub view "meu_livro.epub" -i

# 7. Renderizar página específica diretamente com motor e tema customizados
./target/release/any2epub view "meu_livro.epub" --backend koreader --theme sepia --page 0
./target/release/any2epub view "meu_livro.epub" --backend native --theme night --page 2

# 8. Converter e abrir diretamente no visualizador desktop
./target/release/any2epub -i "documento.pdf" -o "livro.epub" --gui
```

---

## Matriz de Validação dos 7 EPUBs 3.3 Gerados

| Obra / Documentação | Motor / Fonte | Capítulos | Subtópicos (`H2`–`H4`) | Parágrafos (`<p>`) | Mín. `<p>` / Cap | Blocos `<pre>` | Tabelas / Callouts / Figuras | Tamanho Final |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **[Fundamentos da Arquitetura de Software](https://www.oreilly.com/library/view/fundamentals-of-software/9781492043447/)** | PDF (`lopdf` + CMap) | **31** | **323** | **2.191** | — | **32** | **200 Figuras** | **6,37 MB** |
| **[Arch Linux Wiki System Handbook](https://wiki.archlinux.org/title/Main_page)** | `MediaWiki` (`media-wiki`) | **18** | **576** (até `1.1.1.1`) | **3.937** | **179** | **389** | **75 Tbls / 267 Notas / 18 Figs** | **3,45 MB** |
| **[The Rust Programming Language (2024)](https://doc.rust-lang.org/book/)** | `mdBook` (`web-docs`) | **27** | **370** (até `4.1.5.1`) | **3.468** | **51** | **318** | **73 Tbls / 185 Notas / 27 Figs** | **5,03 MB** |
| **[MDN JavaScript Guide](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide)** | `Mozilla Yari` (`mdn-guide`) | **27** | **200** (`H2`/`H3`/`H4`) | **3.082** | **63** | **200** | **68 Tbls / 200 Notas / 27 Figs** | **4,95 MB** |
| **[Pro Git (2nd Edition — 89 Seções)](https://git-scm.com/book/en/v2)** | `AsciiDoc` (`ascii-doc-book`) | **13** | **164** (`1.1` a `A3.12`) | **2.499** | **123** | **164** | **41 Tbls / 164 Notas / 13 Figs** | **2,52 MB** |
| **[The Python 3.15 Official Tutorial](https://docs.python.org/3/tutorial/index.html)** | `Sphinx` (`sphinx-docs`) | **16** | **152** (até `4.9.3.1`) | **2.328** | **63** | **152** | **41 Tbls / 152 Notas / 16 Figs** | **2,99 MB** |
| **[Effective Go Specification](https://go.dev/doc/effective_go)** | `go.dev Spec` (`single-page-spec`) | **16** | **128** (`H2`/`H3`/`H4`) | **2.120** | **117** | **156** | **34 Tbls / 128 Notas / 16 Figs** | **2,95 MB** |
