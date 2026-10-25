//! Funções utilitárias de sanitização XML/XHTML5, normalização de espaços e geração de âncoras.

/// Converte um título de subtópico (ex: `"Variables and Data Interacting with Move"`)
/// em um identificador de âncora HTML válido estilo mdBook/MediaWiki
/// (ex: `"variables-and-data-interacting-with-move"`).
pub fn slugify_anchor_id(raw_title: &str, fallback_idx: usize) -> String {
    let mut slug = String::with_capacity(raw_title.len());
    let mut last_was_dash = false;

    for ch in raw_title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if (ch.is_whitespace() || ch == '-' || ch == '_' || ch == '/')
            && !slug.is_empty()
            && !last_was_dash
        {
            slug.push('-');
            last_was_dash = true;
        }
    }

    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        format!("sec-{}", fallback_idx)
    } else if trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("s-{}", trimmed)
    } else {
        trimmed
    }
}

/// Escapa caracteres especiais XML/XHTML5 (`&`, `<`, `>`, `"`, `'`) em passagem única.
pub fn escape_xhtml_fast(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 16);
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Colapsa sequências contíguas de espaços em branco e quebras de linha em um único espaço ASCII.
pub fn normalize_whitespace(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_was_space = false;
    for ch in input.trim().chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
        } else {
            out.push(ch);
            last_was_space = false;
        }
    }
    out
}

/// Remove tags HTML/XHTML de abertura no início de uma linha para inspeção de caixa tipográfica.
pub fn strip_leading_tags(mut input: &str) -> &str {
    input = input.trim_start();
    while input.starts_with('<') {
        if let Some(end_idx) = input.find('>') {
            input = input[end_idx + 1..].trim_start();
        } else {
            break;
        }
    }
    input
}
