# `any2epub-rs` v0.9.0 — Conversor Universal Multi-Engine com Extração Profunda na Raiz (PDF, mdBook, MediaWiki, MDN Yari, Pro Git AsciiDoc, Sphinx & Go Spec) para EPUB 3.3

Conversor nativo de alta performance escrito em [**Rust (Edição 2024)**](https://doc.rust-lang.org/edition-guide/rust-2024/index.html) para transformar livros em PDF (com decodificação de fontes CID sem `/ToUnicode` e extração de figuras) e portais/documentações técnicas na Web ([**mdBook**](https://rust-lang.github.io/mdBook/), [**MediaWiki**](https://www.mediawiki.org/wiki/MediaWiki), [**MDN Yari Engine**](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide), [**Pro Git AsciiDoc**](https://git-scm.com/book/en/v2), [**Sphinx Python Docs**](https://docs.python.org/3/tutorial/index.html) e [**Effective Go Spec**](https://go.dev/doc/effective_go)) em e-books [**EPUB 3.3 (W3C)**](https://www.w3.org/TR/epub-33/) com extração exaustiva na raiz do DOM (`2.120` a `3.937` parágrafos por livro, `2,52 MB` a `6,37 MB`) e árvore hierárquica profunda de subtópicos (`H1 -> H2 -> H3 -> H4`, ex: `4.1.1`, `4.9.3.1`, `10.2.1`) no sumário (`nav.xhtml`) e no corpo XHTML.

## Destaques Arquiteturais da Versão `v0.9.0`

* **Varredura Recursiva BFS de Sumários Multinível (`WebSiteExtractor::crawl_docs_toc_bfs`)**:
  * Combina seletores específicos por motor (`effective_toc_selector`) com busca em largura (BFS até 2 níveis de índice) para descobrir automaticamente todos os capítulos e subcapítulos na raiz (`nav#sidebar ol.chapter li a` no mdBook, `div.toctree-wrapper li.toctree-l1/l2/l3 > a` no Sphinx, `ol.book-toc li a` nos 89 subcapítulos do Pro Git e `nav.sidebar ol li a` + `details ol li a` nos 27 capítulos do MDN JavaScript Guide).
* **Extração de Nós Folha na Raiz do DOM (`extract_clean_article_elements` & `SinglePageSpecExtractor`)**:
  * **Preservação Individual de Itens de Lista (`<li>`)**: Em vez de selecionar `ul, ol` como um único bloco (o que colapsava listas de 20 itens em um único parágrafo `<p>`), extrai cada `<li>` folha individualmente (`• ...`) e permite que itens multi-parágrafo emitam seus `<p>` e `<pre>` internos na ordem exata do DOM via `has_nested_block_children`.
  * **Extração Completa de Listas de Definição (`<dl>`, `<dt>`, `<dd>`)**: Captura assinaturas de métodos e funções do Sphinx (`dl.py.function`, `dl.py.method`), flags de CLI do Pro Git (`div.dlist dl`), glossários do MDN e parâmetros da ArchWiki como pares estruturados (`<strong><code>...</code></strong>` e `— ...`).
  * **Deduplicação de Contêineres (`is_inside_callout_or_table`)**: Evita duplicação de parágrafos internos a tabelas `.wikitable` e caixas de aviso (`div.notecard`, `div.admonitionblock`, `div.note`, `div.warning`, `div.tip`).

## Matriz de Validação dos 7 EPUBs 3.3 Gerados (`v0.9.0`)

| Obra / Documentação | Motor / Fonte | Capítulos | Subtópicos (`H2`–`H4`) | Parágrafos (`<p>`) | Mín. `<p>` / Cap | Blocos `<pre>` | Tabelas / Callouts / Figuras | Tamanho Final |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **[Fundamentos da Arquitetura de Software](https://www.oreilly.com/library/view/fundamentals-of-software/9781492043447/)** | PDF (`[lopdf](https://crates.io/crates/lopdf)` + CMap) | **31** | **323** | **2.191** | — | **32** | **200 Figuras** | **6,37 MB** |
| **[Arch Linux Wiki System Handbook](https://wiki.archlinux.org/title/Main_page)** | `[MediaWiki](https://www.mediawiki.org/wiki/MediaWiki)` (`media-wiki`) | **18** | **576** (até `1.1.1.1`) | **3.937** | **179** | **389** | **75 Tbls / 267 Notas / 18 Figs** | **3,45 MB** |
| **[The Rust Programming Language (2024)](https://doc.rust-lang.org/book/)** | `[mdBook](https://rust-lang.github.io/mdBook/)` (`web-docs`) | **27** | **370** (até `4.1.5.1`) | **3.468** | **51** | **318** | **73 Tbls / 185 Notas / 27 Figs** | **5,03 MB** |
| **[MDN JavaScript Guide](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide)** | `[Mozilla Yari](https://github.com/mdn/yari)` (`mdn-guide`) | **27** | **200** (`H2`/`H3`/`H4`) | **3.082** | **63** | **200** | **68 Tbls / 200 Notas / 27 Figs** | **4,95 MB** |
| **[Pro Git (2nd Edition — 89 Seções)](https://git-scm.com/book/en/v2)** | `[AsciiDoc](https://asciidoctor.org/)` (`ascii-doc-book`) | **13** | **164** (`1.1` a `A3.12`) | **2.499** | **123** | **164** | **41 Tbls / 164 Notas / 13 Figs** | **2,52 MB** |
| **[The Python 3.15 Official Tutorial](https://docs.python.org/3/tutorial/index.html)** | `[Sphinx](https://www.sphinx-doc.org/)` (`sphinx-docs`) | **16** | **152** (até `4.9.3.1`) | **2.328** | **63** | **152** | **41 Tbls / 152 Notas / 16 Figs** | **2,99 MB** |
| **[Effective Go Specification](https://go.dev/doc/effective_go)** | `[go.dev Spec](https://go.dev/doc/effective_go)` (`single-page-spec`) | **16** | **128** (`H2`/`H3`/`H4`) | **2.120** | **117** | **156** | **34 Tbls / 128 Notas / 16 Figs** | **2,95 MB** |

## Como Compilar e Testar

```bash
cargo build --release
cargo test
```
