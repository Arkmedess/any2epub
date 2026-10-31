//! Ponto de entrada do utilitário de linha de comando `any2epub`.

use std::process::ExitCode;

fn main() -> ExitCode {
    any2epub_rs::cli::cli_main()
}
