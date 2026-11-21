# Roadmap — any2epub

Plano de evolução do projeto com foco em **conversão de alta qualidade para Kindle desbloqueado (KOReader)**, **CLI rica em informações**, **leitor desktop estilo KOReader** e **integração direta com o aparelho**.

> **Como funcionam as versões:** As etapas abaixo agrupam objetivos do produto, sem prender releases a datas ou números fixos. Correções simples geram versões `0.11.x`; recursos novos geram versões `0.12.0+`; a `v1.0.0` só sai quando os formatos, flags e APIs estiverem maduros e estáveis.

---

## Visão Geral das Etapas

| Etapa | Foco Principal | Status | GitHub |
| :--- | :--- | :--- | :--- |
| **v0.9.0 – v0.11.0** | Base do projeto, extração de PDF/Web/Markdown, validador EPUB e início do visualizador | Concluído | [CHANGELOG.md](CHANGELOG.md) |
| **[Etapa 1 — Separação CLI/GUI, Diagnóstico & Metadados](#etapa-1--separação-cligui-diagnóstico--metadados)** | Separar núcleo, CLI e GUI em pacotes próprios, criar comando de inspeção de EPUBs e corrigir metadados fixos | Em Andamento | [Milestone 1](https://github.com/Arkmedess/any2epub/milestone/1) |
| **[Etapa 2 — Conversão Avançada & Otimização Kindle/KOReader](#etapa-2--conversão-avançada--otimização-kindlekoreader)** | PDFs complexos (duas colunas e tabelas), novos formatos (DOCX, HTML, `.ipynb`, CBZ) e imagens/CSS para tela e-ink | Planejado | [Milestone 2](https://github.com/Arkmedess/any2epub/milestone/2) |
| **[Etapa 3 — Leitor Estilo KOReader & Plugin para Kindle](#etapa-3--leitor-estilo-koreader--plugin-para-kindle)** | Fontes TrueType no leitor desktop, desenho nativo via `libcrengine`, plugin Lua e envio via Wi-Fi | Planejado | [Milestone 3](https://github.com/Arkmedess/any2epub/milestone/3) |
| **[Etapa 4 — Maturidade `v1.0.0`](#etapa-4--maturidade-v100)** | Estabilidade das APIs/flags, testes offline completos e checagem automática com o EPUBCheck oficial | Longo Prazo | [Milestone 4](https://github.com/Arkmedess/any2epub/milestone/4) |

---

## O Que Será Feito em Cada Etapa

### Etapa 1 — Separação CLI/GUI, Diagnóstico & Metadados
- **#1** — Separar o projeto em três pacotes dentro do mesmo repositório: núcleo compartilhado, CLI de terminal e leitor GUI desktop
- **#2** — Corrigir versão dinâmica (`--version`), datas reais no `content.opf` e tag de capa apenas quando o livro tiver capa
- **#8** — Criar o comando `any2epub info` para mostrar resumo completo do EPUB (capítulos, palavras, tempo de leitura, imagens, notas e avisos para KOReader)
- **#3** — Publicar pacotes separados para CLI e GUI nas Releases e corrigir link da homepage no `Cargo.toml`

### Etapa 2 — Conversão Avançada & Otimização Kindle/KOReader
- **#9** — Melhorar conversão de PDFs difíceis: ordem certa em páginas de duas ou mais colunas, extração de tabelas e reparo ampliado de fontes sem mapa de caracteres
- **#10** — Suportar novos formatos de entrada: Word (`.docx`), páginas HTML locais, Jupyter Notebooks (`.ipynb`) e quadrinhos/mangás (`.cbz`)
- **#11** — Criar perfil de saída para Kindle/KOReader: redimensionamento e tons de cinza para tela e-ink (300 DPI) e CSS que não brigue com o modo noturno do aparelho

### Etapa 3 — Leitor Estilo KOReader & Plugin para Kindle
- **#4** — Adicionar fontes TrueType/OTF no leitor em Rust para leitura confortável com acentuação completa
- **#5** — Conectar o desenho de página diretamente à `libcrengine` quando a biblioteca do KOReader estiver instalada no computador
- **#12** — Criar plugin oficial em Lua (`any2epub.koplugin`) e modo servidor local (`any2epub serve`) para converter e baixar livros direto no Kindle via Wi-Fi

### Etapa 4 — Maturidade `v1.0.0`
- **#6** — Estabilizar as funções públicas da biblioteca, os códigos de erro do terminal e o formato JSON da CLI
- **#7** — Montar suíte de testes offline com livros de exemplo variados e validação cruzada usando o EPUBCheck oficial no CI
