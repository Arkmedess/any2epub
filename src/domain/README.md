# Módulo de Domínio e Representação Intermediária (`any2epub::domain`)

## 1. Propósito da Fatia
A fatia vertical `domain` define o modelo semântico fundamental e os contratos centrais do `any2epub-rs`. Ela isola os tipos que representam livros, metadados, capítulos, elementos tipográficos, nós hierárquicos do sumário (TOC) e define a trait polimórfica `ContentSourceExtractor`, assegurando que o domínio permaneça puro e desacoplado de formatos de arquivo de entrada ou bibliotecas concretas de I/O.

---

## 2. Contratos Públicos e Assinaturas

### A. Trait Central `ContentSourceExtractor`
Localizada em `src/domain/traits.rs`:

```rust
pub trait ContentSourceExtractor: Send + Sync {
    /// Identifica a fonte para fins de log e rastreabilidade.
    fn name(&self) -> &'static str;

    /// Extrai o conteúdo da fonte e compõe o livro na representação intermediária.
    fn extract(&self) -> Result<IntermediateBook, Box<dyn std::error::Error>>;
}
```

### B. Representação Intermediária (IR)
Localizada em `src/domain/ir.rs`:

- `IntermediateBook`: Contém `BookMetadata`, coleção de `Chapter`, nós de sumário `TocNode` e imagens/figuras globais.
- `Chapter`: Título, nome do arquivo XHTML de destino, elementos de conteúdo estruturados e subtópicos associados.
- `ChapterElement`: Tipos semânticos do documento:
  - `Paragraph(String)`
  - `Heading { level: u8, text: String, anchor_id: String }`
  - `CodeBlock { code: String, language: Option<String> }`
  - `Blockquote(String)`
  - `List { ordered: bool, items: Vec<String> }`
  - `Table { headers: Vec<String>, rows: Vec<Vec<String>> }`
  - `CalloutBox { kind: String, title: Option<String>, content: String }`
  - `FigureRef { figure_id: String, caption: Option<String> }`

### C. Metadados e Figuras
Localizada em `src/domain/metadata.rs`:

- `BookMetadata`: Título, autores, identificador UUID/URN, idioma, editora, data e imagem de capa opcional.
- `ExtractedFigure`: Identificador único, tipo MIME (`image/jpeg`, `image/png`) e bytes binários da figura.

---

## 3. Invariantes de Domínio
1. **Identificadores Únicos e Imutáveis:** Todo `TocNode` e `Chapter` deve possuir referências consistentes para ancoragem semântica no EPUB final (`anchor_id` slugificado).
2. **Imutabilidade e Segurança Tipográfica:** Elementos do domínio não sofrem mutação concorrente descontrolada durante o empacotamento ou validação.
3. **Independência de Frameworks de I/O:** O domínio não possui dependências de renderização gráfica, rede ou drivers de banco de dados.

---

## 4. Dependências e Acoplamento
- **Módulos Internos Consumidos:** `src/utils/` (higienização de textos e slugificação de âncoras).
- **Dependências Externas:** `serde` (para serialização opcional).

---

## 5. Testes Automatizados e Verificação

```bash
# Executar testes unitários do domínio e pipeline
cargo test --test epub_pipeline_test
```
