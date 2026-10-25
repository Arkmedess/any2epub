//! Construtor nativo de pacotes W3C EPUB 3.3 (`NativeEpub3Builder`).

pub mod container;
pub mod css;
pub mod nav;
pub mod opf;
pub mod xhtml;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use rayon::prelude::*;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::domain::ir::{Chapter, ExtractedBookPayload};
use crate::domain::metadata::{BookMetadata, ExtractedFigure};

pub use container::build_container_xml;
pub use css::build_editorial_css;
pub use nav::{build_nav_xhtml, render_toc_nodes_recursive};
pub use opf::build_content_opf;
pub use xhtml::{build_chapter_xhtml, build_cover_xhtml, build_synopsis_xhtml};

/// Construtor nativo de contêineres OCF / W3C EPUB 3.3 com sumário hierárquico aninhado.
pub struct NativeEpub3Builder {
    /// Metadados editoriais e capa da publicação.
    pub metadata: BookMetadata,
    /// Capítulos estruturados em ordem de leitura.
    pub chapters: Vec<Chapter>,
    /// Figuras extraídas a incorporar em `OEBPS/images/`.
    pub figures: Vec<ExtractedFigure>,
}

impl NativeEpub3Builder {
    /// Instancia o construtor a partir de um [`ExtractedBookPayload`] produzido por qualquer extrator.
    pub fn from_payload(payload: ExtractedBookPayload) -> Self {
        Self {
            metadata: payload.metadata,
            chapters: payload.chapters,
            figures: payload.figures,
        }
    }

    /// Grava o pacote EPUB 3.3 completo no disco com `mimetype` sem compressão no byte offset 0.
    pub fn save(&self, output_path: &Path) -> std::io::Result<()> {
        let file = File::create(output_path)?;
        let buf_writer = BufWriter::with_capacity(512 * 1024, file);
        let mut zip = ZipWriter::new(buf_writer);

        let stored_opts = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .unix_permissions(0o644);
        zip.start_file("mimetype", stored_opts)?;
        zip.write_all(b"application/epub+zip")?;

        let deflated_opts = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(6))
            .unix_permissions(0o644);

        zip.start_file("META-INF/container.xml", deflated_opts)?;
        zip.write_all(build_container_xml().as_bytes())?;

        zip.start_file("OEBPS/style.css", deflated_opts)?;
        zip.write_all(build_editorial_css().as_bytes())?;

        if let Some(ref cover_bytes) = self.metadata.cover_jpeg_bytes {
            zip.start_file("OEBPS/images/cover.jpg", deflated_opts)?;
            zip.write_all(cover_bytes)?;

            zip.start_file("OEBPS/cover.xhtml", deflated_opts)?;
            zip.write_all(build_cover_xhtml(&self.metadata).as_bytes())?;
        }

        zip.start_file("OEBPS/sinopse.xhtml", deflated_opts)?;
        zip.write_all(build_synopsis_xhtml(&self.metadata).as_bytes())?;

        for fig in &self.figures {
            let fig_path = format!("OEBPS/{}", fig.href);
            zip.start_file(fig_path, deflated_opts)?;
            zip.write_all(&fig.jpeg_bytes)?;
        }

        zip.start_file("OEBPS/nav.xhtml", deflated_opts)?;
        zip.write_all(build_nav_xhtml(&self.metadata, &self.chapters).as_bytes())?;

        zip.start_file("OEBPS/content.opf", deflated_opts)?;
        zip.write_all(build_content_opf(&self.metadata, &self.chapters, &self.figures).as_bytes())?;

        let rendered_chapters: Vec<(&str, String)> = self
            .chapters
            .par_iter()
            .map(|ch| {
                (
                    ch.filename.as_str(),
                    build_chapter_xhtml(&self.metadata, ch),
                )
            })
            .collect();

        for (filename, xhtml_content) in rendered_chapters {
            let path_in_zip = format!("OEBPS/{}", filename);
            zip.start_file(path_in_zip, deflated_opts)?;
            zip.write_all(xhtml_content.as_bytes())?;
        }

        zip.finish()?;
        Ok(())
    }
}
