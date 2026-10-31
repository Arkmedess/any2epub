//! Opções editoriais de renderização, geometria e temas cromáticos de leitura.

/// Temas cromáticos de fundo e tipografia inspirados na experiência do KOReader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerColorTheme {
    /// Papel Branco Puro (#FFFFFF) com tipografia grafite escura (#111111).
    PaperWhite,
    /// Papel Sépia Aconchegante (#F4ECD8) com tipografia café (#3C2E1F).
    Sepia,
    /// Modo Noturno / Dark (#181818) com tipografia cinza suave (#D0D0D0).
    Night,
    /// Monocromático de Alto Contraste (#FFFFFF / #000000) otimizado para e-ink.
    EInkMono,
}

impl ViewerColorTheme {
    /// Retorna a cor de fundo em formato RGBA.
    pub fn background_rgba(self) -> [u8; 4] {
        match self {
            Self::PaperWhite => [255, 255, 255, 255],
            Self::Sepia => [244, 236, 216, 255],
            Self::Night => [24, 24, 24, 255],
            Self::EInkMono => [255, 255, 255, 255],
        }
    }

    /// Retorna a cor do texto principal em formato RGBA.
    pub fn text_rgba(self) -> [u8; 4] {
        match self {
            Self::PaperWhite => [17, 17, 17, 255],
            Self::Sepia => [60, 46, 31, 255],
            Self::Night => [208, 208, 208, 255],
            Self::EInkMono => [0, 0, 0, 255],
        }
    }
}

/// Opções editoriais e tipográficas aplicáveis ao motor de paginação e renderização.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewerRenderOptions {
    /// Tamanho base da fonte do corpo em pontos tipográficos (ex: 11.0, 12.0, 14.0).
    pub font_size_pt: f32,
    /// Multiplicador de entrelinha (*line-height*, ex: 1.35, 1.5).
    pub line_height_multiplier: f32,
    /// Margem horizontal e vertical da página em pixels.
    pub margin_px: u32,
    /// Habilita hifenização tipográfica TeX baseada em dicionário.
    pub hyphenation_enabled: bool,
    /// Tema cromático ativo.
    pub theme: ViewerColorTheme,
}

impl Default for ViewerRenderOptions {
    fn default() -> Self {
        Self {
            font_size_pt: 12.0,
            line_height_multiplier: 1.4,
            margin_px: 36,
            hyphenation_enabled: true,
            theme: ViewerColorTheme::PaperWhite,
        }
    }
}
