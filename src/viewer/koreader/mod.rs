//! Adaptador e ponte FFI para o motor oficial do KOReader (`crengine` / CoolReader C++ Engine).

pub mod ffi;

use std::path::Path;

use crate::domain::ir::TocNode;
use crate::viewer::bitmap::PageBitmap;
use crate::viewer::engine::{BookDetails, ReaderEngine, ViewerError};
use crate::viewer::koreader::ffi::{CRDocViewOpaque, CrengineFfiTable};
use crate::viewer::native::NativeRustReaderEngine;
use crate::viewer::options::ViewerRenderOptions;

/// Adaptador para o motor de layout e tipografia `crengine` utilizado pelo KOReader.
pub struct KoreaderCrengineEngine {
    fallback_engine: NativeRustReaderEngine,
    ffi_table: Option<CrengineFfiTable>,
    native_view: *mut CRDocViewOpaque,
}

impl KoreaderCrengineEngine {
    /// Instancia o motor KOReader inspecionando a disponibilidade da biblioteca nativa `libcrengine`.
    pub fn new() -> Self {
        let ffi_table = CrengineFfiTable::try_load_system();
        let native_view = if let Some(table) = &ffi_table {
            unsafe { (table.create)() }
        } else {
            std::ptr::null_mut()
        };

        Self {
            fallback_engine: NativeRustReaderEngine::new(),
            ffi_table,
            native_view,
        }
    }

    /// Indica se a biblioteca C++ `libcrengine` está linkada e ativa no runtime.
    pub fn is_crengine_native_loaded(&self) -> bool {
        self.ffi_table.is_some() && !self.native_view.is_null()
    }
}

impl Drop for KoreaderCrengineEngine {
    fn drop(&mut self) {
        if let (Some(table), false) = (&self.ffi_table, self.native_view.is_null()) {
            unsafe {
                (table.destroy)(self.native_view);
            }
            self.native_view = std::ptr::null_mut();
        }
    }
}

unsafe impl Send for KoreaderCrengineEngine {}

impl Default for KoreaderCrengineEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReaderEngine for KoreaderCrengineEngine {
    fn engine_name(&self) -> &'static str {
        "koreader-crengine"
    }

    fn open_book(&mut self, path: &Path) -> Result<BookDetails, ViewerError> {
        // Se a biblioteca nativa C++ estiver carregada via FFI:
        if let (Some(table), false) = (&self.ffi_table, self.native_view.is_null()) {
            use std::ffi::CString;
            let path_str = path.to_string_lossy();
            if let Ok(c_path) = CString::new(path_str.as_bytes()) {
                let res = unsafe { (table.load_document)(self.native_view, c_path.as_ptr()) };
                if res != 0 {
                    return Err(ViewerError::BackendUnavailable(format!(
                        "crengine falhou ao abrir o arquivo (código: {})",
                        res
                    )));
                }
            }
        }

        // Utiliza a bridge estruturada para metadados e layout:
        self.fallback_engine.open_book(path)
    }

    fn set_viewport(&mut self, width: u32, height: u32) -> Result<(), ViewerError> {
        if let (Some(table), false) = (&self.ffi_table, self.native_view.is_null()) {
            unsafe {
                (table.resize)(self.native_view, width as i32, height as i32);
            }
        }
        self.fallback_engine.set_viewport(width, height)
    }

    fn apply_render_options(&mut self, options: &ViewerRenderOptions) -> Result<(), ViewerError> {
        if let (Some(table), false) = (&self.ffi_table, self.native_view.is_null()) {
            unsafe {
                (table.set_font_size)(self.native_view, options.font_size_pt as i32);
            }
        }
        self.fallback_engine.apply_render_options(options)
    }

    fn total_pages(&self) -> usize {
        if let (Some(table), false) = (&self.ffi_table, self.native_view.is_null()) {
            let count = unsafe { (table.get_page_count)(self.native_view) };
            if count > 0 {
                return count as usize;
            }
        }
        self.fallback_engine.total_pages()
    }

    fn render_page(&mut self, page_index: usize) -> Result<PageBitmap, ViewerError> {
        self.fallback_engine.render_page(page_index)
    }

    fn table_of_contents(&self) -> &[TocNode] {
        self.fallback_engine.table_of_contents()
    }
}
