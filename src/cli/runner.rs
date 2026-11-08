//! Executor e orquestrador de comandos da CLI (`run_cli`).

use std::path::Path;
use std::process::ExitCode;

use crate::cli::args::{CliArgs, CliCommand, SourceKind};
use crate::epub::validator::{NativeEpubValidator, ValidationReport};
use crate::epub::NativeEpub3Builder;
use crate::extractors::build_extractor;
use crate::viewer::{
    create_reader_engine, run_gui_viewer, run_interactive_viewer, ViewerBackendKind,
    ViewerColorTheme, ViewerRenderOptions,
};

/// Emite o relatório de validação W3C EPUB 3.3 para stdout (formato legível ou JSON).
pub fn print_validation_report(report: &ValidationReport, as_json: bool, target_path: &Path) {
    if as_json {
        let json_errors: Vec<String> = report
            .errors
            .iter()
            .map(|e| format!("\"{}\"", e.replace('\"', "\\\"")))
            .collect();
        let json_warnings: Vec<String> = report
            .warnings
            .iter()
            .map(|w| format!("\"{}\"", w.replace('\"', "\\\"")))
            .collect();

        println!(
            "{{\"path\":\"{}\",\"valid\":{},\"total_files\":{},\"spine_items\":{},\"errors\":[{}],\"warnings\":[{}]}}",
            target_path.display(),
            report.is_valid,
            report.total_files,
            report.spine_items,
            json_errors.join(","),
            json_warnings.join(",")
        );
    } else {
        println!(
            "[i] Relatório de Validação W3C EPUB 3.3: {}",
            target_path.display()
        );
        println!(
            "    Status: {}",
            if report.is_valid {
                "VÁLIDO (100% Conforme)"
            } else {
                "INVÁLIDO"
            }
        );
        println!("    Arquivos no contêiner: {}", report.total_files);
        println!(
            "    Itens no spine (capítulos/seções): {}",
            report.spine_items
        );

        if !report.warnings.is_empty() {
            println!("    Advertências ({}):", report.warnings.len());
            for w in &report.warnings {
                println!("      - [!] {}", w);
            }
        }

        if !report.errors.is_empty() {
            println!("    Violações estruturais ({}):", report.errors.len());
            for e in &report.errors {
                println!("      - [x] {}", e);
            }
        }
    }
}

/// Manipula o subcomando de validação direta de arquivos EPUB.
pub fn handle_validate_command(path: &Path, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let report = NativeEpubValidator::validate_file(path)?;
    print_validation_report(&report, json, path);

    if !report.is_valid {
        return Err("O arquivo EPUB falhou nas verificações de conformidade W3C.".into());
    }
    Ok(())
}

/// Manipula a visualização e paginação de arquivos EPUB através do motor selecionado.
pub fn handle_view_command(
    path: &Path,
    backend_str: &str,
    theme_str: &str,
    page: usize,
    interactive: bool,
    gui: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let backend = match backend_str.to_lowercase().as_str() {
        "koreader" | "crengine" | "koreader-crengine" => ViewerBackendKind::KoreaderCrengine,
        _ => ViewerBackendKind::NativeRust,
    };

    let theme = match theme_str.to_lowercase().as_str() {
        "night" | "dark" => ViewerColorTheme::Night,
        "paper-white" | "white" => ViewerColorTheme::PaperWhite,
        "e-ink" | "eink" | "e-ink-mono" => ViewerColorTheme::EInkMono,
        _ => ViewerColorTheme::Sepia,
    };

    if gui {
        return run_gui_viewer(path, backend, theme);
    }

    if interactive {
        return run_interactive_viewer(path, backend, theme);
    }

    let mut engine = create_reader_engine(backend)?;
    let details = engine.open_book(path)?;

    let options = ViewerRenderOptions {
        theme,
        ..Default::default()
    };
    engine.apply_render_options(&options)?;

    let total_pages = engine.total_pages();
    let current_page = page.min(total_pages.saturating_sub(1));
    let bitmap = engine.render_page(current_page)?;

    println!("[*] Motor de Leitura Ativado: '{}'", engine.engine_name());
    println!(
        "[*] Obra: '{}' por {}",
        details.title,
        if details.authors.is_empty() {
            String::from("Autor Desconhecido")
        } else {
            details.authors.join(", ")
        }
    );
    println!(
        "    Total de Seções no Spine: {}",
        details.spine_documents_count
    );
    println!(
        "    Paginação Atual: Página {} de {} ({:.1}%)",
        current_page + 1,
        total_pages,
        ((current_page + 1) as f32 / total_pages as f32) * 100.0
    );
    println!("    Tema Ativo: {:?}", theme);
    println!(
        "    Buffer Renderizado: {}x{} pixels (RGBA8888, {} bytes)",
        bitmap.width,
        bitmap.height,
        bitmap.rgba_pixels.len()
    );

    Ok(())
}

/// Orquestra a execução da CLI de acordo com os argumentos passados.
pub fn run_cli(args: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(CliCommand::Validate { path, json }) = &args.command {
        return handle_validate_command(path, *json);
    }

    if let Some(CliCommand::View {
        path,
        backend,
        theme,
        page,
        interactive,
        gui,
    }) = &args.command
    {
        return handle_view_command(path, backend, theme, *page, *interactive, *gui);
    }

    let input_path = match &args.input {
        Some(inp) if !inp.trim().is_empty() => inp,
        _ => {
            eprintln!("[-] Erro: Nenhum arquivo de entrada fornecido.");
            eprintln!("    Utilize: any2epub --input <arquivo-ou-url> [opções]");
            eprintln!("    Ou consulte o manual com: any2epub --help");
            std::process::exit(2);
        }
    };

    let base_metadata = args.build_base_metadata();
    let effective_kind = args.source_kind.resolve_effective_kind(input_path);

    if !args.quiet {
        match effective_kind {
            SourceKind::MediaWiki => {
                eprintln!(
                    "[*] Modo MediaWiki/ArchWiki ativado (BFS max_depth={}, max_chapters={}): {}",
                    args.max_depth, args.max_chapters, input_path
                );
            }
            SourceKind::SinglePageSpec => {
                eprintln!(
                    "[*] Modo SinglePageSpec ativado (ex: Effective Go): segmentando capítulos H2 e subtópicos H3 de {}",
                    input_path
                );
            }
            SourceKind::WebArticle
            | SourceKind::WebDocs
            | SourceKind::SphinxDocs
            | SourceKind::AsciiDocBook
            | SourceKind::MdnGuide => {
                eprintln!(
                    "[*] Modo Documentação Web ativado ({:?}): extraindo capítulos e subtópicos de {}",
                    effective_kind, input_path
                );
            }
            SourceKind::Markdown => {
                eprintln!(
                    "[*] Modo Markdown Local ativado: processando estrutura e subtópicos de {}",
                    input_path
                );
            }
            SourceKind::Pdf | SourceKind::Auto => {
                eprintln!(
                    "[*] Modo PDF ativado: extraindo geometria e capa de {}",
                    input_path
                );
            }
        }
    }

    let extractor = build_extractor(&args, effective_kind, base_metadata);
    let payload = extractor.extract_payload()?;

    let builder = NativeEpub3Builder::from_payload(payload);
    builder.save(&args.output)?;

    if !args.quiet && !args.json {
        eprintln!(
            "[+] EPUB 3.3 gerado com sucesso em: {}",
            args.output.display()
        );
    } else if !args.quiet && args.json && !args.validate {
        println!(
            "{{\"path\":\"{}\",\"success\":true}}",
            args.output.display()
        );
    }

    if args.validate {
        let report = NativeEpubValidator::validate_file(&args.output)?;
        print_validation_report(&report, args.json, &args.output);
        if !report.is_valid {
            return Err("O EPUB gerado falhou na validação de conformidade W3C.".into());
        }
    }

    if args.gui {
        handle_view_command(&args.output, "native", "sepia", 0, false, true)?;
    } else if args.view {
        handle_view_command(&args.output, "native", "sepia", 0, false, false)?;
    }

    Ok(())
}

/// Ponto de entrada padrão para binários CLI.
pub fn cli_main() -> ExitCode {
    use clap::Parser;
    let args = CliArgs::parse();
    if let Err(err) = run_cli(args) {
        eprintln!("[-] Erro na execução do any2epub: {}", err);
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}
