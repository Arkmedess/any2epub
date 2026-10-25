//! Geração do descritor OCF `META-INF/container.xml` segundo a especificação W3C EPUB 3.3.

/// Retorna o conteúdo XML estático do arquivo `META-INF/container.xml`.
pub fn build_container_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#
}
