//! Janela gráfica desktop nativa do visualizador estilo KOReader (`run_gui_viewer`).

use std::path::Path;

use crate::viewer::engine::{ViewerBackendKind, ViewerError};
use crate::viewer::options::ViewerColorTheme;

#[cfg(feature = "gui")]
use crate::viewer::engine::create_reader_engine;
#[cfg(feature = "gui")]
use crate::viewer::options::ViewerRenderOptions;
#[cfg(feature = "gui")]
use minifb::{Key, Window, WindowOptions};

/// Abre a janela desktop nativa com viewport interativo estilo KOReader.
#[cfg(feature = "gui")]
pub fn run_gui_viewer(
    path: &Path,
    mut current_backend: ViewerBackendKind,
    mut current_theme: ViewerColorTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    let width = 800;
    let height = 1050;

    let mut engine = create_reader_engine(current_backend)?;
    let details = engine.open_book(path)?;

    let mut options = ViewerRenderOptions {
        theme: current_theme,
        font_size_pt: 12.0,
        line_height_multiplier: 1.45,
        margin_px: 40,
        hyphenation_enabled: true,
    };
    engine.set_viewport(width as u32, height as u32)?;
    engine.apply_render_options(&options)?;

    let mut current_page = 0;
    let mut total_pages = engine.total_pages().max(1);

    let window_title = format!(
        "any2epub Reader [{} | {:?}] — {}",
        engine.engine_name(),
        current_theme,
        details.title
    );

    let window_opts = WindowOptions {
        resize: false,
        scale: minifb::Scale::X1,
        ..WindowOptions::default()
    };

    let mut window = Window::new(&window_title, width, height, window_opts)
        .map_err(|e| ViewerError::BackendUnavailable(format!("Falha ao abrir janela gráfica: {}", e)))?;

    // Limita a taxa de atualização a 60 FPS
    window.set_target_fps(60);

    let mut needs_redraw = true;
    let mut pixel_buffer: Vec<u32> = vec![0; width * height];

    while window.is_open() && !window.is_key_down(Key::Escape) && !window.is_key_down(Key::Q) {
        if needs_redraw {
            total_pages = engine.total_pages().max(1);
            if current_page >= total_pages {
                current_page = total_pages.saturating_sub(1);
            }

            let bitmap = engine.render_page(current_page)?;
            pixel_buffer = bitmap.to_argb_u32();

            let title = format!(
                "any2epub Reader [{} | {:?}] — Pág. {} de {} ({:.1}%) — {}",
                engine.engine_name(),
                current_theme,
                current_page + 1,
                total_pages,
                ((current_page + 1) as f32 / total_pages as f32) * 100.0,
                details.title
            );
            window.set_title(&title);
            needs_redraw = false;
        }

        // Navegação de páginas (setas, espaço, page down/up)
        if (window.is_key_pressed(Key::Right, minifb::KeyRepeat::No)
            || window.is_key_pressed(Key::Space, minifb::KeyRepeat::No)
            || window.is_key_pressed(Key::PageDown, minifb::KeyRepeat::No)
            || window.is_key_pressed(Key::Down, minifb::KeyRepeat::No))
            && current_page + 1 < total_pages
        {
            current_page += 1;
            needs_redraw = true;
        } else if (window.is_key_pressed(Key::Left, minifb::KeyRepeat::No)
            || window.is_key_pressed(Key::PageUp, minifb::KeyRepeat::No)
            || window.is_key_pressed(Key::Up, minifb::KeyRepeat::No))
            && current_page > 0
        {
            current_page -= 1;
            needs_redraw = true;
        }

        // Alternância de motor de renderização (tecla M)
        if window.is_key_pressed(Key::M, minifb::KeyRepeat::No) {
            current_backend = match current_backend {
                ViewerBackendKind::NativeRust => ViewerBackendKind::KoreaderCrengine,
                ViewerBackendKind::KoreaderCrengine => ViewerBackendKind::NativeRust,
            };
            engine = create_reader_engine(current_backend)?;
            engine.open_book(path)?;
            engine.set_viewport(width as u32, height as u32)?;
            engine.apply_render_options(&options)?;
            needs_redraw = true;
        }

        // Alternância de tema cromático (tecla T)
        if window.is_key_pressed(Key::T, minifb::KeyRepeat::No) {
            current_theme = match current_theme {
                ViewerColorTheme::Sepia => ViewerColorTheme::PaperWhite,
                ViewerColorTheme::PaperWhite => ViewerColorTheme::Night,
                ViewerColorTheme::Night => ViewerColorTheme::EInkMono,
                ViewerColorTheme::EInkMono => ViewerColorTheme::Sepia,
            };
            options.theme = current_theme;
            engine.apply_render_options(&options)?;
            needs_redraw = true;
        }

        // Ajuste dinâmico de tamanho de fonte (+ e -)
        if window.is_key_pressed(Key::Equal, minifb::KeyRepeat::No) {
            options.font_size_pt = (options.font_size_pt + 1.0).min(28.0);
            engine.apply_render_options(&options)?;
            needs_redraw = true;
        } else if window.is_key_pressed(Key::Minus, minifb::KeyRepeat::No) {
            options.font_size_pt = (options.font_size_pt - 1.0).max(8.0);
            engine.apply_render_options(&options)?;
            needs_redraw = true;
        }

        window
            .update_with_buffer(&pixel_buffer, width, height)
            .map_err(|e| ViewerError::BackendUnavailable(format!("Erro de atualização de buffer: {}", e)))?;
    }

    Ok(())
}

/// Fallback para quando o crate é compilado sem a feature `gui`.
#[cfg(not(feature = "gui"))]
pub fn run_gui_viewer(
    _path: &Path,
    _current_backend: ViewerBackendKind,
    _current_theme: ViewerColorTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    Err(ViewerError::BackendUnavailable(
        "A interface gráfica desktop exige a feature 'gui'. Compile com: cargo build --features gui".into(),
    )
    .into())
}
