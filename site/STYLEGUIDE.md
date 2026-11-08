# Style Guide & Design Tokens — any2epub Web Showcase

Este documento estabelece as diretrizes de direção de arte, identidade visual, governança de tokens e arquitetura frontend para a presença web do **any2epub** (`https://arkmedess.github.io/any2epub`), incorporando os princípios de engenharia e refinamento estético de **Dux Labs**, **Google Antigravity** e **Laravel**.

---

## 1. Universo Visual & Tríade de Inspiração Autoral

- **Arquétipo Geral:** Editorial Técnico de Alta Performance (fusão da sobriedade de infraestrutura de sistemas com o requinte de publicações tipográficas).
- **Tríade de Referências de Excelência:**
  1. **Dux Labs (`https://duxlabs.dev/`):**
     - Shell de interface em camadas tonais progressivas (`--surface-container-low`, `--surface-card`, `--surface-raised`, `--surface-container-high`).
     - *Unified Window Chrome* com identificador de contexto do workspace (`~/projects/any2epub · ⑂ main · UTF-8`), abas de terminal fundidas à barra de título e realce de sintaxe semântico inspirado no algoritmo `dank16`.
  2. **Google Antigravity (`https://antigravity.google/`):**
     - Pill flutuante de anúncio (`.top-announcement-pill`) com dot pulsante de status ativo (`@keyframes pulse-dot`).
     - Títulos display bicolores com tracking apertado (`-0.035em`) combinando declaração primária com cláusula em `--text-secondary` (`font-weight: 450`).
     - Seletor interativo de personas/casos de uso (*DevOps & Infraestrutura*, *Autores & Pesquisadores*, *Leitores & E-Ink*).
  3. **Laravel (`https://laravel.com/`):**
     - Grid arquitetural de hairlines expostas ("Blueprint Grid") estruturando as seções com linhas contínuas de `1px solid var(--surface-border-subtle)`.
     - Textura sutil de fundo em dot matrix (`--bg-grid-dot` radial com opacidade $\le 0.05$).
     - Faixa de métricas de engenharia com números tabulares de alto impacto (`tabular-nums`).

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

  /* Superfícies e Fundos (oklch) — Hierarquia Tonal em Camadas estilo Dux Labs */
  --bg-canvas: light-dark(oklch(98.5% 0.005 85), oklch(13% 0.015 250));
  --surface-container-lowest: light-dark(oklch(99.5% 0.002 85), oklch(11% 0.012 250));
  --surface-container-low: light-dark(oklch(97.5% 0.006 85), oklch(15% 0.018 250));
  --surface-card: light-dark(oklch(100% 0 0), oklch(17% 0.02 250));
  --surface-raised: light-dark(oklch(96% 0.01 85), oklch(21% 0.025 250));
  --surface-container-high: light-dark(oklch(93.5% 0.012 85), oklch(24% 0.028 250));
  --surface-overlay: light-dark(oklch(100% 0 0), oklch(26% 0.03 250));

  /* Linhas e Hairlines Estruturais estilo Laravel Blueprint */
  --surface-border: light-dark(oklch(88% 0.01 85), oklch(27% 0.02 250));
  --surface-border-subtle: light-dark(oklch(92% 0.008 85), oklch(23% 0.015 250));
  --border-highlight: light-dark(oklch(1 0 0 / 0.8), oklch(1 0 0 / 0.08));

  /* Textura de Fundo Blueprint Dot Matrix */
  --bg-grid-dot: radial-gradient(color-mix(in oklch, var(--text-primary) 5.5%, transparent) 1px, transparent 1px);
  --bg-grid-size: 24px 24px;

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

- **Display & Títulos:** Sans-serif e system fonts modernas com contraste semântico bicolor no `h1` (`.title-primary` e `.title-muted` com `font-weight: 450`) e `text-wrap: balance`. Fontes serifadas (`Newsreader`, `Charter`, `Georgia`) ficam expressamente isoladas dentro do frame de leitura simulado (`.screen-mock`).
- **Interface & Navegação:** Sans-serif nítida com pesos `500` e `600`, calhas fluidas (`clamp(1.25rem, 4vw, 3rem)`) e foco visível `:focus-visible` de alto contraste.
- **Código & Terminais:** `JetBrains Mono`, `Fira Code`, `ui-monospace` com `font-variant-numeric: tabular-nums` e proteção `user-select: none` em marcadores de prompt (`❯`, `$`) e comentários (`#`).

---

## 4. Componentes Chave da Experiência

1. **Top Floating Pill Announcement (Google Antigravity & Dux Labs):**
   - Micro-componente com dot pulsante anunciando novas versões e recursos do motor gráfico.
2. **Terminal Shell Dux com Window Chrome:**
   - Chrome de janela com tab de contexto git/workspace, título e botão tátil "Copiar" com confirmação de 2 segundos via `aria-live="polite"`.
3. **Faixa de Métricas de Engenharia (Laravel Blueprint):**
   - Faixa de estatísticas estruturada com números tabulares de alto impacto.
4. **Seletor de Personas Técnicas (Google Antigravity):**
   - Switcher interativo de casos de uso (*DevOps*, *Autores*, *Leitores*) com WAI-ARIA integral.
5. **Blueprint Grid dos 3 Pilares Técnicos:**
   - Hairlines contínuas de `1px solid var(--surface-border-subtle)` com 3 eixos fundamentais: *Zero C++ Runtime*, *Validação EPUB 3.3 Integrada* e *Motor Duplo a 60 FPS*.
6. **Matriz Auditada das 7 Obras em EPUB 3.3:**
   - Cards detalhados de alta densidade informacional com badges de engine, contagem de capítulos, subtópicos e parágrafos.
7. **Simulador Interativo Multi-Página KOReader / Rust:**
   - 4 temas cromáticos (`Sepia`, `Paper White`, `Night Mode`, `E-Ink Mono`).
   - Seletor de motor gráfico (`native-rust` vs `koreader-crengine`).
   - Controles de zoom tipográfico dinâmico (80% a 140%).
   - Paginação interativa por 3 capítulos reais do *Rust Book* (*Ownership*, *Borrowing*, *Lifetimes*) com barra de progresso viva.
8. **Instalação com Unified Window Chrome & WAI-ARIA (Dux Labs & Laravel):**
   - Barra de abas fundida diretamente no topo da janela do terminal.
   - Padrão WAI-ARIA completo: `role="tablist"`, `role="tab"`, `aria-selected`, `aria-controls`, `role="tabpanel"`.
   - Suporte completo a navegação por setas (`ArrowLeft`/`ArrowRight`, `Home`/`End`) e *roving tabindex*.
   - Estabilidade dimensional sem layout shift via CSS Grid overlay (`grid-area: 1 / 1`) e atributo nativo `hidden`.
