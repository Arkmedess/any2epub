# Plano de Orquestração e Arquitetura em Rust: Conversor Autoral PDF para EPUB 3.3

Este plano dá continuidade direta à arquitetura definida no chat compartilhado, migrando o protótipo ilustrativo em Python baseado em `pdfplumber` para um motor nativo de alta performance em **Rust** (`pdf2epub.rs`), projetado para converter livros de texto corrido com **centenas de páginas por PDF** sem qualquer dependência de ferramentas CLI externas (como Calibre ou Pandoc).

## 1. Justificativa Arquitetural: Por que Migrar de Python para Rust em PDFs de Centenas de Páginas?

No desenho original em Python, o `pdfplumber` (construído sobre `pdfminer.six`) instancia dicionários Python individuais para cada glifo e caixa delimitadora (*Bounding Box*), operando em *single-thread* sob o *Global Interpreter Lock* (GIL). Em romances, ensaios e livros técnicos de 400 a 800+ páginas, esse modelo gera alto consumo de memória RAM e latência na casa de dezenas de segundos por volume.

A arquitetura em Rust resolve esses gargalos em quatro frentes:

| Camada do Pipeline | Protótipo Original (Python) | Arquitetura Orquestrada (Rust) | Ganho Técnico em Livros de 500+ Páginas |
| :--- | :--- | :--- | :--- |
| **Extração Geométrica (BBoxes)** | `pdfplumber` (`extract_words`) | `pdfium-render` (ou alternativa pura `lopdf` / `mupdf`) | Extração direta em C++/Rust com structs contíguas (`PdfWordBox` de tamanho fixo na *stack*/*heap*), eliminando a sobrecarga de objetos dinâmicos por caractere |
| **Concorrência por Página** | Loop sequencial página a página | `rayon` (`par_iter` *work-stealing*) | Cálculo da moda tipográfica, filtragem de margens, agrupamento de linhas e renderização XHTML5 distribuídos em todos os núcleos da CPU |
| **Casamento de Padrões (Regex)** | Módulo `re` (backtracking) | `regex` (Autômato Finito Determinístico) | Complexidade garantida $O(m \times n)$ linear sem risco de *catastrophic backtracking* em parágrafos longos |
| **Empacotamento OCF/EPUB 3** | `zipfile` sequencial | `zip` (`ZipWriter` com `BufWriter` de 512 KB) | Escrita binária direta com controle estrito de `CompressionMethod::Stored` para o `mimetype` no byte offset 0 segundo a Especificação W3C EPUB 3.3 |
