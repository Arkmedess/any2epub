# Arquitetura do Motor de Visualização Multi-Engine e GUI Desktop

Este documento descreve a topologia arquitetural do módulo de visualização (`src/viewer/`) do `any2epub-rs`, implementado com padrão Strategy para desacoplar a apresentação do motor de paginação editorial (KOReader `crengine` e Rust puro).

---

## 1. Nível 1: Diagrama de Contexto de Sistema (System Context)

Mostra a interação do leitor/desenvolvedor com o subsistema de leitura, a CLI e os motores editoriais:

```mermaid
graph TD
    User["Leitor / Desenvolvedor"]
    CLI["CLI any2epub (Subcomando 'view' / 'validate')"]
    ViewerModule["Módulo Viewer (any2epub::viewer)"]
    KoreaderEngine["KOReader crengine (C-ABI FFI)"]
    RustEngine["Motor Nativo em Rust Puro"]
    DisplayTerminal["Terminal TUI (stdout / ANSI)"]
    DisplayDesktop["Janela Desktop (minifb Framebuffer)"]

    User -->|"view livro.epub -i"| CLI
    User -->|"view livro.epub --gui"| CLI
    CLI -->|"Carrega EPUB e inicializa opções"| ViewerModule
    ViewerModule -->|"Delega layout e paginação"| KoreaderEngine
    ViewerModule -->|"Delega layout e paginação"| RustEngine
    ViewerModule -->|"Desenha texto formatado e barra"| DisplayTerminal
    ViewerModule -->|"Transfere PageBitmap (RGBA8888 -> ARGB32)"| DisplayDesktop
```

---

## 2. Nível 2: Diagrama de Contêineres e Módulos (Container / Top-Level Architecture)

Ilustra o isolamento entre a camada CLI, a abstração da biblioteca e os adaptadores de visualização:

```mermaid
graph LR
    subgraph any2epub Binário & Biblioteca
        Args["cli::args (CliArgs / CliCommand)"]
        Runner["cli::runner (Orquestrador)"]
        Validator["epub::validator (W3C EPUB 3.3)"]
        Extractors["extractors (PDF, Markdown, Web, MediaWiki, Spec)"]
        ViewerFacade["viewer::mod (create_reader_engine)"]
    end

    subgraph Viewer Internals
        TraitEngine["viewer::engine::ReaderEngine (Trait)"]
        EngineNative["viewer::native::NativeRustReaderEngine"]
        EngineKoreader["viewer::koreader::KoreaderCrengineEngine"]
        Options["viewer::options (Theme, RenderOptions)"]
        Bitmap["viewer::bitmap::PageBitmap (RGBA8888)"]
        TUI["viewer::interactive (TUI Session)"]
        GUI["viewer::gui (minifb Window 60 FPS)"]
    end

    Args --> Runner
    Runner --> Validator
    Runner --> Extractors
    Runner --> ViewerFacade
    ViewerFacade --> TraitEngine
    TraitEngine <|-- EngineNative
    TraitEngine <|-- EngineKoreader
    TraitEngine --> Options
    TraitEngine --> Bitmap
    Runner --> TUI
    Runner --> GUI
    TUI --> TraitEngine
    GUI --> TraitEngine
```

---

## 3. Nível 3: Diagrama de Componentes do Módulo Viewer

Detalha o ciclo de vida de renderização de página e o chaveamento dinâmico em tempo real:

```mermaid
sequenceDiagram
    autonumber
    actor Leitor
    participant GUI as viewer::gui::run_gui_viewer
    participant Engine as ReaderEngine (Polimórfico)
    participant Bitmap as PageBitmap (RGBA8888)
    participant Framebuffer as minifb::Window (ARGB32)

    Leitor->>GUI: Executa `any2epub view livro.epub --gui`
    GUI->>Engine: open_epub(path)
    Engine-->>GUI: Ok(total_pages)
    
    loop Loop Principal (60 FPS)
        GUI->>Engine: render_page(current_page, &options)
        Engine-->>GUI: PageBitmap
        GUI->>Bitmap: to_argb_u32()
        Bitmap-->>GUI: Vec<u32> (ARGB nativo)
        GUI->>Framebuffer: update_with_buffer(&argb_pixels, width, height)
        
        alt Tecla 'M' Pressionada (Trocar Motor)
            Leitor->>GUI: Pressiona 'M'
            GUI->>Engine: create_reader_engine(outro_backend)
            GUI->>Engine: open_epub(path)
        else Tecla 'T' Pressionada (Trocar Tema)
            Leitor->>GUI: Pressiona 'T'
            GUI->>GUI: options.color_theme.next()
        else Setas / Espaço Pressionadas (Navegar)
            Leitor->>GUI: Avança / Recua página
            GUI->>GUI: current_page = clamp(target)
        end
    end
```

---

## 4. Temas Cromáticos Suportados

Os seguintes esquemas editoriais são pré-calibrados tanto para o terminal quanto para a interface desktop:

| Tema (`ViewerColorTheme`) | Fundo (`background`) | Texto (`foreground`) | Destaque (`highlight`) | Caso de Uso Recomendado |
| :--- | :---: | :---: | :---: | :--- |
| **`PaperWhite`** | `#FFFFFF` (255, 255, 255) | `#1A1A1A` (26, 26, 26) | `#0066CC` | Leitura padrão diurna em alta definição. |
| **`Sepia`** | `#FBF0D9` (251, 240, 217) | `#433422` (67, 52, 34) | `#8F5902` | Leitura confortável com baixa fadiga visual. |
| **`Night`** | `#1E1E1E` (30, 30, 30) | `#CCCCCC` (204, 204, 204) | `#4DA6FF` | Leitura noturna em ambientes escuros (Dark Mode). |
| **`EInkMono`** | `#F5F5F5` (245, 245, 245) | `#000000` (0, 0, 0) | `#555555` | Simulação de telas E-Ink monocromáticas e e-readers. |
