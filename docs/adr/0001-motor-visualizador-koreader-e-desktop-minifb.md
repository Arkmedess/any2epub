# 0001: Motor Visualizador Multi-Engine (KOReader crengine / Rust Puro) e Interface Desktop Nativa via minifb

* **Status:** aceito
* **Data:** 2026-10-30
* **Decisores:** Arthur & Antigravity AI
* **Módulos Afetados:** `viewer`, `cli`, `Cargo.toml`

## 1. Contexto e Declaração do Problema
O `any2epub-rs` converte documentos estruturados (PDF, Markdown, mdBook, MediaWiki, MDN, Sphinx, AsciiDoc) em e-books EPUB 3.3 em conformidade estrita com a W3C. Entretanto, os desenvolvedores e leitores precisavam de um meio determinístico e imediato de auditar e ler os arquivos gerados sem depender de leitores de terceiros (como Calibre ou navegadores), com fidelidade ao consagrado motor de paginação editorial do [KOReader](https://koreader.rocks/) (`crengine`) e com uma experiência desktop rápida e leve.

Além disso, para ambientes de servidores ou contêineres CI/CD headless, o binário não podia exigir obrigatoriamente dependências pesadas de bibliotecas gráficas C++ ou X11/Wayland em seu modo padrão.

## 2. Direcionadores de Decisão (Drivers)
* **Driver 1 (Fidelidade Editorial KOReader):** Disponibilizar a capacidade de renderizar o EPUB utilizando exatamente o motor C-ABI `crengine` do KOReader, preservando seu algoritmo de quebra de página e hifenização.
* **Driver 2 (Portabilidade e Zero Dependências Nativas Obrigatórias):** Permitir que o sistema opere com um motor 100% nativo em Rust (`NativeRustReaderEngine`), garantindo compilação pura e execução imediata em qualquer plataforma.
* **Driver 3 (Interface Gráfica de Alto Desempenho e Baixo Overhead):** Fornecer visualização desktop interativa a 60 FPS com controles de teclado, sem os gigabytes de memória de runtimes baseadas em Web (Chromium/Electron/Tauri).
* **Driver 4 (Modularidade e Feature-Gating):** A dependência de interface gráfica deve ser puramente opcional via Cargo Feature (`gui`), preservando o binário headless leve para conversão em pipelines automatizados.

## 3. Opções Consideradas
* **Opção 1:** Visualização via Webview / Tauri / Electron.
* **Opção 2:** Framework de GUI Reativo Completo (Slint, Iced ou egui).
* **Opção 3:** Abstração Polimórfica `ReaderEngine` (KOReader FFI + Rust Puro) com Framebuffer Nativo via `minifb` (Feature Opcional `gui`) e Leitor Interativo no Terminal (TUI).

## 4. Decisão Escolhida
Escolhida a **Opção 3**, porque atende simultaneamente aos requisitos de máxima fidelidade tipográfica (KOReader `crengine`), portabilidade completa (Rust puro), baixíssimo consumo de memória (framebuffer nativo de ~3 MB para 800x1050 pixels) e isolamento arquitetural por meio de uma feature flag `gui` opcional no Cargo.

## 5. Prós e Contras das Opções

### Opção 1: Visualização via Webview / Tauri / Electron
* Boa, porque reaproveitaria motores de renderização HTML/CSS completos dos navegadores.
* Ruim, porque impõe centenas de megabytes de dependências transitivas, compilações lentas, alto consumo de memória RAM e não resolve a exigência do motor KOReader `crengine`.

### Opção 2: Framework de GUI Reativo Completo (Slint, Iced ou egui)
* Boa, porque oferece gerenciamento de janelas e árvores de componentes ricas.
* Ruim, porque adiciona dezenas de crates complexas (wgpu, winit, glutin) e modelos de reatividade desnecessários para uma aplicação cujo propósito é a exibição paginada de um `PageBitmap` gerado por um motor editorial.

### Opção 3: ReaderEngine Polimórfico + Framebuffer minifb + TUI
* Boa, porque o `minifb` conecta-se diretamente ao subsistema de exibição nativo do SO (X11 no Linux, Cocoa no macOS, Win32 no Windows) com consumo mínimo de CPU/RAM, operando a 60 FPS constantes.
* Boa, porque permite alternar em tempo real entre o motor `koreader-crengine` e `native-rust` pressionando uma única tecla (`M`), inclusive durante a leitura.
* Boa, porque a feature `gui` é opcional (`cargo build --features gui`), mantendo o binário padrão (`cargo build`) totalmente enxuto para servidores.
* Ruim, porque requer a gestão manual de eventos de teclado e cópia de buffer de pixels (`to_argb_u32`), o que foi prontamente encapsulado em `src/viewer/gui.rs` e `src/viewer/bitmap.rs`.

## 6. Consequências e Trade-offs
* **Impacto Positivo:**
  - O usuário final pode ler EPUBs no terminal interativo (`any2epub view livro.epub -i`) ou em uma janela desktop nativa (`any2epub view livro.epub --gui`).
  - O desenvolvedor pode auditar a quebra de página de ambos os motores (`crengine` e `native`) lado a lado.
  - Zero impacto no tempo de compilação ou tamanho do binário quando compilado sem a feature `gui`.
* **Impacto Negativo / Débito Aceito:**
  - Para usar a janela gráfica, a compilação requer a flag `--features gui`.
* **Ações Mitigatórias:**
  - A documentação e o script de empacotamento da release (`dist/`) foram atualizados para incluir os binários pré-compilados com suporte a GUI por padrão para distribuições desktop Linux x86_64.
