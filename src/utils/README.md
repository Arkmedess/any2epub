# Módulo de Utilitários Comuns (`any2epub::utils`)

## 1. Propósito da Fatia
A fatia horizontal de baixo nível `utils` centraliza funções auxiliares puras reutilizadas transversalmente em todo o projeto. Ela abrange higienização de strings para conformidade estrita com XML/XHTML5, slugificação determinística de identificadores de âncoras e compilação em cache (lazy) de expressões regulares via autômatos finitos determinísticos (DFA).

---

## 2. Contratos Públicos e Assinaturas

### A. Higienização e Manipulação de Texto
Localizada em `src/utils/text.rs`:

```rust
/// Escapa caracteres reservados para segurança em XML/XHTML5 (&, <, >, ", ').
pub fn escape_xml(input: &str) -> String;

/// Converte um texto de cabeçalho em um identificador de âncora válido para URLs e XHTML.
pub fn slugify_anchor_id(input: &str) -> String;
```

### B. Expressões Regulares Compiladas em Cache
Localizada em `src/utils/regexes.rs`:

- Centraliza expressões regulares estáticas usando `std::sync::OnceLock<Regex>` para evitar sobrecarga de recompilação em loops ou workers de extração concorrente.

---

## 3. Invariantes de Domínio
1. **Total Determinismo em Escapamento:** Caracteres como `&`, `<`, `>`, `"` e `'` são convertidos deterministicamente em suas respectivas entidades de entidade XML (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`).
2. **Âncoras Limpas:** Slugificadores não geram espaços em branco ou caracteres especiais incompatíveis com a especificação de URI/XHTML id.

---

## 4. Dependências e Acoplamento
- **Módulos Internos Consumidos:** Nenhum (camada base isolada).
- **Crates Externas Principais:** `regex`.

---

## 5. Testes Automatizados e Verificação

```bash
# Executar suíte de testes de validação que consom utilitários
cargo test --test epub_validation_test
```
