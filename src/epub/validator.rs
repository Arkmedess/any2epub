//! Validador estrutural e semântico de integridade W3C EPUB 3.3 (`NativeEpubValidator`).

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;
use zip::ZipArchive;

/// Relatório consolidado de validação de um pacote EPUB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    /// Indica se o pacote atende a todas as regras canônicas estruturais W3C EPUB 3.3.
    pub is_valid: bool,
    /// Lista de violações e erros que invalidam a conformidade.
    pub errors: Vec<String>,
    /// Lista de avisos ou advertências recomendadas.
    pub warnings: Vec<String>,
    /// Contagem total de arquivos e entradas no contêiner ZIP.
    pub total_files: usize,
    /// Contagem de capítulos e documentos referenciados no `<spine>`.
    pub spine_items: usize,
}

/// Validador nativo de arquivos e streams EPUB 3.3.
pub struct NativeEpubValidator;

impl NativeEpubValidator {
    /// Valida um arquivo EPUB a partir do seu caminho no sistema de arquivos.
    pub fn validate_file(path: &Path) -> Result<ValidationReport, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::validate_reader(reader)
    }

    /// Valida um contêiner EPUB a partir de qualquer leitor que implemente `Read + Seek`.
    pub fn validate_reader<R: Read + Seek>(reader: R) -> Result<ValidationReport, std::io::Error> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut spine_items = 0;

        let mut archive = match ZipArchive::new(reader) {
            Ok(arch) => arch,
            Err(err) => {
                return Ok(ValidationReport {
                    is_valid: false,
                    errors: vec![format!("Arquivo não é um contêiner ZIP válido: {}", err)],
                    warnings: Vec::new(),
                    total_files: 0,
                    spine_items: 0,
                });
            }
        };

        let total_files = archive.len();
        if total_files == 0 {
            errors.push(String::from("O contêiner ZIP está completamente vazio."));
            return Ok(ValidationReport {
                is_valid: false,
                errors,
                warnings,
                total_files: 0,
                spine_items: 0,
            });
        }

        // Regra 1: O primeiro arquivo no offset 0 DEVE ser 'mimetype' e descompactado (Stored)
        match archive.by_index(0) {
            Ok(mut first_file) => {
                if first_file.name() != "mimetype" {
                    errors.push(format!(
                        "Violação W3C OCF: A primeira entrada do contêiner deve ser 'mimetype', mas foi '{}'",
                        first_file.name()
                    ));
                } else {
                    if first_file.compression() != zip::CompressionMethod::Stored {
                        errors.push(format!(
                            "Violação W3C OCF: O arquivo 'mimetype' deve ser gravado sem compressão (Stored), mas utilizou {:?}",
                            first_file.compression()
                        ));
                    }
                    let mut content = String::new();
                    if let Err(e) = first_file.read_to_string(&mut content) {
                        errors.push(format!("Falha ao ler o conteúdo do mimetype: {}", e));
                    } else if content.trim() != "application/epub+zip" {
                        errors.push(format!(
                            "Violação W3C OCF: Conteúdo do mimetype deve ser 'application/epub+zip', encontrado '{}'",
                            content
                        ));
                    }
                }
            }
            Err(e) => {
                errors.push(format!(
                    "Falha ao inspecionar primeira entrada do contêiner: {}",
                    e
                ));
            }
        }

        // Regra 2: META-INF/container.xml obrigatório
        let mut opf_path = String::new();
        match archive.by_name("META-INF/container.xml") {
            Ok(mut container_file) => {
                let mut xml_content = String::new();
                if let Err(e) = container_file.read_to_string(&mut xml_content) {
                    errors.push(format!("Falha ao ler META-INF/container.xml: {}", e));
                } else {
                    if let Some(pos) = xml_content.find("full-path=\"") {
                        let remainder = &xml_content[pos + 11..];
                        if let Some(end_pos) = remainder.find('"') {
                            opf_path = remainder[..end_pos].to_string();
                        }
                    }
                    if opf_path.is_empty() {
                        errors.push(String::from(
                            "Violação W3C OCF: META-INF/container.xml não contém atributo 'full-path' apontando para o arquivo OPF.",
                        ));
                    }
                }
            }
            Err(_) => {
                errors.push(String::from(
                    "Violação W3C OCF: Arquivo obrigatório META-INF/container.xml não encontrado.",
                ));
            }
        }

        // Regra 3: Inspecionar o arquivo OPF
        if !opf_path.is_empty() {
            let mut manifest_items = HashMap::new();
            let mut opf_base_dir = String::new();
            if let Some(slash_pos) = opf_path.rfind('/') {
                opf_base_dir = format!("{}/", &opf_path[..slash_pos]);
            }

            let opf_read_result = match archive.by_name(&opf_path) {
                Ok(mut opf_file) => {
                    let mut opf_content = String::new();
                    opf_file
                        .read_to_string(&mut opf_content)
                        .map(|_| opf_content)
                }
                Err(_) => {
                    errors.push(format!(
                        "Violação W3C OCF: Arquivo de pacote OPF '{}' referenciado no container.xml não existe no ZIP.",
                        opf_path
                    ));
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "OPF file not found in archive",
                    ))
                }
            };

            if let Ok(opf_content) = opf_read_result {
                if !opf_content.contains("<package") || !opf_content.contains("version=\"3.") {
                    warnings.push(String::from(
                        "O arquivo OPF não possui tag <package> explícita com version=\"3.0\" ou \"3.3\".",
                    ));
                }
                if !opf_content.contains("<dc:identifier") {
                    errors.push(String::from(
                        "Violação Dublin Core: <dc:identifier> ausente no OPF.",
                    ));
                }
                if !opf_content.contains("<dc:title") {
                    errors.push(String::from(
                        "Violação Dublin Core: <dc:title> ausente no OPF.",
                    ));
                }
                if !opf_content.contains("<dc:language") {
                    errors.push(String::from(
                        "Violação Dublin Core: <dc:language> ausente no OPF.",
                    ));
                }

                let mut cursor = 0;
                let mut has_nav_property = false;
                while let Some(item_start) = opf_content[cursor..].find("<item ") {
                    let actual_start = cursor + item_start;
                    if let Some(tag_end) = opf_content[actual_start..].find('>') {
                        let tag_str = &opf_content[actual_start..actual_start + tag_end];
                        let id = extract_attr(tag_str, "id");
                        let href = extract_attr(tag_str, "href");
                        let props = extract_attr(tag_str, "properties");

                        if props.as_deref() == Some("nav") {
                            has_nav_property = true;
                        }

                        if let (Some(id_val), Some(href_val)) = (id, href) {
                            manifest_items.insert(id_val, href_val);
                        }
                        cursor = actual_start + tag_end;
                    } else {
                        break;
                    }
                }

                if !has_nav_property {
                    errors.push(String::from(
                        "Violação W3C EPUB 3: Nenhum item do manifesto possui o atributo obrigatório properties=\"nav\".",
                    ));
                }

                let mut spine_cursor = 0;
                while let Some(itemref_start) = opf_content[spine_cursor..].find("<itemref ") {
                    let actual_start = spine_cursor + itemref_start;
                    if let Some(tag_end) = opf_content[actual_start..].find('>') {
                        let tag_str = &opf_content[actual_start..actual_start + tag_end];
                        if let Some(idref) = extract_attr(tag_str, "idref") {
                            spine_items += 1;
                            if !manifest_items.contains_key(&idref) {
                                errors.push(format!(
                                    "Violação de Integridade: <itemref idref=\"{}\"> no spine não existe no <manifest>.",
                                    idref
                                ));
                            }
                        }
                        spine_cursor = actual_start + tag_end;
                    } else {
                        break;
                    }
                }

                for (id, href) in &manifest_items {
                    let full_resource_path = format!("{}{}", opf_base_dir, href);
                    if archive.by_name(&full_resource_path).is_err() {
                        errors.push(format!(
                            "Recurso ausente no pacote ZIP: item id='{}' aponta para '{}', que não existe no arquivo.",
                            id, full_resource_path
                        ));
                    }
                }
            }
        }

        let is_valid = errors.is_empty();
        Ok(ValidationReport {
            is_valid,
            errors,
            warnings,
            total_files,
            spine_items,
        })
    }
}

fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr_name);
    let start_idx = tag.find(&pattern)? + pattern.len();
    let remainder = &tag[start_idx..];
    let end_idx = remainder.find('"')?;
    Some(remainder[..end_idx].to_string())
}
