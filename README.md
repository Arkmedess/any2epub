# any2epub

Conversor em **Rust** que transforma livros PDF, arquivos Markdown e documentações técnicas da Web em **EPUB 3.3**, com foco em leitura no **Kindle desbloqueado (KOReader)** e outros e-readers. Inclui validador estrutural e leitor integrado (terminal e janela desktop).

[Showcase](https://arkmedess.github.io/any2epub/) · [Releases](https://github.com/Arkmedess/any2epub/releases) · [Roadmap](ROADMAP.md) · [Changelog](CHANGELOG.md)

---

## 1. Instalação

### Binário pronto
Baixe a versão do seu sistema nas [Releases](https://github.com/Arkmedess/any2epub/releases) (Linux `x86_64`, macOS `aarch64` ou Windows `x86_64`) e confira o hash no arquivo `SHA256SUMS`. Os binários prontos vêm sem interface gráfica.

### Compilando com Cargo

```bash
# Instalação padrão (linha de comando + modo terminal)
cargo build --release

# Com janela gráfica desktop (--features gui)
cargo build --release --features gui
```

No Ubuntu/Debian, a opção `--features gui` precisa dos pacotes de janela:

```bash
sudo apt-get install -y libx11-dev libxcursor-dev libxrandr-dev libxi-dev
```

### Docker

```bash
docker build -t any2epub:local .
docker run --rm -v "$PWD:/data" any2epub:local -i "/data/livro.pdf" -o "/data/livro.epub" --validate
```

---

## 2. Como usar

O fluxo segue três etapas: **converter**, **validar** e **visualizar**.

### Passo 1 — Converter para EPUB 3.3

Por padrão (`--source-kind auto`), o tipo da entrada é detectado pelo arquivo ou URL:

```bash
# Converter um PDF local e validar ao final
any2epub -i "livro.pdf" -o "livro.epub" --validate

# Converter uma pasta com arquivos Markdown
any2epub -i "docs/" -o "manual.epub" --validate

# Converter documentação online definindo título e idioma
any2epub \
  -i "https://doc.rust-lang.org/book/" \
  -o "rust-book.epub" \
  --title "The Rust Programming Language" \
  --lang "en" \
  --validate

# Converter wiki controlando alcance e limite de páginas
any2epub \
  -i "https://wiki.archlinux.org/title/Main_page" \
  -o "archwiki.epub" \
  --source-kind media-wiki \
  --max-depth 2 \
  --max-chapters 18
```

Modos aceitos em `--source-kind`:
- `auto`: detecta pelo arquivo (`.md` ou `.pdf`) ou padrão da URL (padrão).
- `pdf`: PDFs locais (extrai texto, capa/figuras, capitulares, notas de rodapé e corrige fontes `Calibri`/`Lekton` sem mapa de caracteres).
- `markdown`: arquivo `.md` único ou pasta com subcapítulos e blocos de aviso (`[!NOTE]`, `[!TIP]`, `[!WARNING]`).
- `web-docs`: documentações online como `mdBook`.
- `media-wiki`: wikis (`MediaWiki` / `ArchWiki`) seguindo links internos até `--max-depth` e `--max-chapters`.
- `mdn-guide`: guias do MDN Web Docs.
- `ascii-doc-book`: livros em AsciiDoc como o *Pro Git*.
- `sphinx-docs`: documentações geradas pelo Sphinx / ReadTheDocs.
- `single-page-spec`: especificações em página única divididas por subtítulos (ex.: *Effective Go*).
- `web-article`: página/artigo avulso da Web.

> Os resultados finais e saídas `--json` vão para `stdout`; mensagens de progresso vão para `stderr` (use `-q` para silenciar).

### Passo 2 — Validar o EPUB gerado

Confere a ordem do `mimetype` no ZIP, `META-INF/container.xml`, `nav.xhtml` e integridade do `content.opf`:

```bash
any2epub validate "livro.epub"
any2epub validate "livro.epub" --json
```

### Passo 3 — Ler e conferir o resultado

Você pode abrir o EPUB no **terminal** ou na **janela desktop** (se compilado com `--features gui`):

```bash
# Leitor interativo no terminal
any2epub view "livro.epub" -i

# Renderizar uma página específica no terminal
any2epub view "livro.epub" --theme sepia --page 0

# Abrir na janela desktop (requer --features gui)
any2epub view "livro.epub" --gui --theme sepia

# Converter e abrir direto na janela desktop
any2epub -i "documento.pdf" -o "livro.epub" --gui
```

Temas disponíveis (`--theme`): `paper-white`, `sepia`, `night` e `e-ink-mono`.

Motores disponíveis (`--backend`): `native` (Rust) e `koreader` (`crengine` quando `libcrengine` estiver instalada no sistema — no estado atual, o `crengine` é usado para abrir e paginar o livro, enquanto o desenho na tela ainda passa pelo motor em Rust).

**Atalhos no terminal (`-i`):**
- `Enter` ou `n`: próxima página
- `p`: página anterior
- `g <número>`: ir para página
- `t`: trocar tema
- `m`: alternar motor
- `q`: sair

**Atalhos na janela desktop (`--gui`):**
- `Right` / `Down` / `PageDown` / `Space`: próxima página
- `Left` / `Up` / `PageUp`: página anterior
- `T`: trocar tema
- `M`: alternar motor
- `+` / `=` e `-`: ajustar tamanho da fonte
- `Q` ou `Esc`: fechar

---

## 3. Exemplos testados

Conversões reais geradas e validadas em EPUB 3.3:

- **Fundamentals of Software Architecture** (`pdf`): 31 capítulos, 323 subtópicos, 2.191 parágrafos, 200 figuras (6,37 MB)
- **Arch Linux Wiki** (`media-wiki`): 18 capítulos, 576 subtópicos, 3.937 parágrafos, 389 blocos de código, 18 figuras (3,45 MB)
- **The Rust Programming Language** (`web-docs`): 27 capítulos, 370 subtópicos, 3.468 parágrafos, 318 blocos de código, 27 figuras (5,03 MB)
- **MDN JavaScript Guide** (`mdn-guide`): 27 capítulos, 200 subtópicos, 3.082 parágrafos, 200 blocos de código, 27 figuras (4,95 MB)
- **Pro Git — 2nd Edition** (`ascii-doc-book`): 13 capítulos, 164 subtópicos, 2.499 parágrafos, 164 blocos de código, 13 figuras (2,52 MB)
- **Python Tutorial** (`sphinx-docs`): 16 capítulos, 152 subtópicos, 2.328 parágrafos, 152 blocos de código, 16 figuras (2,99 MB)
- **Effective Go** (`single-page-spec`): 16 capítulos, 128 subtópicos, 2.120 parágrafos, 156 blocos de código, 16 figuras (2,95 MB)

---

## 4. Estrutura e desenvolvimento

- `src/cli/` — comandos e flags (`any2epub` e alias `pdf2epub`) ([detalhes](src/cli/README.md))
- `src/domain/` — estrutura interna dos capítulos e interface dos extratores ([detalhes](src/domain/README.md))
- `src/extractors/` — leitura de PDF, Markdown, sites e wikis ([detalhes](src/extractors/README.md))
- `src/epub/` — geração e validação de pacotes EPUB 3.3 ([detalhes](src/epub/README.md))
- `src/viewer/` — leitor terminal/desktop e ponte `crengine` ([detalhes](src/viewer/README.md))
- `docs/` — decisões de arquitetura ([ADR-0001](docs/adr/0001-motor-visualizador-koreader-e-desktop-minifb.md), [diagramas do leitor](docs/architecture/visualizador_koreader_rust.md))

Verificação local (mesmos comandos do [CI](.github/workflows/ci.yml)):

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --features gui -- -D warnings
cargo test --features gui
```

Licença: `MIT OR Apache-2.0`.
