//! Tabela de reparo de CMap CID (ligaduras Calibri e glifos monoespaçados Lekton).

/// Repara glifos CID de fontes embarcadas sem tabela `/ToUnicode` válida (Calibri e Lekton).
pub fn repair_broken_cmap_cid(font_name: &str, cid: u16) -> Option<&'static str> {
    let fname_lower = font_name.to_lowercase();
    if fname_lower.contains("calibri") {
        match cid {
            0x014C => Some("ft"),
            0x019F => Some("ti"),
            0x01A1 => Some("tí"),
            _ => None,
        }
    } else if fname_lower.contains("lekton") {
        match cid {
            0x01EB => Some("C"),
            0x01EE => Some("Q"),
            0x01EF => Some("V"),
            0x01F0 => Some("Z"),
            0x01F1 => Some("c"),
            0x01F2 => Some("f"),
            0x01F3 => Some("g"),
            0x01F4 => Some("i"),
            0x01F5 => Some("j"),
            0x01F6 => Some("k"),
            0x01F7 => Some("l"),
            0x01F8 => Some("r"),
            0x01F9 => Some("t"),
            0x01FA => Some("v"),
            0x01FB => Some("y"),
            0x0205 => Some("="),
            _ => None,
        }
    } else {
        None
    }
}
