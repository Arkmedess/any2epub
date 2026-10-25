//! Integração nativa com `pdfium-render` e `image` para extração de páginas, capa e figuras.

use std::io::Cursor;
use std::path::Path;

use image::ImageFormat;
use pdfium_render::prelude::*;

use crate::domain::ir::{PdfWordBox, RawPdfPage};
use crate::domain::metadata::ExtractedFigure;

/// Tupla de retorno da extração geométrica de PDF contendo `(capa_jpeg, paginas_brutas, figuras)`.
pub type PdfExtractionTuple = (Option<Vec<u8>>, Vec<RawPdfPage>, Vec<ExtractedFigure>);

/// Extrai a capa (página 0), todas as caixas de palavras (`PdfWordBox`) e figuras ilustrativas de um arquivo PDF.
pub fn extract_pdf_with_cover_and_figures(
    pdf_path: &Path,
) -> Result<PdfExtractionTuple, Box<dyn std::error::Error>> {
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())?,
    );

    let document = pdfium.load_pdf_from_file(pdf_path, None)?;
    let total_pages = document.pages().len() as usize;
    let mut raw_pages = Vec::with_capacity(total_pages);
    let mut all_figures = Vec::new();
    let mut cover_jpeg_bytes: Option<Vec<u8>> = None;
    let mut fig_counter = 1usize;

    for (page_idx, page) in document.pages().iter().enumerate() {
        if page_idx == 0 {
            let render_cfg = PdfRenderConfig::new().set_target_width(1400);
            let bitmap = page.render_with_config(&render_cfg)?;
            let dyn_img = bitmap.as_image();
            let mut buf = Cursor::new(Vec::new());
            dyn_img.write_to(&mut buf, ImageFormat::Jpeg)?;
            cover_jpeg_bytes = Some(buf.into_inner());
            continue;
        }

        let width = page.width().value;
        let height = page.height().value;
        let mut words: Vec<PdfWordBox> = Vec::new();
        let mut page_figures: Vec<ExtractedFigure> = Vec::new();

        for object in page.objects().iter() {
            if let Some(text_obj) = object.as_text_object() {
                let text = text_obj.text();
                if text.trim().is_empty() {
                    continue;
                }
                if let Ok(bounds) = object.bounds() {
                    let font_size = text_obj.unscaled_font_size().value;
                    let font_name = text_obj.font().name();
                    let fn_lower = font_name.to_lowercase();
                    let is_italic = text_obj.font().is_italic()
                        || fn_lower.contains("italic")
                        || fn_lower.contains("oblique");
                    let is_bold = fn_lower.contains("bold") || fn_lower.contains("semibold");
                    let is_monospace = fn_lower.contains("lekton") || fn_lower.contains("mono");

                    let top = (height - bounds.top().value).max(0.0);
                    let bottom = (height - bounds.bottom().value).max(top);

                    words.push(PdfWordBox {
                        text,
                        font_name,
                        x0: bounds.left().value,
                        top,
                        x1: bounds.right().value,
                        bottom,
                        font_size,
                        is_italic,
                        is_bold,
                        is_monospace,
                    });
                }
            } else if let Some(img_obj) = object.as_image_object() {
                if let (Ok(bounds), Ok(raw_img)) = (object.bounds(), img_obj.get_raw_image()) {
                    let w_pt = bounds.right().value - bounds.left().value;
                    let h_pt = bounds.top().value - bounds.bottom().value;
                    if w_pt > 85.0 && h_pt > 55.0 && page_idx >= 19 {
                        let mut buf = Cursor::new(Vec::new());
                        if raw_img.write_to(&mut buf, ImageFormat::Jpeg).is_ok() {
                            let fig_id = format!("fig_{:03}", fig_counter);
                            let fig_href =
                                format!("images/fig_{:03}_p{}.jpg", fig_counter, page_idx + 1);
                            let top = (height - bounds.top().value).max(0.0);
                            let fig = ExtractedFigure {
                                id: fig_id,
                                href: fig_href,
                                top,
                                jpeg_bytes: buf.into_inner(),
                            };
                            page_figures.push(fig.clone());
                            all_figures.push(fig);
                            fig_counter += 1;
                        }
                    }
                }
            }
        }

        raw_pages.push(RawPdfPage {
            page_index: page_idx,
            width,
            height,
            words,
            figures: page_figures,
        });
    }

    Ok((cover_jpeg_bytes, raw_pages, all_figures))
}
