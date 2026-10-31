# Módulo de Visualização Multi-Engine (`any2epub::viewer`)

## 1. Propósito da Fatia
A fatia vertical `viewer` provê um motor polimórfico de visualização e leitura de e-books EPUB 3.3. Ela desacopla completamente a lógica de renderização e paginação das camadas de apresentação (Terminal TUI interativo e Janela Desktop Nativa a 60 FPS), oferecendo suporte transparente ao motor canônico `crengine` do KOReader via FFI e a um motor 100% nativo em Rust.

---

## 2. Contratos Públicos e Assinaturas

### A. Trait Central `ReaderEngine`
Localizada em `src/viewer/engine.rs`:

```rust
pub trait ReaderEngine: Send + Sync {
    /// Nome identificador do motor (ex.: "native-rust" ou "koreader-crengine").
    fn name(&self) -> &'static str;

    /// Carrega o arquivo EPUB e calcula o total de páginas paginadas.
    fn open_epub(&mut self, epub_path: &Path) -> Result<usize, ReaderEngineError>;

    /// Renderiza uma página específica em um framebuffer RGBA.
    fn render_page(
        &mut self,
        page_index: usize,
        options: &ViewerRenderOptions,
    ) -> Result<PageBitmap, ReaderEngineError>;

    /// Extrai o sumário de navegação com os títulos dos capítulos e índice de página inicial.
    fn table_of_contents(&self) -> Vec<(String, usize)>;

    /// Retorna o total de páginas calculado para a geometria atual.
    fn total_pages(&self) -> usize;
}
```

### B. Fábrica Polimórfica
```rust
pub fn create_reader_engine(backend: ViewerBackendKind) -> Box<dyn ReaderEngine>
```

### C. Estruturas de Dados e Configuração
- `PageBitmap`: Framebuffer rasterizado em bytes contíguos RGBA8888 com método `to_argb_u32()` para consumo por subsistemas nativos X11/Wayland/Win32.
- `ViewerColorTheme`: Esquemas cromáticos `PaperWhite`, `Sepia`, `Night` e `EInkMono`.
- `ViewerRenderOptions`: Largura, altura, DPI, tamanho de fonte e tema de cores ativo.

### D. Camadas de Apresentação
- `run_interactive_viewer(path, engine, theme)`: Sessão de terminal TUI com barra de progresso gráfica, atalhos de teclado e ciclo de temas.
- `run_gui_viewer(path, initial_backend, theme)`: Janela desktop nativa interativa gerenciada por `minifb` (condicionada à feature flag `gui`).

---

## 3. Invariantes de Domínio
1. **Fallback e Disponibilidade Garantida:** Se a biblioteca compartilhada do KOReader (`libcrengine.so` ou `crengine.dll`) não estiver presente no sistema operacional do host, o backend `KoreaderCrengineEngine` opera em modo simulado/bridge seguro emitindo aviso e provendo layout tipográfico funcional sem interrupção abrupta ou panics.
2. **Imutabilidade do Framebuffer por Frame:** Toda chamada a `render_page` gera um `PageBitmap` com dimensões e contagem de bytes estritamente congruentes com `width * height * 4`.
3. **Respeito ao Limite de Páginas:** Índices fora dos limites `[0, total_pages - 1]` retornam `ReaderEngineError::PageOutOfBounds` determinístico.

---

## 4. Dependências e Acoplamento
- **Módulos Internos Consumidos:** `src/epub/` (leitura de arquivos zip e DOM XHTML do EPUB).
- **Dependências Externas Obrigatórias:** Nenhuma dependência pesada de sistema (Rust puro).
- **Dependências Externas Opcionais:** `minifb` ativada apenas sob a feature flag `gui` (`Cargo.toml`).

---

## 5. Testes Automatizados e Verificação

Para executar os testes automatizados da fatia:

```bash
# Executar testes unitários e de integração do módulo viewer
cargo test --test viewer_engine_test --test interactive_viewer_test --features gui

# Verificar conformidade estrita de tipagem e linter
cargo clippy --features gui --all-targets -- -D warnings
```
