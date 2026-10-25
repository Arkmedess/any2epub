//! Expressões regulares compiladas em autômato finito determinístico (DFA linear sem backtracking).

use std::sync::OnceLock;

use regex::Regex;

/// Expressão regular para identificação de cabeçalhos de capítulos e divisões estruturais em PDFs.
pub fn chapter_heading_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^(?i)(CAP[ÍI]TULO\s+[0-9IVXLCDM]+|PARTE\s+[0-9IVXLCDM]+|PR[ÓO]LOGO|PREF[ÁA]CIO|EP[ÍI]LOGO|AP[ÊE]NDICE)",
        )
        .expect("Regex de capítulo inválida")
    })
}

/// Expressão regular para identificação de legendas de figuras, quadros e tabelas.
pub fn caption_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(Figura|Figure|Quadro|Tabela|Exemplo|Equa[çc][ãa]o)\s+\d+")
            .expect("Regex de legenda inválida")
    })
}

/// Expressão regular para detecção de pontuação terminal ao final de uma linha reconstruída.
pub fn terminal_punctuation_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"[\.\!\?\:\;"”»\u{2026}\)]$|\.\.\.$"#)
            .expect("Regex de pontuação terminal inválida")
    })
}

/// Expressão regular para captura de notas de rodapé na zona inferior da página.
pub fn footnote_line_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(\[?\d{1,3}\]?|[¹²³⁴⁵⁶⁷⁸⁹⁰]+)\s+(.+)$")
            .expect("Regex de nota de rodapé inválida")
    })
}

/// Expressão regular para vinculação bidirecional de referências de notas de rodapé no corpo do texto.
pub fn inline_footnote_ref_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"([a-zA-ZÀ-ÿ,\.])(\[\d{1,2}\]|\d{1,2})(\s|$)")
            .expect("Regex de referência inline de nota inválida")
    })
}
