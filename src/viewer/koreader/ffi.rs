//! Definições de tipos C-ABI e assinaturas FFI para o motor `crengine` (CoolReader C++).

use std::os::raw::{c_char, c_int, c_void};

/// Ponteiro opaco para a instância do motor `LVDocView` / `cr3` em C++.
#[repr(C)]
pub struct CRDocViewOpaque {
    _private: [u8; 0],
}

/// Assinaturas canônicas de FFI exportadas pela biblioteca compartilhada `libcrengine` ou wrapper C.
pub type FnCrDocViewCreate = unsafe extern "C" fn() -> *mut CRDocViewOpaque;
pub type FnCrDocViewDestroy = unsafe extern "C" fn(view: *mut CRDocViewOpaque);
pub type FnCrDocViewLoadDocument =
    unsafe extern "C" fn(view: *mut CRDocViewOpaque, path: *const c_char) -> c_int;
pub type FnCrDocViewResize =
    unsafe extern "C" fn(view: *mut CRDocViewOpaque, width: c_int, height: c_int);
pub type FnCrDocViewGetPageCount = unsafe extern "C" fn(view: *mut CRDocViewOpaque) -> c_int;
pub type FnCrDocViewRenderPage = unsafe extern "C" fn(
    view: *mut CRDocViewOpaque,
    page_num: c_int,
    buffer: *mut c_void,
    bytes_per_row: c_int,
) -> c_int;
pub type FnCrDocViewSetFontSize =
    unsafe extern "C" fn(view: *mut CRDocViewOpaque, font_size_pt: c_int);

/// Tabela de símbolos FFI carregados dinamicamente para o backend `crengine`.
pub struct CrengineFfiTable {
    pub create: FnCrDocViewCreate,
    pub destroy: FnCrDocViewDestroy,
    pub load_document: FnCrDocViewLoadDocument,
    pub resize: FnCrDocViewResize,
    pub get_page_count: FnCrDocViewGetPageCount,
    pub render_page: FnCrDocViewRenderPage,
    pub set_font_size: FnCrDocViewSetFontSize,
}

impl CrengineFfiTable {
    /// Tenta carregar os símbolos da biblioteca nativa `libcrengine.so` / `libcrengine.dylib` via `libloading` ou dlsym.
    /// Em ambiente onde a biblioteca C++ não estiver presente, retorna `None`.
    pub fn try_load_system() -> Option<Self> {
        // Implementação extensível para dlopen("libcrengine.so")
        None
    }
}
