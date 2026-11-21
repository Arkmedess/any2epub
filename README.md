# any2epub

Conversor em **Rust** para transformar livros PDF, pastas Markdown e documentações técnicas da Web em arquivos **EPUB 3.3** válidos, prontos para leitura no **Kindle desbloqueado (KOReader)** e em outros e-readers. O projeto separa a ferramenta de linha de comando (**CLI**) do leitor gráfico desktop (**GUI**).

[Showcase & Simulador](https://arkmedess.github.io/any2epub/) · [Releases](https://github.com/Arkmedess/any2epub/releases) · [Roadmap](ROADMAP.md) · [Changelog](CHANGELOG.md)

---

## Separação CLI vs. GUI

| Perfil de Uso | Como usar / compilar | Para que serve | Dependências de Sistema |
| :--- | :--- | :--- | :--- |
| **CLI (Padrão / Terminal)** | `any2epub` (ou `pdf2epub`) sem features extras | Conversão de livros e sites, inspeção e validação EPUB (`--json`), leitura simples no terminal (`-i`) | Nenhuma dependência gráfica |
| **GUI (Leitor Desktop)** | Compilado com `--features gui` | Janela desktop a 60 FPS para visualizar e testar a leitura do EPUB antes de enviar ao e-reader (`--gui`) | Bibliotecas X11/Wayland no Linux (`libx11-dev`, `libxcursor-dev`, `libxrandr-dev`, `libxi-dev`) |

---

## Fontes Suportadas (`--source-kind`)

No modo padrão (`--source-kind auto`), o programa identifica a origem pelo arquivo ou endereço Web. Para escolher o leitor de origem manualmente, use uma das opções abaixo:

| Valor `--source-kind` | Origem | O que extrai |
| :--- | :--- | :--- |
| `auto` | Detecção automática | `.md`/`.markdown` -> `markdown`; outros arquivos locais -> `pdf`; URLs identificadas pelo domínio/estrutura |
| `pdf` | Livros e artigos em PDF | Texto corrido, reparo de fontes PDF sem tabela de caracteres (`Calibri`, `Lekton`), imagens/capa, capitulares e notas de rodapé |
| `markdown` | Arquivo `.md` ou pasta com vários capítulos | Ordem natural de arquivos, títulos `#`–`####`, blocos de código e caixas de aviso (`[!NOTE]`, `[!TIP]`, `[!WARNING]`) |
| `web-docs` | Documentação online (`mdBook` e similares) | Leitura completa do índice lateral em múltiplos níveis, tabelas, blocos de código e diagramas |
| `media-wiki` | Wikis (`MediaWiki` / `ArchWiki`) | Coleta automática de páginas ligadas (`--max-depth`, `--max-chapters`) e tabelas `.wikitable` |
| `mdn-guide` | Guias MDN (`Mozilla Yari`) | Menu lateral completo, listas de definição, exemplos e notas |
| `ascii-doc-book` | Livros AsciiDoc (`Pro Git`) | Capítulos e subseções completos, blocos de aviso e código |
| `sphinx-docs` | Documentação `Sphinx` / ReadTheDocs | Índice em árvore (`toctree`) e referências de API |
| `single-page-spec` | Documentação em página única (`Effective Go`) | Divisão automática por seções `<h2>` em capítulos |
| `web-article` | Artigos Web avulsos | Texto principal limpo de uma única página |

---

## Instalação

### 1. Binários Prontos (CLI)
Baixe o pacote para seu sistema nas [GitHub Releases](https://github.com/Arkmedess/any2epub/releases) (`x86_64-unknown-linux-gnu`, `aarch64-apple-darwin` ou `x86_64-pc-windows-msvc`) e confira a assinatura no arquivo `SHA256SUMS`. Os binários de release vêm no perfil padrão de terminal (sem interface gráfica).

### 2. Compilação via Cargo

```bash
# CLI pura (para terminal, scripts, servidores e automação)
cargo build --release

# CLI + Leitor Desktop GUI (exige pacotes de janela X11/Wayland no Linux)
cargo build --release --features gui
```

No Ubuntu/Debian, instale os pacotes de janela antes de compilar com `--features gui`:

```bash
sudo apt-get install -y libx11-dev libxcursor-dev libxrandr-dev libxi-dev
```

### 3. Container Docker (CLI sem usuário root)

```bash
docker build -t any2epub:local .
docker run --rm -v "$PWD:/data" any2epub:local -i "/data/livro.pdf" -o "/data/livro.epub" --validate
```

---

## Uso da CLI (`any2epub`)

A saída da CLI separa dados de logs: resultados e JSON (`--json`) vão para `stdout`, enquanto avisos e progresso vão para `stderr` (ocultáveis com `-q`/`--quiet`).

### Conversão para EPUB 3.3

```bash
# PDF local com checagem final do EPUB gerado
any2epub -i "livro.pdf" -o "livro.epub" --validate

# Pasta com capítulos em Markdown
any2epub -i "docs/" -o "manual.epub" --validate

# Documentação Web com título e idioma definidos
any2epub \
  -i "https://doc.rust-lang.org/book/" \
  -o "rust-book.epub" \
  --title "The Rust Programming Language" \
  --lang "en" \
  --validate

# Wiki limitando profundidade e número de páginas coletadas
any2epub \
  -i "https://wiki.archlinux.org/title/Main_page" \
  -o "archwiki.epub" \
  --source-kind media-wiki \
  --max-depth 2 \
  --max-chapters 18
```

### Validação EPUB 3.3

Confere se o arquivo `mimetype` está sem compressão no início do ZIP, se `META-INF/container.xml` e `nav.xhtml` estão válidos e se todos os itens do livro estão ligados corretamente entre `manifest` e `spine` no `content.opf`.

```bash
# Relatório legível no terminal
any2epub validate "livro.epub"

# Saída em JSON para scripts e CI
any2epub validate "livro.epub" --json
```

### Leitura no Terminal (TUI)

```bash
# Modo interativo direto no terminal
any2epub view "livro.epub" -i

# Renderizar uma página específica e exibir informações
any2epub view "livro.epub" --theme sepia --page 0
```

Atalhos no modo terminal (`-i`): `Enter`/`n` (próxima página), `p` (anterior), `g <número>` (ir para página), `t` (trocar tema), `m` (alternar motor), `q` (sair).

---

## Uso da GUI (Leitor Desktop — `--features gui`)

Quando compilado com `--features gui`, o comando `view --gui` (ou `-g`/`--gui` ao converter) abre uma janela desktop com 4 temas (`paper-white`, `sepia`, `night`, `e-ink-mono`).

```bash
# Abrir um EPUB existente na janela desktop
any2epub view "livro.epub" --gui --theme sepia

# Converter PDF e abrir direto no leitor
any2epub -i "documento.pdf" -o "livro.epub" --gui
```

Atalhos na janela GUI:

| Tecla | Ação |
| :--- | :--- |
| `Right` / `Down` / `PageDown` / `Space` | Próxima página |
| `Left` / `Up` / `PageUp` | Página anterior |
| `T` | Alternar temas (`PaperWhite` -> `Sepia` -> `Night` -> `EInkMono`) |
| `M` | Alternar motor (`native` <-> `koreader`) |
| `+` / `=` e `-` | Aumentar / diminuir tamanho da fonte |
| `Q` / `Esc` | Fechar janela |

> **Estado atual do motor `koreader`:** o código já procura e carrega a biblioteca `libcrengine` do sistema quando disponível (usando-a para abrir o livro e contar páginas), mas o desenho da página na tela ainda usa o motor nativo em Rust (fonte pixelada 5x7). O suporte a fontes TrueType/OTF e o desenho direto pelo `crengine` estão planejados no [ROADMAP.md](ROADMAP.md).

---

## Organização do Código

| Pasta | Papel | Documentação |
| :--- | :--- | :--- |
| `src/cli/` | Leitura de flags (`clap`), metadados iniciais e execução dos subcomandos | [src/cli/README.md](src/cli/README.md) |
| `src/domain/` | Estrutura interna do livro (`Chapter`, `TocNode`) e interface comum dos extratores | [src/domain/README.md](src/domain/README.md) |
| `src/extractors/` | Leitores de PDF, Markdown, sites de documentação e wikis | [src/extractors/README.md](src/extractors/README.md) |
| `src/epub/` | Montagem do pacote EPUB 3.3 e validador estrutural | [src/epub/README.md](src/epub/README.md) |
| `src/viewer/` | Motores de paginação/desenho, ponte `crengine`, TUI e janela desktop | [src/viewer/README.md](src/viewer/README.md) |
| `docs/` | Decisões de arquitetura e diagramas | [ADR-0001](docs/adr/0001-motor-visualizador-koreader-e-desktop-minifb.md) · [Arquitetura Viewer](docs/architecture/visualizador_koreader_rust.md) |

---

## Exemplos Reais de Conversão (EPUB 3.3)

| Obra / Documentação | Origem / Modo | Capítulos | Subtópicos (`H2`–`H4`) | Parágrafos (`<p>`) | Blocos `<pre>` | Tabelas / Avisos / Figuras | Tamanho |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| [Fundamentals of Software Architecture](https://www.oreilly.com/library/view/fundamentals-of-software/9781492043447/) | `pdf` (`lopdf` + reparo CMap) | 31 | 323 | 2.191 | 32 | 200 figuras | 6,37 MB |
| [Arch Linux Wiki](https://wiki.archlinux.org/title/Main_page) | `media-wiki` | 18 | 576 | 3.937 | 389 | 75 tbls / 267 notas / 18 figs | 3,45 MB |
| [The Rust Programming Language](https://doc.rust-lang.org/book/) | `web-docs` (`mdBook`) | 27 | 370 | 3.468 | 318 | 73 tbls / 185 notas / 27 figs | 5,03 MB |
| [MDN JavaScript Guide](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide) | `mdn-guide` (`Yari`) | 27 | 200 | 3.082 | 200 | 68 tbls / 200 notas / 27 figs | 4,95 MB |
| [Pro Git (2nd Edition)](https://git-scm.com/book/en/v2) | `ascii-doc-book` | 13 | 164 | 2.499 | 164 | 41 tbls / 164 notas / 13 figs | 2,52 MB |
| [Python Tutorial](https://docs.python.org/3/tutorial/index.html) | `sphinx-docs` | 16 | 152 | 2.328 | 152 | 41 tbls / 152 notas / 16 figs | 2,99 MB |
| [Effective Go](https://go.dev/doc/effective_go) | `single-page-spec` | 16 | 128 | 2.120 | 156 | 34 tbls / 128 notas / 16 figs | 2,95 MB |

---

## Desenvolvimento e Testes

Comandos executados pelo CI ([.github/workflows/ci.yml](.github/workflows/ci.yml)):

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --features gui -- -D warnings
cargo test --features gui
```

Licença: `MIT OR Apache-2.0`.
