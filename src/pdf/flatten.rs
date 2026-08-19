use std::path::Path;
use crate::i18n;

pub fn flatten_pdf(input_path: &Path, keep_original: bool) -> Result<(), String> {
    let mut doc = lopdf::Document::load(input_path)
        .map_err(|e| i18n::err_load_pdf(&e.to_string()))?;
    
    let pages: Vec<_> = doc.get_pages().into_iter().collect();
    
    for (page_num, page_id) in pages {
        flatten_page(&mut doc, page_id)
            .map_err(|e| i18n::err_process_page(page_num, &e))?;
    }
    
    let output_path = if keep_original {
        let stem = input_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        let extension = input_path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_else(|| "pdf".to_string());
        let parent = input_path.parent().unwrap_or(Path::new("."));
        parent.join(format!("{}_flattened.{}", stem, extension))
    } else {
        input_path.to_path_buf()
    };
    
    doc.save(&output_path)
        .map_err(|e| i18n::err_save_pdf(&e.to_string()))?;
    
    Ok(())
}

fn get_stream_content(_doc: &lopdf::Document, stream: &lopdf::Stream) -> Result<Vec<u8>, String> {
    match stream.decompressed_content() {
        Ok(content) => Ok(content),
        Err(_) => Ok(stream.content.clone()),
    }
}

fn get_content_from_object(doc: &lopdf::Document, obj: &lopdf::Object) -> Result<Vec<u8>, String> {
    use lopdf::Object;
    
    match obj {
        Object::Stream(stream) => {
            get_stream_content(doc, stream)
        }
        Object::Array(arr) => {
            let mut combined = Vec::new();
            for item in arr {
                match item {
                    Object::Reference(id) => {
                        match doc.get_object(*id) {
                            Ok(Object::Stream(stream)) => {
                                let content = get_stream_content(doc, stream)?;
                                combined.extend_from_slice(&content);
                                combined.extend_from_slice(b"\n");
                            }
                            Ok(Object::Array(inner_arr)) => {
                                let inner_content = get_content_from_object(doc, &Object::Array(inner_arr.clone()))?;
                                combined.extend_from_slice(&inner_content);
                            }
                            _ => {}
                        }
                    }
                    Object::Stream(stream) => {
                        let content = get_stream_content(doc, stream)?;
                        combined.extend_from_slice(&content);
                        combined.extend_from_slice(b"\n");
                    }
                    _ => {}
                }
            }
            Ok(combined)
        }
        Object::Reference(id) => {
            match doc.get_object(*id) {
                Ok(inner_obj) => get_content_from_object(doc, inner_obj),
                Err(e) => Err(i18n::err_deref_object(&format!("{:?}", e))),
            }
        }
        _ => Ok(Vec::new()),
    }
}

fn flatten_page(doc: &mut lopdf::Document, page_id: lopdf::ObjectId) -> Result<(), String> {
    use lopdf::{Object, Stream};
    
    let annotation_data = {
        let page = doc.get_object(page_id)
            .map_err(|e| i18n::err_get_page(&e.to_string()))?;
        
        let page_dict = match page {
            Object::Dictionary(dict) => dict,
            _ => return Ok(()),
        };
        
        let annots = match page_dict.get(b"Annots") {
            Ok(annots) => annots.clone(),
            Err(_) => return Ok(()),
        };
        
        let annot_refs = match &annots {
            Object::Array(arr) => arr.clone(),
            _ => return Ok(()),
        };
        
        if annot_refs.is_empty() {
            return Ok(());
        }
        
        let mut data = Vec::new();
        
        for annot_ref in &annot_refs {
            let annot_id = match annot_ref {
                Object::Reference(id) => *id,
                _ => continue,
            };
            
            let annot = match doc.get_object(annot_id) {
                Ok(obj) => obj,
                Err(_) => continue,
            };
            
            let annot_dict = match annot {
                Object::Dictionary(dict) => dict,
                _ => continue,
            };
            
            if let Ok(ap) = annot_dict.get(b"AP") {
                if let Object::Dictionary(ap_dict) = ap {
                    if let Ok(normal) = ap_dict.get(b"N") {
                        let stream_content = match normal {
                            Object::Stream(stream) => {
                                get_stream_content(doc, stream)?
                            }
                            Object::Reference(ref_id) => {
                                if let Ok(obj) = doc.get_object(*ref_id) {
                                    if let Object::Stream(stream) = obj {
                                        get_stream_content(doc, stream)?
                                    } else {
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            }
                            _ => continue,
                        };
                        
                        data.push(stream_content);
                    }
                }
            }
        }
        
        data
    };
    
    if !annotation_data.is_empty() {
        let current_content = {
            let page = doc.get_object(page_id)
                .map_err(|e| i18n::err_get_page(&e.to_string()))?;
            
            if let Object::Dictionary(dict) = page {
                match dict.get(b"Contents") {
                    Ok(contents) => get_content_from_object(doc, contents)?,
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            }
        };
        
        let mut new_content = Vec::new();
        
        new_content.extend_from_slice(b"q\n");
        new_content.extend_from_slice(&current_content);
        new_content.extend_from_slice(b"\nQ\n");
        
        for stream_content in &annotation_data {
            new_content.extend_from_slice(b"q\n");
            new_content.extend_from_slice(stream_content);
            new_content.extend_from_slice(b"\nQ\n");
        }
        
        let new_stream = Stream::new(lopdf::Dictionary::new(), new_content);
        let new_id = doc.add_object(new_stream);
        
        let page = doc.get_object_mut(page_id)
            .map_err(|e| i18n::err_modify_page(&e.to_string()))?;
        
        if let Object::Dictionary(dict) = page {
            dict.set(b"Contents", Object::Reference(new_id));
        }
    }
    
    let page = doc.get_object_mut(page_id)
        .map_err(|e| i18n::err_modify_page(&e.to_string()))?;
    
    if let Object::Dictionary(dict) = page {
        dict.remove(b"Annots");
    }
    
    Ok(())
}
