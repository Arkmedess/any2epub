# Style Guide & Design Tokens — any2epub Web Showcase

Este documento estabelece as diretrizes de direção de arte, identidade visual e governança de design para a presença web do **any2epub** (`https://arkmedess.github.io/any2epub`).

---

## 1. Universo Visual & Personalidade

- **Arquétipo:** Editorial Técnico de Alta Performance (fusão de tipografia tipográfica editorial estilo *The Economist* / *O'Reilly* com a precisão minimalista de ferramentas modernas de infraestrutura como *Linear*, *Rust Lang* e *KOReader*).
- **Sensação Visual:** Confiança técnica, precisão tipográfica milimétrica, contraste nítido, sobriedade e velocidade instantânea.
- **Erradicação Ativa de Clichês de IA:** Zero gradientes roxo-para-ciano fluorescentes, zero cards "bento" ocos, zero textos vazios de marketing ("Desbloqueie seu potencial"). O valor é demonstrado através de artefatos reais (inspeção interativa de sumários, matriz comparativa de densidade e simulador ao vivo dos 4 temas de leitura).

---

## 2. Sistema de Cores & Tokens Semânticos (`oklch`)

Suporte nativo a Light & Dark Mode via `light-dark()` e variáveis `oklch`:

```css
:root {
  /* Cores Base e Superfícies */
  --bg-canvas: light-dark(oklch(98.5% 0.005 85), oklch(14% 0.015 250));
  --surface-card: light-dark(oklch(100% 0 0), oklch(18% 0.02 250));
  --surface-raised: light-dark(oklch(96% 0.01 85), oklch(22% 0.025 250));
  --surface-border: light-dark(oklch(88% 0.01 85), oklch(28% 0.02 250));

  /* Tipografia */
  --text-primary: light-dark(oklch(20% 0.02 250), oklch(95% 0.005 85));
  --text-secondary: light-dark(oklch(45% 0.02 250), oklch(75% 0.01 250));
  --text-muted: light-dark(oklch(60% 0.01 250), oklch(60% 0.01 250));

  /* Acentos de Marca (Laranja Ferrugem / Rust + E-Ink Teal) */
  --accent-rust: oklch(58% 0.20 42);
  --accent-teal: oklch(65% 0.14 195);
  --accent-eink: oklch(82% 0.03 95);

  /* Temas Cromáticos do Leitor */
  --reader-paper: #FAF8F5;
  --reader-sepia: #F4ECD8;
  --reader-night: #1A1A1A;
  --reader-eink: #E5E5E0;

  color-scheme: light dark;
}
```

---

## 3. Tipografia & Hierarquia Multifatorial

- **Display & Títulos:** `Newsreader`, `Charter`, `Iowan Old Style` ou `Georgia` serifada com tracking negativo (`-0.02em`) e `text-wrap: balance`.
- **Interface & Navegação:** `system-ui`, `-apple-system`, `BlinkMacSystemFont`, `Inter` com pesos `500` e `600`.
- **Código & Terminais:** `JetBrains Mono`, `Fira Code`, `ui-monospace`, `monospace` com `font-variant-numeric: tabular-nums`.

---

## 4. Componentes Chave da Experiência

1. **Terminal Interativo Hero ("Show, Don't Tell"):**
   - Demonstração visual do pipeline de conversão com comandos copiáveis em 1 clique (`cargo install`, `brew`, `docker`).
2. **Matriz Auditada dos 7 Livros em EPUB 3.3:**
   - Cards detalhados com contagem real de parágrafos, subtópicos até o 4º nível e tamanho de arquivo.
   - Botão direto para baixar os exemplos pré-compilados.
3. **Simulador de Leitura KOReader / Rust ao Vivo:**
   - Alternância interativa dos 4 temas cromáticos (`PaperWhite`, `Sepia`, `Night`, `EInkMono`) simulando a renderização a 60 FPS da biblioteca.
4. **Guia Rápido de Instalação e CLI:**
   - Tabs limpas com zero layout shift e seletor de plataforma (Linux, macOS, Windows, Docker).
