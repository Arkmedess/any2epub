//! Módulo de visualização e motor de leitura de e-books EPUB com múltiplos backends selecionáveis.

pub mod bitmap;
pub mod engine;
pub mod font;
pub mod gui;
pub mod interactive;
pub mod koreader;
pub mod native;
pub mod options;

pub use bitmap::PageBitmap;
pub use engine::{
    create_reader_engine, BookDetails, ReaderEngine, ViewerBackendKind, ViewerError,
};
pub use gui::run_gui_viewer;
pub use interactive::run_interactive_viewer;
pub use koreader::KoreaderCrengineEngine;
pub use native::NativeRustReaderEngine;
pub use options::{ViewerColorTheme, ViewerRenderOptions};
