# Módulo de Extratores Polimórficos de Conteúdo (`any2epub::extractors`)

## 1. Propósito da Fatia
A fatia vertical `extractors` implementa a ingestão e extração profunda de conteúdo a partir de múltiplas fontes heterogêneas (livros em PDF, coleções de Markdown locais, páginas e grafos Web). Cada extrator encapsula as peculiaridades de sua mídia de origem e converte o material para a representação intermediária pura (`IntermediateBook`).

---

## 2. Contratos Públicos e Assinaturas

### A. Fábrica Polimórfica `build_extractor`
Localizada em `src/extractors/mod.rs`:

```rust
pub fn build_extractor(
    input: &str,
    source_kind: Option<SourceKind>,
    max_depth: Option<usize>,
    max_chapters: Option<usize>,
) -> Result<Box<dyn ContentSourceExtractor>, Box<dyn std::error::Error>>
```

### B. Extratores Implementados
- **`LocalMarkdownExtractor` (`src/extractors/markdown.rs`):**
  - Processa arquivos únicos `.md` ou diretórios recursivos com ordenação natural.
  - Extrai hierarquia de cabeçalhos (`#` a `####`), blocos de código com linguagem e caixas de aviso GitHub-style (`> [!NOTE]`, `> [!TIP]`, `> [!WARNING]`).
- **`PdfExtractor` (`src/extractors/pdf/`):**
  - Processa PDFs através de decodificação de fontes CID sem `/ToUnicode` (`repair_broken_cmap_cid`), extração de figuras e diagramas em alta resolução, drop caps e notas de rodapé.
- **`WebSiteExtractor` (`src/extractors/web.rs`):**
  - Varredura recursiva BFS em sumários multinível para portais técnicos ([mdBook](https://rust-lang.github.io/mdBook/), [Sphinx](https://www.sphinx-doc.org/), [Pro Git AsciiDoc](https://git-scm.com/book/en/v2), [MDN Yari](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide)).
  - Extração de nós folha isolados (`li`, `dt`, `dd`, `p`, `pre`, `table`) prevenindo duplicação em caixas de callout.
- **`MediaWikiExtractor` (`src/extractors/mediawiki.rs`):**
  - Ingestão de tópicos em grafos com tratamento de tabelas `.wikitable` e remoção de elementos de navegação/edição da wiki.
- **`SinglePageSpecExtractor` (`src/extractors/spec.rs`):**
  - Segmentação cirúrgica de especificações técnicas em página única (como o [Effective Go](https://go.dev/doc/effective_go)).

---

## 3. Invariantes de Domínio
1. **Sanitização de Caracteres e Quebras:** Texto bruto extraído de PDFs ou Web passa por normalização de quebras de linha duras antes de integrar a IR.
2. **Prevenção de Duplicação no DOM:** Nós internos a caixas de callout ou tabelas não devem ser re-emitidos como parágrafos soltos fora de seu contêiner semântico (`is_inside_callout_or_table`).
3. **Escopo Canônico de URLs:** Rastreamento Web via BFS restringe-se estritamente ao domínio e caminho base do livro configurado.

---

## 4. Dependências e Acoplamento
- **Módulos Internos Consumidos:** `src/domain/` e `src/utils/`.
- **Crates Externas Principais:** `lopdf`, `pdfium-render`, `scraper`, `reqwest`, `pulldown-cmark`.

---

## 5. Testes Automatizados e Verificação

```bash
# Executar testes da suíte de extração
cargo test --test markdown_extraction_test --test web_extraction_test --test url_resolution_test
```
