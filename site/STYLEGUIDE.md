# Style Guide & Design Tokens — any2epub Web Showcase

Este documento estabelece as diretrizes de direção de arte, identidade visual e governança de design para a presença web do **any2epub** (`https://arkmedess.github.io/any2epub`).

---

## 1. Universo Visual & Personalidade

- **Arquétipo:** Editorial Técnico de Alta Performance (fusão de tipografia tipográfica editorial estilo *The Economist* / *O'Reilly* com a precisão minimalista de ferramentas modernas de infraestrutura como *Linear*, *Rust Lang* e *KOReader*).
- **Sensação Visual:** Confiança técnica, precisão tipográfica milimétrica, contraste nítido, sobriedade e velocidade instantânea.
- **Erradicação Ativa de Clichês de IA:** Zero gradientes roxo-para-ciano fluorescentes, zero cards "bento" ocos, zero textos vazios de marketing ("Desbloqueie seu potencial"). O valor é demonstrado através de artefatos reais ("Show, Don't Tell"):
  - Terminal interativo com realce de sintaxe semântico e cópia em 1 clique com `aria-live="polite"`.
  - Grid arquitetural de hairlines expostas ("Blueprint Grid") apresentando os 3 pilares de engenharia.
  - Matriz auditada das **7 Obras Técnicas** com contagem real de parágrafos, subtópicos e tamanho de arquivo.
  - Simulador interativo multi-página do leitor desktop nativo com os 4 temas cromáticos, alternância de motor (`native-rust` vs `koreader-crengine`) e zoom tipográfico.

---

## 2. Sistema de Cores & Tokens Semânticos (`oklch`)

Suporte nativo a Light & Dark Mode via `light-dark()` e variáveis `oklch`:

```css
:root {
  /* Declaração nativa de tema para navegador */
  color-scheme: light dark;

  /* Suporte dinâmico a seletor de tema via meta tag */
  &:has(> head > meta[name="color-scheme"][content="dark"]) {
    color-scheme: dark;
  }
  &:has(> head > meta[name="color-scheme"][content="light"]) {
    color-scheme: light;
  }

  /* Superfícies e Fundos (oklch) */
  --bg-canvas: light-dark(oklch(98.5% 0.005 85), oklch(13% 0.015 250));
  --surface-card: light-dark(oklch(100% 0 0), oklch(17% 0.02 250));
  --surface-raised: light-dark(oklch(96% 0.01 85), oklch(21% 0.025 250));
  --surface-container-low: light-dark(oklch(97.5% 0.006 85), oklch(15% 0.018 250));
  --surface-container-high: light-dark(oklch(93.5% 0.012 85), oklch(24% 0.028 250));
  --surface-border: light-dark(oklch(88% 0.01 85), oklch(27% 0.02 250));
  --surface-border-subtle: light-dark(oklch(92% 0.008 85), oklch(23% 0.015 250));
  --border-highlight: light-dark(oklch(1 0 0 / 0.8), oklch(1 0 0 / 0.08));

  /* Tipografia */
  --text-primary: light-dark(oklch(18% 0.02 250), oklch(96% 0.005 85));
  --text-secondary: light-dark(oklch(40% 0.02 250), oklch(76% 0.01 250));
  --text-muted: light-dark(oklch(56% 0.01 250), oklch(56% 0.01 250));

  /* Acentos de Marca (Laranja Ferrugem / Rust + E-Ink Teal) */
  --accent-rust: oklch(58% 0.20 42);
  --accent-rust-hover: oklch(52% 0.22 42);
  --accent-rust-subtle: light-dark(oklch(94% 0.03 42), oklch(24% 0.05 42));
  --accent-teal: oklch(65% 0.14 195);
  --accent-emerald: oklch(62% 0.17 150);

  /* Temas Cromáticos do Leitor */
  --theme-paper-bg: #FAF8F5;
  --theme-paper-text: #222222;
  --theme-sepia-bg: #F4ECD8;
  --theme-sepia-text: #433422;
  --theme-night-bg: #18181A;
  --theme-night-text: #D6D6D8;
  --theme-eink-bg: #EAEAE5;
  --theme-eink-text: #111111;
}
```

---

## 3. Tipografia & Hierarquia Multifatorial

- **Display & Títulos:** `system-ui` e fontes display modernas com contraste semântico bicolor no `h1` (`.title-primary` e `.title-muted` com `font-weight: 450`) e `text-wrap: balance`. Fontes serifadas (`Newsreader`, `Charter`, `Georgia`) ficam expressamente isoladas dentro do frame de leitura simulado (`.screen-mock`).
- **Interface & Navegação:** Sans-serif nítida com pesos `500` e `600` e foco visível `:focus-visible` de alto contraste.
- **Código & Terminais:** `JetBrains Mono`, `Fira Code`, `ui-monospace` com `font-variant-numeric: tabular-nums` e proteção `user-select: none` em marcadores de prompt (`$`) e comentários (`#`).

---

## 4. Componentes Chave da Experiência

1. **Dark Mode Toggle Acessível com Anti-FOUC:**
   - Script inline no `<head>` lendo `localStorage.getItem("color-scheme")`.
   - Botão interativo com ícone SVG adaptativo (Sol / Lua), `aria-label` descritivo e anúncio via `aria-live="polite"`.
2. **Terminal Hero com Cópia Rápida:**
   - Chrome de janela unificado com os 3 dots coloridos, título e botão tátil "Copiar" com feedback em 2 segundos.
3. **Blueprint Grid (Pilares de Engenharia):**
   - Hairlines contínuas de `1px solid var(--surface-border-subtle)` com 3 eixos fundamentais: *Zero C++ Runtime*, *Validação EPUB 3.3 Integrada* e *Motor Duplo a 60 FPS*.
4. **Matriz Auditada das 7 Obras em EPUB 3.3:**
   - Cards detalhados com badges de motor de entrada (`PDF lopdf`, `MediaWiki`, `mdBook`, `Single-Page Spec`, `Mozilla Yari`, `AsciiDoc`, `Sphinx Docs`), contagem de capítulos, subtópicos e parágrafos.
5. **Simulador Interativo Multi-Página KOReader / Rust:**
   - 4 temas cromáticos (`Sepia`, `Paper White`, `Night Mode`, `E-Ink Mono`).
   - Seletor de motor gráfico (`native-rust` vs `koreader-crengine`).
   - Controles de zoom tipográfico dinâmico (80% a 140%).
   - Paginação interativa por 3 capítulos reais do *Rust Book* (*Ownership*, *Borrowing*, *Lifetimes*) com barra de progresso viva.
6. **Instalação com Unified Window Chrome & WAI-ARIA:**
   - Barra de abas fundida diretamente no topo da janela do terminal.
   - Padrão WAI-ARIA completo: `role="tablist"`, `role="tab"`, `aria-selected`, `aria-controls`, `role="tabpanel"`.
   - Suporte completo a navegação por setas (`ArrowLeft`/`ArrowRight`, `Home`/`End`) e *roving tabindex*.
   - Estabilidade dimensional sem layout shift via CSS Grid overlay (`grid-area: 1 / 1`) e atributo nativo `hidden`.
