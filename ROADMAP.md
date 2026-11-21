# Roadmap de Engenharia — any2epub

Planejamento técnico orientado a **conversão EPUB 3.3 para Kindle desbloqueado (KOReader)**, **CLI rica em diagnóstico**, **previewer desktop estilo KOReader** e **ecossistema `.koplugin`**.

> **Política de Versionamento (SemVer):** As milestones abaixo organizam **épicos de capacidade**, sem travar releases intermediárias. Correções retrocompatíveis (`fix:`) disparam versões **PATCH** (`0.11.x`) a qualquer momento; novas capacidades ou alterações pré-1.0 (`feat:`, `!`) disparam versões **MINOR** (`0.12.0+`); a **`v1.0.0`** marca o congelamento formal dos contratos públicos.

---

## Visão Executiva e Marcos

| Milestone | Foco Principal | Status | Referência |
| :--- | :--- | :--- | :--- |
| **v0.9.0 – v0.11.0** | Pipeline modular, extração Web/PDF/Markdown, validador W3C e base do viewer | Concluído | [CHANGELOG.md](CHANGELOG.md) |
| **[M1: Workspace CLI/GUI, Diagnóstico & Metadados](#m1-workspace-cligui-diagnóstico--metadados)** | Cargo Workspace (`core`, `cli`, `gui`), `any2epub info`, metadados OPF dinâmicos e releases duais | Em Andamento | [Milestone 1](https://github.com/Arkmedess/any2epub/milestone/1) |
| **[M2: Motores Complexos & Perfis Kindle/E-Ink](#m2-motores-complexos--perfis-kindlee-ink)** | PDF multi-coluna/tabelas/CMap, novos formatos (DOCX, HTML, `.ipynb`, CBZ) e perfil e-ink 300 DPI | Planejado | [Milestone 2](https://github.com/Arkmedess/any2epub/milestone/2) |
| **[M3: Ecossistema KOReader & Plugin (`.koplugin`)](#m3-ecossistema-koreader--plugin-koplugin)** | Fontes TrueType/OTF, renderização `libcrengine`, plugin Lua e servidor companion Wi-Fi | Planejado | [Milestone 3](https://github.com/Arkmedess/any2epub/milestone/3) |
| **[M4: `v1.0.0` — Maturidade, Contratos & EPUBCheck](#m4-v100--maturidade-contratos--epubcheck)** | Congelamento de API pública `any2epub-core`, schema CLI v1, Golden Datasets e EPUBCheck no CI | Longo Prazo | [Milestone 4](https://github.com/Arkmedess/any2epub/milestone/4) |

---

## Detalhamento das Milestones

### M1: Workspace CLI/GUI, Diagnóstico & Metadados
**Objetivo:** Separar fisicamente núcleo, CLI headless e leitor desktop em Cargo Workspace, enriquecer a CLI com auditoria detalhada de EPUBs e corrigir metadados OPF.

- [ ] **#1** — `refactor(workspace): migrar projeto para Cargo Workspace (core, cli, gui)`
- [ ] **#2** — `fix(cli,epub): corrigir versão dinâmica, datas OPF e meta cover condicional`
- [ ] **#8** — `feat(cli): adicionar subcomando any2epub info para auditoria de EPUBs (Kindle/KOReader)`
- [ ] **#3** — `ci(release): publicar artefatos separados para CLI headless e GUI desktop e alinhar homepage`

### M2: Motores Complexos & Perfis Kindle/E-Ink
**Objetivo:** Garantir eficácia em documentos difíceis (PDFs acadêmicos multi-coluna, tabelas, fontes CID variadas), ampliar formatos de entrada e otimizar saída para hardware Kindle/KOReader.

- [ ] **#9** — `feat(pdf): motor geométrico avançado para multi-coluna, tabelas e CMap generalizado`
- [ ] **#10** — `feat(extractors): ampliar formatos de entrada (DOCX, HTML local, Jupyter .ipynb e CBZ)`
- [ ] **#11** — `feat(epub): perfil Kindle/KOReader com otimização e-ink de imagens e CSS adaptativo`

### M3: Ecossistema KOReader & Plugin (`.koplugin`)
**Objetivo:** Transformar `any2epub-gui` em previewer fiel ao KOReader e conectar diretamente o Kindle desbloqueado via plugin Lua + servidor companion na rede local.

- [ ] **#4** — `feat(viewer): suportar fontes TrueType/OTF no NativeRustReaderEngine`
- [ ] **#5** — `feat(viewer): implementar renderização RGBA direta via libcrengine em KoreaderCrengineEngine`
- [ ] **#12** — `feat(koreader): criar plugin any2epub.koplugin (Lua) e modo companion/Wi-Fi para Kindle desbloqueado`

### M4: `v1.0.0` — Maturidade, Contratos & EPUBCheck
**Objetivo:** Alcançar maturidade de produção com estabilidade SemVer garantida e verificação cruzada contra o validador oficial W3C/EPUBCheck.

- [ ] **#6** — `feat(core): estabilizar API pública de any2epub_rs, exit codes e schema JSON da CLI v1`
- [ ] **#7** — `test(pipeline): consolidar suíte determinística de Golden Datasets sem dependência de rede`
