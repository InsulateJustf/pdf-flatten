use std::path::PathBuf;
use crate::i18n;

#[derive(Clone)]
pub struct PdfInfo {
    pub path: PathBuf,
    pub name: String,
    pub pages: usize,
    pub annotations: usize,
    pub status: String,
}

pub fn get_pdf_info(path: &PathBuf) -> Result<PdfInfo, String> {
    let doc = lopdf::Document::load(path)
        .map_err(|e| i18n::err_load_pdf(&e.to_string()))?;
    
    let page_count = doc.get_pages().len();
    
    let mut annotation_count = 0;
    for (_, page_id) in doc.get_pages() {
        if let Ok(page) = doc.get_object(page_id) {
            if let lopdf::Object::Dictionary(dict) = page {
                if let Ok(annots) = dict.get(b"Annots") {
                    if let lopdf::Object::Array(array) = annots {
                        annotation_count += array.len();
                    }
                }
            }
        }
    }
    
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown.pdf".to_string());
    
    Ok(PdfInfo {
        path: path.clone(),
        name,
        pages: page_count,
        annotations: annotation_count,
        status: i18n::status_pending().to_string(),
    })
}
