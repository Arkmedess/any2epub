//! Camada de interface de linha de comando (CLI) e resolução de argumentos via `clap`.

pub mod args;
pub mod runner;

pub use args::{CliArgs, CliCommand, SourceKind};
pub use runner::{
    cli_main, handle_validate_command, handle_view_command, print_validation_report, run_cli,
};
