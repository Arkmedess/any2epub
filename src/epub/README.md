# Módulo de Empacotamento e Validação EPUB 3.3 (`any2epub::epub`)

## 1. Propósito da Fatia
A fatia vertical `epub` é responsável por transformar o modelo intermediário (`IntermediateBook`) em um e-book [EPUB 3.3 (W3C)](https://www.w3.org/TR/epub-33/) canônico e válido. Além do empacotamento com compressão determinística, provê um validador estrutural nativo independente (`NativeEpubValidator`) que audita conformidade OCF, Dublin Core no OPF e integridade referencial manifest/spine.

---

## 2. Contratos Públicos e Assinaturas

### A. Construtor Nativo `NativeEpub3Builder`
Localizado em `src/epub/mod.rs`:

```rust
pub struct NativeEpub3Builder;

impl NativeEpub3Builder {
    pub fn build(book: &IntermediateBook, output_path: &Path) -> Result<(), Box<dyn std::error::Error>>;
}
```

- Escreve o container OCF: arquivo `mimetype` descompactado (`Stored`) estritamente nos primeiros 58 bytes do arquivo zip (offset 0).
- Gera `META-INF/container.xml` apontando para o arquivo de metadados do pacote (`OEBPS/content.opf`).
- Produz `content.opf` com Dublin Core 1.1, identificador UUID/URN e manifesto mapeado item a item.
- Serializa páginas em XHTML5 estrito (`OEBPS/chapter_*.xhtml`, `nav.xhtml`, `cover.xhtml`, `stylesheet.css`).

### B. Validador Estrutural Canônico W3C `NativeEpubValidator`
Localizado em `src/epub/validator.rs`:

```rust
pub struct NativeEpubValidator;

impl NativeEpubValidator {
    pub fn validate(epub_path: &Path) -> Result<EpubValidationReport, EpubValidationError>;
}
```

- Retorna um `EpubValidationReport` contendo status booleano `valid`, lista de avisos, contagem de arquivos e conformidade Dublin Core.

---

## 3. Invariantes de Domínio
1. **Offset Zero do Mimetype:** O arquivo `mimetype` (`application/epub+zip`) deve ser o primeiríssimo registro no arquivo ZIP, sem compressão (método Stored) e sem campos de cabeçalho extra.
2. **Conformidade Referencial Estrita:** Todo arquivo XHTML referenciado no `<spine>` deve existir no `<manifest>` com atributos `id` e `media-type` corretos.
3. **Validade XHTML5:** Todo documento de conteúdo serializado deve fechar todas as tags (`<br/>`, `<img/>`, `<link/>`) e conter a declaração `<!DOCTYPE html>`.

---

## 4. Dependências e Acoplamento
- **Módulos Internos Consumidos:** `src/domain/` (IR e metadados) e `src/utils/` (escapamento XML e expressões regulares).
- **Crates Externas Principais:** `zip`, `uuid`, `quick-xml`.

---

## 5. Testes Automatizados e Verificação

```bash
# Executar testes de empacotamento e validação W3C EPUB 3.3
cargo test --test epub_pipeline_test --test epub_validation_test
```
