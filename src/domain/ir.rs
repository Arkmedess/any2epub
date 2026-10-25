//! Representação Intermediária (IR) unificada de capítulos, subtópicos e elementos semânticos.

use crate::domain::metadata::{BookMetadata, ExtractedFigure};

/// Nó recursivo de Sumário Hierárquico (suporta `4 -> 4.1 -> 4.1.1 -> 4.1.5.1` em `OEBPS/nav.xhtml`).
#[derive(Debug, Clone)]
pub struct TocNode {
    /// Prefixo numérico hierárquico (ex: `"4.1.1"`, `"4.1.5.1"`, `"1.1.1.1"`).
    pub number_prefix: String,
    /// Título limpo do subtópico (ex: `"The Stack and the Heap"`).
    pub title: String,
    /// Destino relativo com âncora (ex: `"chapter_008.xhtml#the-stack-and-the-heap"`).
    pub href: String,
    /// Subtópicos filhos aninhados.
    pub children: Vec<TocNode>,
}

/// Representação Intermediária unificada produzida por qualquer extrator de fonte.
#[derive(Debug, Clone)]
pub struct ExtractedBookPayload {
    /// Metadados editoriais e capa da obra.
    pub metadata: BookMetadata,
    /// Capítulos estruturados em ordem de leitura (spine).
    pub chapters: Vec<Chapter>,
    /// Figuras e diagramas extraídos das páginas ou documentos.
    pub figures: Vec<ExtractedFigure>,
}

/// Representação geométrica e tipográfica de um token extraído do PDF.
#[derive(Debug, Clone)]
pub struct PdfWordBox {
    /// Conteúdo textual bruto do token.
    pub text: String,
    /// Nome da fonte tipográfica associada no PDF.
    pub font_name: String,
    /// Coordenada horizontal esquerda (em pontos tipográficos).
    pub x0: f32,
    /// Coordenada vertical superior normalizada (top-down).
    pub top: f32,
    /// Coordenada horizontal direita (em pontos tipográficos).
    pub x1: f32,
    /// Coordenada vertical inferior normalizada (top-down).
    pub bottom: f32,
    /// Tamanho nominal da fonte não escalado.
    pub font_size: f32,
    /// Indica se o glifo possui estilo itálico ou oblíquo.
    pub is_italic: bool,
    /// Indica se o glifo possui peso negrito ou seminegrito.
    pub is_bold: bool,
    /// Indica se a fonte é monoespaçada (código).
    pub is_monospace: bool,
}

/// Página bruta extraída na camada geométrica de um documento PDF.
#[derive(Debug, Clone)]
pub struct RawPdfPage {
    /// Índice baseado em zero da página no documento original.
    pub page_index: usize,
    /// Largura total da página em pontos.
    pub width: f32,
    /// Altura total da página em pontos.
    pub height: f32,
    /// Palavras e caixas delimitadoras extraídas da página.
    pub words: Vec<PdfWordBox>,
    /// Figuras extraídas desta página específica.
    pub figures: Vec<ExtractedFigure>,
}

/// Linha reconstruída a partir do agrupamento vertical (eixo Y) de palavras.
#[derive(Debug, Clone)]
pub struct ReconstructedLine {
    /// Texto plano sem marcação HTML.
    pub raw_text: String,
    /// Texto enriquecido com tags inline (`<em>`, `<strong>`, `<code>`).
    pub html_text: String,
    /// Recuo horizontal esquerdo da linha.
    pub x0: f32,
    /// Coordenada vertical superior da linha.
    pub top: f32,
    /// Maior tamanho de fonte observado entre os tokens da linha.
    pub max_font_size: f32,
    /// Indica se a linha inicia com letra capitular (*Drop Cap*).
    pub starts_with_dropcap: bool,
    /// Indica se a linha utiliza família tipográfica de título.
    pub is_heading_font: bool,
    /// Indica se todos os tokens da linha pertencem a fonte monoespaçada.
    pub is_all_monospace: bool,
}

/// Nota de rodapé identificada na base da página.
#[derive(Debug, Clone)]
pub struct FootnoteEntry {
    /// Número sequencial único da nota dentro do capítulo.
    pub id_num: usize,
    /// Marcador textual original (ex: `"1"`, `"*"`).
    pub marker: String,
    /// Conteúdo da nota sanitizado em XHTML5.
    pub content_html: String,
}

/// Tipo de caixa de destaque semântica (`Note`, `Tip`, `Warning` — padrão ArchWiki/Docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalloutKind {
    /// Caixa informativa padrão.
    Note,
    /// Caixa de dica prática ou recomendação.
    Tip,
    /// Caixa de alerta ou aviso importante.
    Warning,
}

/// Elemento semântico de capítulo classificado pelo motor de estado (com suporte até 4º nível H4 e âncoras ID).
#[derive(Debug, Clone)]
pub enum ChapterElement {
    /// Subtópico de 1º nível interno ao capítulo (ex: `4.1.1 The Stack and the Heap`).
    SubheadingH2 {
        /// Identificador de âncora HTML.
        id: String,
        /// Prefixo numérico hierárquico.
        number_prefix: String,
        /// Título sanitizado do subtópico.
        title: String,
    },
    /// Subtópico de 2º nível interno ao capítulo (ex: `4.1.5.1 Variables and Data Interacting with Move`).
    SubheadingH3 {
        /// Identificador de âncora HTML.
        id: String,
        /// Prefixo numérico hierárquico.
        number_prefix: String,
        /// Título sanitizado do subtópico.
        title: String,
    },
    /// Subtópico de 3º nível interno ao capítulo (ex: `1.1.1.1 Virtual packages` na ArchWiki).
    SubheadingH4 {
        /// Identificador de âncora HTML.
        id: String,
        /// Prefixo numérico hierárquico.
        number_prefix: String,
        /// Título sanitizado do subtópico.
        title: String,
    },
    /// Parágrafo de prosa ou item de lista/definição padrão.
    StandardParagraph(String),
    /// Parágrafo de diálogo literário introduzido por travessão.
    DialogueParagraph(String),
    /// Parágrafo inicial de capítulo com letra capitular destacada.
    DropCapParagraph(char, String),
    /// Bloco de código-fonte pré-formatado (`<pre><code>`).
    CodeBlock(String),
    /// Caixa de aviso ou nota lateral (`<aside class="callout">`).
    CalloutBox {
        /// Categoria visual e semântica da caixa.
        kind: CalloutKind,
        /// Título da caixa de destaque.
        title: String,
        /// Corpo em XHTML5 sanitizado.
        body_html: String,
    },
    /// Tabela de dados estruturada (`<table class="wikitable">`).
    DataTable {
        /// Células do cabeçalho da tabela (`<th>`).
        headers: Vec<String>,
        /// Linhas de dados da tabela (`<td>`).
        rows: Vec<Vec<String>>,
    },
    /// Referência a imagem de figura incorporada no pacote.
    FigureImage(String),
    /// Legenda descritiva associada a uma figura ou tabela.
    FigureCaption(String),
}

/// Capítulo estruturado pronto para serialização XHTML5 com árvore recursiva de subtópicos.
#[derive(Debug, Clone)]
pub struct Chapter {
    /// Número de ordem do capítulo (baseado em 1).
    pub index: usize,
    /// Prefixo numérico exibido no sumário e cabeçalhos.
    pub number_prefix: String,
    /// Título completo do capítulo.
    pub title: String,
    /// Nome do arquivo XHTML correspondente em `OEBPS/` (ex: `"chapter_001.xhtml"`).
    pub filename: String,
    /// Sequência ordenada de elementos semânticos que compõem o capítulo.
    pub elements: Vec<ChapterElement>,
    /// Notas de rodapé vinculadas ao capítulo.
    pub footnotes: Vec<FootnoteEntry>,
    /// Árvore hierárquica de subtópicos (`H2 -> H3 -> H4`) para o `nav.xhtml`.
    pub subtopics: Vec<TocNode>,
}

/// Insere um subtópico na árvore hierárquica `Vec<TocNode>` respeitando a profundidade (`H2 -> H3 -> H4`).
pub fn insert_subtopic_node(
    tree: &mut Vec<TocNode>,
    level: u8,
    number_prefix: String,
    title: String,
    href: String,
) {
    let new_node = TocNode {
        number_prefix,
        title,
        href,
        children: Vec::new(),
    };

    match level {
        2 => tree.push(new_node),
        3 => {
            if let Some(last_h2) = tree.last_mut() {
                last_h2.children.push(new_node);
            } else {
                tree.push(new_node);
            }
        }
        _ => {
            if let Some(last_h2) = tree.last_mut() {
                if let Some(last_h3) = last_h2.children.last_mut() {
                    last_h3.children.push(new_node);
                } else {
                    last_h2.children.push(new_node);
                }
            } else {
                tree.push(new_node);
            }
        }
    }
}
