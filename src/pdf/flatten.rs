use std::path::Path;

/// Flatten annotations in a PDF file, merging them into the page content.
/// 
/// If keep_original is true, the output will be saved with "_flattened" suffix.
/// Otherwise, the original file will be overwritten.
pub fn flatten_pdf(input_path: &Path, keep_original: bool) -> Result<(), String> {
    // Load the PDF document
    let mut doc = lopdf::Document::load(input_path)
        .map_err(|e| format!("无法加载 PDF: {}", e))?;
    
    // Get all page IDs
    let pages: Vec<_> = doc.get_pages().into_iter().collect();
    
    // Process each page
    for (page_num, page_id) in pages {
        flatten_page(&mut doc, page_id)
            .map_err(|e| format!("处理第 {} 页时出错: {}", page_num, e))?;
    }
    
    // Determine output path
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
    
    // Save the document
    doc.save(&output_path)
        .map_err(|e| format!("无法保存 PDF: {}", e))?;
    
    Ok(())
}

fn flatten_page(doc: &mut lopdf::Document, page_id: lopdf::ObjectId) -> Result<(), String> {
    use lopdf::{Object, Stream};
    
    // First, collect all annotation appearance streams (immutable borrow)
    let appearance_streams = {
        let page = doc.get_object(page_id)
            .map_err(|e| format!("无法获取页面对象: {}", e))?;
        
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
        
        let mut streams = Vec::new();
        
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
                        match normal {
                            Object::Stream(stream) => {
                                streams.push(stream.content.clone());
                            }
                            Object::Reference(ref_id) => {
                                if let Ok(obj) = doc.get_object(*ref_id) {
                                    if let Object::Stream(stream) = obj {
                                        streams.push(stream.content.clone());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        streams
    };
    
    // If we have appearance streams, merge them into page content
    if !appearance_streams.is_empty() {
        // Get current content or create empty
        let current_content = {
            let page = doc.get_object(page_id)
                .map_err(|e| format!("无法获取页面对象: {}", e))?;
            
            if let Object::Dictionary(dict) = page {
                match dict.get(b"Contents") {
                    Ok(Object::Reference(id)) => {
                        if let Ok(Object::Stream(stream)) = doc.get_object(*id) {
                            stream.content.clone()
                        } else {
                            Vec::new()
                        }
                    }
                    Ok(Object::Array(arr)) => {
                        let mut combined = Vec::new();
                        for item in arr {
                            if let Object::Reference(id) = item {
                                if let Ok(Object::Stream(stream)) = doc.get_object(*id) {
                                    combined.extend_from_slice(&stream.content);
                                    combined.extend_from_slice(b"\n");
                                }
                            }
                        }
                        combined
                    }
                    _ => Vec::new(),
                }
            } else {
                Vec::new()
            }
        };
        
        // Build new content with appearance streams appended
        let mut new_content = current_content;
        for stream_content in &appearance_streams {
            new_content.extend_from_slice(b"\nq\n");
            new_content.extend_from_slice(stream_content);
            new_content.extend_from_slice(b"\nQ\n");
        }
        
        // Create new stream and update page
        let new_stream = Stream::new(lopdf::Dictionary::new(), new_content);
        let new_id = doc.add_object(new_stream);
        
        let page = doc.get_object_mut(page_id)
            .map_err(|e| format!("无法修改页面: {}", e))?;
        
        if let Object::Dictionary(dict) = page {
            dict.set(b"Contents", Object::Reference(new_id));
        }
    }
    
    // Remove annotations from page
    let page = doc.get_object_mut(page_id)
        .map_err(|e| format!("无法修改页面: {}", e))?;
    
    if let Object::Dictionary(dict) = page {
        dict.remove(b"Annots");
    }
    
    Ok(())
}
