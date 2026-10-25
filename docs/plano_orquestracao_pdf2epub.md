# Plano de Orquestração e Arquitetura em Rust: Conversor Autoral PDF para EPUB 3.3

Este plano documenta a arquitetura em **Rust** (`any2epub-rs`), projetada para converter livros de texto corrido com **centenas de páginas por PDF** e documentações técnicas na Web sem qualquer dependência de ferramentas CLI externas (como Calibre ou Pandoc).

## 1. Justificativa Arquitetural: Por que Migrar de Python para Rust em PDFs de Centenas de Páginas?

No desenho original em Python, o `pdfplumber` (construído sobre `pdfminer.six`) instancia dicionários Python individuais para cada glifo e caixa delimitadora (*Bounding Box*), operando em *single-thread* sob o *Global Interpreter Lock* (GIL). Em romances, ensaios e livros técnicos de 400 a 800+ páginas, esse modelo gera alto consumo de memória RAM e latência na casa de dezenas de segundos por volume.

A arquitetura em Rust resolve esses gargalos em quatro frentes:

| Camada do Pipeline | Protótipo Original (Python) | Arquitetura Orquestrada (Rust) | Ganho Técnico em Livros de 500+ Páginas |
| :--- | :--- | :--- | :--- |
| **Extração Geométrica (BBoxes)** | `pdfplumber` (`extract_words`) | `pdfium-render` / `lopdf` | Extração direta em C++/Rust com structs contíguas (`PdfWordBox` de tamanho fixo na *stack*/*heap*), eliminando a sobrecarga de objetos dinâmicos por caractere |
| **Concorrência por Página** | Loop sequencial página a página | `rayon` (`par_iter` *work-stealing*) | Cálculo da moda tipográfica, filtragem de margens, agrupamento de linhas e renderização XHTML5 distribuídos em todos os núcleos da CPU |
| **Casamento de Padrões (Regex)** | Módulo `re` (backtracking) | `regex` (Autômato Finito Determinístico) | Complexidade garantida $O(m \times n)$ linear sem risco de *catastrophic backtracking* em parágrafos longos |
| **Empacotamento OCF/EPUB 3** | `zipfile` sequencial | `zip` (`ZipWriter` com `BufWriter` de 512 KB) | Escrita binária direta com controle estrito de `CompressionMethod::Stored` para o `mimetype` no byte offset 0 segundo a Especificação W3C EPUB 3.3 |

## 2. Orquestração dos 4 Estágios + 3 Incrementos Tipográficos

O plano de execução foi estruturado em **duas fases de processamento** (Fase Paralela de Página e Fase Sequencial de Costura Contínua) para combinar o máximo paralelismo de CPU com a continuidade semântica entre quebras de página:

### Estágio 1 — Leitura Anatômica e Moda Estatística Paralela (`PdfProsePipeline`)
1. **Amostragem Paralela da Fonte Base:** Utiliza `rayon` (`par_iter().fold().reduce()`) sobre as páginas iniciais para construir um histograma de tamanhos de fonte (em intervalos de `0.1pt`) ponderado pela contagem de caracteres de cada token, determinando com precisão a **moda do corpo de texto** (`body_font_size`).
2. **Normalização do Eixo Coordenado:** Converte o sistema de coordenadas nativo do PDF (origem no canto inferior esquerdo via `pdfium-render`) para coordenadas de leitura *top-down* (`top = height - bounds.top()`), preservando `x0`, `x1`, `font_size`, `is_italic` e `is_bold`.
3. **Expurgo Geométrico de Cabeçalhos e Rodapés:** Filtra em paralelo qualquer elemento situado acima de `7%` (`top < height * 0.07`) ou abaixo de `93%` (`bottom > height * 0.93`) da altura da página, eliminando títulos correntes (*running headers*) e numeração de página.

### Estágio 2 — Orquestração dos 3 Incrementos Avançados
Os três incrementos tipográficos foram incorporados diretamente ao motor de reconstrução de linhas e parágrafos em `src/extractors/pdf/pipeline.rs`:

1. **Incremento A — Capitulares (*Drop Caps*):**
   - **Problema no PDF:** Letras capitulares no início de capítulos possuem fonte `>= 1.8x` a moda e ocupam uma caixa isolada à esquerda das 2 ou 3 primeiras linhas. Sem tratamento específico, um extrator comum interpreta a letra isolada como um título de capítulo falso (`max_size > 1.35 * moda`) ou um parágrafo de uma única letra.
   - **Solução Orquestrada:** Em `group_words_into_lines`, se um cluster contém apenas 1 caractere com `font_size >= 1.80 * body_font_size` (ou se o primeiro token da linha atende a esse critério e o token seguinte retorna ao tamanho normal), a letra é capturada em `pending_dropcap`, colada sem espaço caso pertença à primeira palavra (verificando o gap horizontal `x0 - x1 <= 0.45 * body_font_size`) e serializada em XHTML5 como `<p class="dropcap-para"><span class="dropcap">L</span>orem...</p>`.
2. **Incremento B — Diálogos com Travessão (`—` / `–`):**
   - **Problema no PDF:** Na literatura lusófona, falas introduzidas por travessão (`—` `\u{2014}` ou meia-risca `–` `\u{2013}`) frequentemente terminam sem ponto final ou são sucedidas por outra fala curta.
   - **Solução Orquestrada:** Quando uma linha reconstruída inicia com `—`, `–` ou `- `, o motor de estado executa imediatamente `flush_paragraph_buffer` para encerrar o parágrafo anterior e marca o novo buffer como diálogo, gerando `<p class="dialogue">` independente.
3. **Incremento C — Notas de Rodapé Bidirecionais (`<aside epub:type="footnote">`):**
   - **Problema no PDF:** Notas de rodapé impressas na base da página interrompem um parágrafo que continua na página seguinte.
   - **Solução Orquestrada:** Durante a fase paralela por página, linhas situadas na zona inferior (`top >= 0.75 * height`) com `max_font_size <= 0.85 * body_font_size` que casam com marcadores numéricos ou sobrescritos são separadas do fluxo de prosa principal (`page_footnotes`). Durante a costura do capítulo, referências no texto são convertidas via `regex` para `<a epub:type="noteref" id="ref-fn-N" href="#fn-N"><sup>[N]</sup></a>` e os blocos correspondentes são anexados ao final do arquivo do capítulo dentro de `<aside epub:type="footnote" id="fn-N">` com link de retorno `↩`, seguindo a semântica da W3C EPUB 3.3.

### Estágio 3 — Costura Entre Páginas (*Cross-Page Stitching*), Des-hifenização *Case-Aware* e Sanitização XHTML5
1. **Continuidade Entre Quebras de Página:** O `buffer_text` permanece aberto na transição da página $N$ para $N+1$ sempre que a última linha útil da página $N$ não termina com pontuação terminal (`.`, `!`, `?`, `:`, `;`, `"`, `”`, `»`, `…`, `)`). Assim, frases divididas entre duas páginas são fundidas no mesmo `<p>`.
2. **Des-hifenização Consciente de Caixa (*Case-Aware*):** A função `stitch_line_into_buffer` verifica se a linha anterior termina em hífen (`-` ou *soft hyphen* `\u{00AD}`) sem espaço precedente e inspeciona o primeiro caractere alfabético da linha seguinte (saltando tags inline como `<em>` ou `<strong>` via `strip_leading_tags`). Se for minúsculo (`char::is_lowercase()`), remove o hífen e une as sílabas; se for maiúsculo (ex: nomes compostos como `Grã-Bretanha`) ou travessão espaçado, preserva o sinal.
3. **Sanitização XML/XHTML5 em Passagem Única:** A função `escape_xhtml_fast` escapa `&`, `<`, `>`, `"` e `'` antes da aplicação das tags tipográficas (`<em>`, `<strong>`), impedindo que um único caractere `&` corrompa a renderização XML nos motores estritos de e-ink (Kindle, Kobo, Apple Books).

### Estágio 4 — Construção Nativa do Container OCF / EPUB 3.3 (`NativeEpub3Builder`)
1. **Renderização Paralela de Capítulos:** Todos os capítulos extraídos (`Vec<Chapter>`) são convertidos para strings XHTML5 completas em paralelo via `self.chapters.par_iter().map(...)` (`rayon`).
2. **Empacotamento ZIP em Memória:** Utiliza `zip` (`ZipWriter`) sobre um `BufWriter` de 512 KB:
   - Grava `mimetype` (`application/epub+zip`) como o primeiro arquivo do pacote com `CompressionMethod::Stored` (sem compressão).
   - Grava `META-INF/container.xml`, `OEBPS/style.css`, `OEBPS/nav.xhtml`, `OEBPS/content.opf` (com identificador único gerado via `uuid` v4) e os arquivos `OEBPS/chapter_XXX.xhtml` com `CompressionMethod::Deflated`.
