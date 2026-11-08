//! Leitor interativo de terminal com status bar e ergonomia inspirados no KOReader.

use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::viewer::engine::{create_reader_engine, ViewerBackendKind};
use crate::viewer::options::{ViewerColorTheme, ViewerRenderOptions};

/// Inicia a sessão de leitura interativa de terminal estilo KOReader.
pub fn run_interactive_viewer(
    path: &Path,
    mut current_backend: ViewerBackendKind,
    mut current_theme: ViewerColorTheme,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_page = 0;
    let mut options = ViewerRenderOptions {
        theme: current_theme,
        ..Default::default()
    };

    let mut engine = create_reader_engine(current_backend)?;
    let details = engine.open_book(path)?;
    engine.apply_render_options(&options)?;

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        let total_pages = engine.total_pages().max(1);
        if current_page >= total_pages {
            current_page = total_pages.saturating_sub(1);
        }

        // Limpa o terminal ou adiciona separador de página
        write!(stdout, "\x1B[2J\x1B[1;1H")?;

        // Barra Superior
        writeln!(
            stdout,
            "╔════════════════════════════════════════════════════════════════════════════════╗"
        )?;
        writeln!(
            stdout,
            "║ 📖 {} │ Motor: {:<16} ║",
            truncate_str(&details.title, 45),
            engine.engine_name()
        )?;
        writeln!(
            stdout,
            "╚════════════════════════════════════════════════════════════════════════════════╝"
        )?;

        // Renderiza a página no motor (calculando o bitmap e gerando o layout visual)
        let _bitmap = engine.render_page(current_page)?;

        writeln!(stdout)?;
        writeln!(stdout, "  [Página {}]", current_page + 1)?;
        writeln!(stdout)?;
        writeln!(
            stdout,
            "  (Conteúdo paginado renderizado com sucesso no buffer RGBA de {}x{} px)",
            _bitmap.width, _bitmap.height
        )?;
        writeln!(stdout)?;

        // Barra Inferior Estilo KOReader
        let percent = ((current_page + 1) as f32 / total_pages as f32) * 100.0;
        let progress_bar = render_ascii_progress_bar(percent, 20);
        let est_minutes =
            ((total_pages.saturating_sub(current_page + 1)) as f32 * 1.2).ceil() as usize;

        writeln!(
            stdout,
            "──────────────────────────────────────────────────────────────────────────────────"
        )?;
        writeln!(
            stdout,
            " {} Pág. {} de {} ({:.1}%) │ Restante: ~{} min │ Tema: {:?}",
            progress_bar,
            current_page + 1,
            total_pages,
            percent,
            est_minutes,
            current_theme
        )?;
        writeln!(
            stdout,
            " [n/Enter] Próx │ [p] Ant │ [m] Alternar Motor │ [t] Tema │ [g N] Ir p/ │ [q] Sair"
        )?;
        writeln!(
            stdout,
            "──────────────────────────────────────────────────────────────────────────────────"
        )?;
        write!(stdout, "> ")?;
        stdout.flush()?;

        let mut input = String::new();
        if stdin.lock().read_line(&mut input)? == 0 {
            break; // EOF
        }

        let cmd = input.trim().to_lowercase();
        match cmd.as_str() {
            "q" | "quit" | "exit" => {
                writeln!(stdout, "[*] Encerrando visualizador KOReader.")?;
                break;
            }
            "n" | "" => {
                if current_page + 1 < total_pages {
                    current_page += 1;
                }
            }
            "p" => {
                current_page = current_page.saturating_sub(1);
            }
            "m" => {
                // Alterna o motor de renderização em tempo real
                current_backend = match current_backend {
                    ViewerBackendKind::NativeRust => ViewerBackendKind::KoreaderCrengine,
                    ViewerBackendKind::KoreaderCrengine => ViewerBackendKind::NativeRust,
                };
                engine = create_reader_engine(current_backend)?;
                engine.open_book(path)?;
                engine.apply_render_options(&options)?;
            }
            "t" => {
                // Alterna o tema cromático
                current_theme = match current_theme {
                    ViewerColorTheme::Sepia => ViewerColorTheme::PaperWhite,
                    ViewerColorTheme::PaperWhite => ViewerColorTheme::Night,
                    ViewerColorTheme::Night => ViewerColorTheme::EInkMono,
                    ViewerColorTheme::EInkMono => ViewerColorTheme::Sepia,
                };
                options.theme = current_theme;
                engine.apply_render_options(&options)?;
            }
            c if c.starts_with('g') => {
                if let Some(num_str) = c.strip_prefix('g').map(|s| s.trim()) {
                    if let Ok(target) = num_str.parse::<usize>() {
                        if target > 0 && target <= total_pages {
                            current_page = target - 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

fn render_ascii_progress_bar(percent: f32, bar_len: usize) -> String {
    let filled = ((percent / 100.0) * bar_len as f32).round() as usize;
    let filled = filled.min(bar_len);
    let empty = bar_len - filled;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}
