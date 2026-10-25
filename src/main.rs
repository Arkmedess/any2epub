//! Ponto de entrada do utilitário de linha de comando `pdf2epub` (`any2epub-rs`).

use std::process::ExitCode;

use clap::Parser;

use any2epub_rs::cli::{CliArgs, SourceKind};
use any2epub_rs::epub::NativeEpub3Builder;
use any2epub_rs::extractors::build_extractor;

fn run_cli(args: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    let base_metadata = args.build_base_metadata();
    let effective_kind = args.source_kind.resolve_effective_kind(&args.input);

    match effective_kind {
        SourceKind::MediaWiki => {
            eprintln!(
                "[*] Modo MediaWiki/ArchWiki ativado (BFS max_depth={}, max_chapters={}): {}",
                args.max_depth, args.max_chapters, args.input
            );
        }
        SourceKind::SinglePageSpec => {
            eprintln!(
                "[*] Modo SinglePageSpec ativado (ex: Effective Go): segmentando capítulos H2 e subtópicos H3 de {}",
                args.input
            );
        }
        SourceKind::WebArticle
        | SourceKind::WebDocs
        | SourceKind::SphinxDocs
        | SourceKind::AsciiDocBook
        | SourceKind::MdnGuide => {
            eprintln!(
                "[*] Modo Documentação Web ativado ({:?}): extraindo capítulos e subtópicos de {}",
                effective_kind, args.input
            );
        }
        SourceKind::Pdf | SourceKind::Auto => {
            eprintln!(
                "[*] Modo PDF ativado: extraindo geometria e capa de {}",
                args.input
            );
        }
    }

    let extractor = build_extractor(&args, effective_kind, base_metadata);
    let payload = extractor.extract_payload()?;

    let builder = NativeEpub3Builder::from_payload(payload);
    builder.save(&args.output)?;

    println!(
        "[+] EPUB 3.3 gerado com sucesso em: {}",
        args.output.display()
    );

    Ok(())
}

fn main() -> ExitCode {
    let args = CliArgs::parse();
    if let Err(err) = run_cli(args) {
        eprintln!("[-] Erro durante a conversão para EPUB 3.3: {}", err);
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}
