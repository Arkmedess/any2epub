# Módulo de Linha de Comando (`any2epub::cli`)

## 1. Propósito da Fatia
A fatia vertical `cli` implementa a interface de linha de comando oficial do `any2epub-rs`. Ela fornece o parsing e a validação de argumentos de terminal (via `clap`), os subcomandos dedicados (`validate` e `view`), o fluxo de conversão universal ponta a ponta e a orquestração amigável de relatórios de progresso em texto puro e JSON estruturado para pipelines de automação e CI/CD.

---

## 2. Contratos Públicos e Assinaturas

### A. Definição de Argumentos e Comandos
Localizada em `src/cli/args.rs`:

```rust
pub struct CliArgs {
    pub input: Option<String>,
    pub output: Option<String>,
    pub source_kind: Option<SourceKind>,
    pub max_depth: Option<usize>,
    pub max_chapters: Option<usize>,
    pub command: Option<CliCommand>,
    pub validate: bool,
    pub view: bool,
    pub gui: bool,
}

pub enum CliCommand {
    Validate {
        path: String,
        json: bool,
    },
    View {
        path: String,
        backend: String,
        theme: String,
        page: Option<usize>,
        interactive: bool,
        gui: bool,
    },
}
```

### B. Orquestrador de Execução
Localizado em `src/cli/runner.rs`:

```rust
pub fn run_cli_app(args: CliArgs) -> Result<(), Box<dyn std::error::Error>>
```

- Processa comandos diretos (`validate` ou `view`) ou executa o pipeline de conversão com validação e visualização pós-processamento opcionais (`--validate`, `--view`, `--gui`).
- Sanitiza caminhos locais e URLs remotas (RFC 3986).

---

## 3. Invariantes de Domínio
1. **Comportamento Determinístico de Saída:** Quando nenhum subcomando for fornecido, a CLI exige obrigatoriamente um caminho/URL de entrada válido via `-i`/`--input`.
2. **Separação Canônica entre stdout e stderr:** Mensagens de log, status e progresso fluem para `stderr` ou para a interface TUI, enquanto relatórios estruturados sob a flag `--json` são canalizados com exclusividade para `stdout`.
3. **Códigos de Saída Semânticos:** Código `0` para sucesso e `1` para erros de conversão, I/O ou falha de conformidade na validação estrutural.

---

## 4. Dependências e Acoplamento
- **Módulos Internos Consumidos:**
  - `src/domain/`: Representações intermediárias e metadados.
  - `src/extractors/`: Fábrica e instâncias de extração polimórfica.
  - `src/epub/`: Construtor e validador canônico W3C EPUB 3.3.
  - `src/viewer/`: Motores de leitura TUI e GUI desktop nativa.
- **Crates Externas Principais:** `clap` (derive v4), `serde` e `serde_json`.

---

## 5. Testes Automatizados e Verificação

```bash
# Executar suíte de testes de argumentos e comandos da CLI
cargo test --test cli_command_test

# Executar linter estrito
cargo clippy --all-targets -- -D warnings
```
