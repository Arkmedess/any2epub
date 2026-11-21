# Roadmap de Engenharia — any2epub

Planejamento técnico, separação arquitetural entre **CLI** e **GUI** e marcos SemVer do projeto.

---

## Visão Executiva e Marcos SemVer

| Milestone | Foco Principal | Status | Referência |
| :--- | :--- | :--- | :--- |
| **v0.9.0** | Arquitetura modular em estágios e extração profunda Web/PDF | Concluído | [CHANGELOG.md](CHANGELOG.md#090---2026-10-24) |
| **v0.10.0** | Validador W3C EPUB 3.3, extrator Markdown local e base do viewer | Concluído | [CHANGELOG.md](CHANGELOG.md#0100---2026-10-30) |
| **v0.11.0** | CI/CD multiplataforma, Dockerfile rootless, streams POSIX e rasterizador 5x7 | Concluído | [CHANGELOG.md](CHANGELOG.md#0110---2026-11-07) |
| **[v0.12.0](#v0120--separação-cli--gui-metadados-dinâmicos-e-distribuição)** | Separação CLI headless vs. GUI desktop, versão/data dinâmicas e releases duais | Em Andamento | [Milestone 1](https://github.com/Arkmedess/any2epub/milestone/1) |
| **[v0.13.0](#v0130--tipografia-truetype--renderização-nativa-koreader)** | Fontes TrueType/OTF no motor Rust e renderização RGBA direta via `libcrengine` | Planejado | [Milestone 2](https://github.com/Arkmedess/any2epub/milestone/2) |
| **[v1.0.0](#v100--contratos-públicos-estáveis--golden-datasets)** | Selagem da API pública `any2epub_rs`, contratos CLI/JSON e suíte offline | Bloqueado por v0.13.0 | [Milestone 3](https://github.com/Arkmedess/any2epub/milestone/3) |

---

## Detalhamento das Milestones

### v0.12.0 — Separação CLI & GUI, Metadados Dinâmicos e Distribuição
**Objetivo:** Encerrar a ambiguidade entre CLI headless e leitor desktop GUI, tornando a organização de crates/binários, metadados e pacotes de release explícita.

- [ ] **#1** — `refactor(workspace): separar alvos e organização entre CLI headless e GUI desktop`
- [ ] **#2** — `fix(cli): usar versão dinâmica do Cargo e data dinâmica de publicação nos metadados`
- [ ] **#3** — `ci(release): publicar artefatos separados para CLI headless e GUI desktop e alinhar homepage`

**Critérios de Conclusão (DoD):**
1. Compilação padrão da CLI sem dependência de X11/Wayland/minifb e alvo GUI dedicado.
2. `any2epub --version` e `<dc:date>` derivados dinamicamente de `CARGO_PKG_VERSION` e data/flag `--date`.
3. Workflow de release publicando artefatos distintos para CLI e GUI com `SHA256SUMS`.

---

### v0.13.0 — Tipografia TrueType & Renderização Nativa KOReader
**Objetivo:** Elevar a qualidade tipográfica do visualizador desktop e concluir a rasterização nativa via FFI do `crengine`.

- [ ] **#4** — `feat(viewer): suportar fontes TrueType/OTF no NativeRustReaderEngine`
- [ ] **#5** — `feat(viewer): implementar renderização RGBA direta via libcrengine em KoreaderCrengineEngine`

**Critérios de Conclusão (DoD):**
1. Suporte completo a acentuação UTF-8 e pesos tipográficos sem fallback `?` no motor Rust.
2. `KoreaderCrengineEngine::render_page` renderizando via `libcrengine` quando presente no host e mantendo fallback seguro quando ausente.

---

### v1.0.0 — Contratos Públicos Estáveis & Golden Datasets
**Objetivo:** Estabilizar interfaces públicas da biblioteca e CLI com suíte determinística independente de rede externa.

- [ ] **#6** — `feat(core): estabilizar API pública de any2epub_rs, exit codes e schema JSON da CLI v1`
- [ ] **#7** — `test(pipeline): consolidar suíte determinística de Golden Datasets sem dependência de rede`

**Critérios de Conclusão (DoD):**
1. Contratos públicos de `src/lib.rs`, JSON de `validate --json` e exit codes (`0`, `1`, `2`) cobertos por testes e documentados.
2. Suíte `cargo test --features gui` 100% reproduzível em ambiente offline.
