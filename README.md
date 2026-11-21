# any2epub

Conversor em **Rust** para transformar livros PDF, diretórios Markdown e documentações técnicas da Web em arquivos **EPUB 3.3** validados contra a especificação W3C. O projeto separa o núcleo **CLI** (conversão headless, automação em CI e validação estrutural) da interface **GUI** (leitor desktop opcional).

[Showcase & Simulador](https://arkmedess.github.io/any2epub/) · [Releases](https://github.com/Arkmedess/any2epub/releases) · [Roadmap](ROADMAP.md) · [Changelog](CHANGELOG.md)

---

## Separação CLI vs. GUI

| Perfil de Uso | Binário / Feature | Escopo Principal | Dependências de Sistema |
| :--- | :--- | :--- | :--- |
| **CLI (Padrão / Headless)** | `any2epub` (ou alias `pdf2epub`) sem features extras | Conversão em lote, pipeline CI/CD, validação W3C (`--json`), leitura rápida via TUI (`-i`) | Nenhuma dependência gráfica |
| **GUI (Desktop Nativo)** | Compilado com `--features gui` | Janela gráfica `minifb` a 60 FPS para leitura visual de EPUBs (`--gui`) | Bibliotecas X11/Wayland no Linux (`libx11-dev`, `libxcursor-dev`, `libxrandr-dev`, `libxi-dev`) |

---

## Fontes Suportadas (`--source-kind`)

Com `--source-kind auto` (padrão), o motor infere a fonte pelo caminho local ou padrão de URL. Para forçar um extrator específico, passe uma das opções abaixo:

| Valor `--source-kind` | Fonte Alvo | Características Extraídas |
| :--- | :--- | :--- |
| `auto` | Detecção automática | `.md`/`.markdown` -> `markdown`; outros locais -> `pdf`; URLs mapeadas por padrão |
| `pdf` | Livros e artigos em PDF | Reparo de CMap CID (`Calibri`, `Lekton`), figuras/capa (`pdfium-render`), capitulares e notas de rodapé |
| `markdown` | Arquivo `.md` ou diretório recursivo | Ordenação natural, cabeçalhos `#`–`####`, blocos de código e callouts (`[!NOTE]`, `[!TIP]`, `[!WARNING]`) |
| `web-docs` | Documentação linear (`mdBook` e similares) | Varredura BFS de sumário multinível, tabelas, blocos `<pre>` e diagramas |
| `media-wiki` | Portais em grafo (`MediaWiki` / `ArchWiki`) | Travessia BFS configurável (`--max-depth`, `--max-chapters`) e tabelas `.wikitable` |
| `mdn-guide` | Guias MDN (`Mozilla Yari`) | Navegação de barra lateral, listas de definição (`<dl>`/`<dt>`/`<dd>`) e callouts |
| `ascii-doc-book` | Livros AsciiDoc (`Pro Git`) | Capítulos e seções `ol.book-toc`, blocos `admonitionblock` e exemplos |
| `sphinx-docs` | Documentação `Sphinx` / ReadTheDocs | Árvores `toctree-l1`/`toctree-l2` e assinaturas de API |
| `single-page-spec` | Especificações em página única (`Effective Go`) | Segmentação por `<h2>` em capítulos navegáveis |
| `web-article` | Artigos Web avulsos | Extração limpa de artigo único |

---

## Instalação

### 1. Binários Pré-Compilados (CLI Headless)
Baixe o pacote correspondente nas [GitHub Releases](https://github.com/Arkmedess/any2epub/releases) (`x86_64-unknown-linux-gnu`, `aarch64-apple-darwin` ou `x86_64-pc-windows-msvc`) e verifique a integridade com `SHA256SUMS`. Os binários de release usam o perfil headless padrão (sem GUI).

### 2. Compilação via Cargo

```bash
# CLI headless (recomendado para servidores, automação e uso em terminal)
cargo build --release

# CLI + Leitor Desktop GUI (requer pacotes de desenvolvimento X11/Wayland no Linux)
cargo build --release --features gui
```

No Ubuntu/Debian, instale as dependências de janela antes de compilar com `--features gui`:

```bash
sudo apt-get install -y libx11-dev libxcursor-dev libxrandr-dev libxi-dev
```

### 3. Container Docker (CLI Headless Rootless)

```bash
docker build -t any2epub:local .
docker run --rm -v "$PWD:/data" any2epub:local -i "/data/livro.pdf" -o "/data/livro.epub" --validate
```

---

## Uso da CLI (`any2epub`)

A CLI segue separação POSIX de streams: relatórios finais e JSON (`--json`) vão para `stdout`; logs de progresso e diagnósticos vão para `stderr` (suprimíveis com `-q`/`--quiet`).

### Conversão de Documentos

```bash
# PDF local para EPUB 3.3 com validação estrutural ao final
any2epub -i "livro.pdf" -o "livro.epub" --validate

# Diretório de Markdown local
any2epub -i "docs/" -o "manual.epub" --validate

# Documentação Web com metadados explícitos
any2epub \
  -i "https://doc.rust-lang.org/book/" \
  -o "rust-book.epub" \
  --title "The Rust Programming Language" \
  --lang "en" \
  --validate

# Portal MediaWiki limitando profundidade e capítulos do grafo
any2epub \
  -i "https://wiki.archlinux.org/title/Main_page" \
  -o "archwiki.epub" \
  --source-kind media-wiki \
  --max-depth 2 \
  --max-chapters 18
```

### Validação W3C EPUB 3.3

Verifica `mimetype` não comprimido (`Stored`) no byte offset `0`, `META-INF/container.xml`, metadados Dublin Core em `content.opf`, integridade entre `<manifest>` e `<spine>` e estrutura do `nav.xhtml`.

```bash
# Saída legível no terminal
any2epub validate "livro.epub"

# Saída estruturada em JSON para pipelines de CI/CD
any2epub validate "livro.epub" --json
```

### Leitura via Terminal (TUI)

```bash
# Sessão interativa no terminal
any2epub view "livro.epub" -i

# Renderizar uma página específica diretamente no terminal
any2epub view "livro.epub" --theme sepia --page 0
```

Atalhos no modo TUI (`-i`): `Enter`/`n` (próxima página), `p` (anterior), `g <número>` (ir para página), `t` (ciclar tema), `m` (alternar motor), `q` (sair).

---

## Uso da GUI (Leitor Desktop — `--features gui`)

Quando compilado com `--features gui`, o subcomando `view --gui` (ou a flag `-g`/`--gui` após uma conversão) abre uma janela desktop nativa via `minifb` com 4 temas (`paper-white`, `sepia`, `night`, `e-ink-mono`).

```bash
# Abrir EPUB existente na janela gráfica
any2epub view "livro.epub" --gui --theme sepia

# Converter PDF e abrir imediatamente na GUI
any2epub -i "documento.pdf" -o "livro.epub" --gui
```

Atalhos na janela GUI:

| Tecla | Ação |
| :--- | :--- |
| `Right` / `Down` / `PageDown` / `Space` | Próxima página |
| `Left` / `Up` / `PageUp` | Página anterior |
| `T` | Ciclar temas (`PaperWhite` -> `Sepia` -> `Night` -> `EInkMono`) |
| `M` | Alternar backend (`native` <-> `koreader`) |
| `+` / `=` e `-` | Aumentar / diminuir tamanho da fonte |
| `Q` / `Esc` | Fechar janela |

> **Nota sobre o backend `koreader`:** `KoreaderCrengineEngine` inspeciona e carrega `libcrengine` dinamicamente via FFI quando disponível no sistema para abertura, redimensionamento e contagem de páginas, mas a rasterização de `render_page` ainda delega para `NativeRustReaderEngine` (fonte bitmap 5x7 em buffer RGBA). A implementação de renderização nativa direta do `crengine` e suporte a fontes TrueType/OTF estão planejadas no [ROADMAP.md](ROADMAP.md).

---

## Arquitetura Modular

O projeto organiza o fluxo em estágios desacoplados:

| Módulo | Responsabilidade | Documentação |
| :--- | :--- | :--- |
| `src/cli/` | Parsing de argumentos (`clap`), inferência de metadados e orquestração de subcomandos | [src/cli/README.md](src/cli/README.md) |
| `src/domain/` | Representação intermediária (`Chapter`, `ChapterElement`, `TocNode`) e trait `ContentSourceExtractor` | [src/domain/README.md](src/domain/README.md) |
| `src/extractors/` | Extratores PDF (`lopdf`/`pdfium-render`), Markdown local, Web BFS, MediaWiki e Spec | [src/extractors/README.md](src/extractors/README.md) |
| `src/epub/` | Empacotador OCF/EPUB 3.3 (`NativeEpub3Builder`) e validador (`NativeEpubValidator`) | [src/epub/README.md](src/epub/README.md) |
| `src/viewer/` | Trait `ReaderEngine`, rasterizador nativo, ponte FFI `crengine`, TUI e janela `minifb` | [src/viewer/README.md](src/viewer/README.md) |
| `docs/` | ADRs e diagramas C4 Mermaid do pipeline e visualizador | [ADR-0001](docs/adr/0001-motor-visualizador-koreader-e-desktop-minifb.md) · [Arquitetura Viewer](docs/architecture/visualizador_koreader_rust.md) |

---

## Referência de Extrações Validadas (EPUB 3.3)

| Obra / Documentação | Motor / Fonte | Capítulos | Subtópicos (`H2`–`H4`) | Parágrafos (`<p>`) | Blocos `<pre>` | Tabelas / Callouts / Figuras | Tamanho |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| [Fundamentals of Software Architecture](https://www.oreilly.com/library/view/fundamentals-of-software/9781492043447/) | `pdf` (`lopdf` + CMap) | 31 | 323 | 2.191 | 32 | 200 figuras | 6,37 MB |
| [Arch Linux Wiki](https://wiki.archlinux.org/title/Main_page) | `media-wiki` | 18 | 576 | 3.937 | 389 | 75 tbls / 267 notas / 18 figs | 3,45 MB |
| [The Rust Programming Language](https://doc.rust-lang.org/book/) | `web-docs` (`mdBook`) | 27 | 370 | 3.468 | 318 | 73 tbls / 185 notas / 27 figs | 5,03 MB |
| [MDN JavaScript Guide](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide) | `mdn-guide` (`Yari`) | 27 | 200 | 3.082 | 200 | 68 tbls / 200 notas / 27 figs | 4,95 MB |
| [Pro Git (2nd Edition)](https://git-scm.com/book/en/v2) | `ascii-doc-book` | 13 | 164 | 2.499 | 164 | 41 tbls / 164 notas / 13 figs | 2,52 MB |
| [Python Tutorial](https://docs.python.org/3/tutorial/index.html) | `sphinx-docs` | 16 | 152 | 2.328 | 152 | 41 tbls / 152 notas / 16 figs | 2,99 MB |
| [Effective Go](https://go.dev/doc/effective_go) | `single-page-spec` | 16 | 128 | 2.120 | 156 | 34 tbls / 128 notas / 16 figs | 2,95 MB |

---

## Desenvolvimento e Verificação

Comandos exigidos pelo CI ([.github/workflows/ci.yml](.github/workflows/ci.yml)):

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --features gui -- -D warnings
cargo test --features gui
```

Licença: `MIT OR Apache-2.0`.
