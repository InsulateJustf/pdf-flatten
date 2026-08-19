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

/// Get the decompressed content of a stream, preserving original if decompression fails
fn get_stream_content(_doc: &lopdf::Document, stream: &lopdf::Stream) -> Result<Vec<u8>, String> {
    // Try to decompress if filtered
    match stream.decompressed_content() {
        Ok(content) => Ok(content),
        Err(_) => {
            // If decompression fails, use raw content
            Ok(stream.content.clone())
        }
    }
}

/// Get content from a content object (could be a stream, array of streams, or reference to either)
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
                                // Handle nested arrays
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
                Err(e) => Err(format!("无法解引用对象: {:?}", e)),
            }
        }
        _ => Ok(Vec::new()),
    }
}

/// Get annotation rectangle [x1, y1, x2, y2]
fn get_annot_rect(annot_dict: &lopdf::Dictionary) -> Option<[f64; 4]> {
    if let Ok(lopdf::Object::Array(rect)) = annot_dict.get(b"Rect") {
        if rect.len() == 4 {
            let coords: Vec<f64> = rect.iter()
                .filter_map(|obj| {
                    match obj {
                        lopdf::Object::Integer(i) => Some(*i as f64),
                        lopdf::Object::Real(f) => Some(*f as f64),
                        _ => None,
                    }
                })
                .collect();
            if coords.len() == 4 {
                return Some([coords[0], coords[1], coords[2], coords[3]]);
            }
        }
    }
    None
}

fn flatten_page(doc: &mut lopdf::Document, page_id: lopdf::ObjectId) -> Result<(), String> {
    use lopdf::{Object, Stream};
    
    // First, collect all annotation data (immutable borrow)
    let annotation_data = {
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
            
            // Get annotation rectangle for positioning
            let rect = get_annot_rect(annot_dict);
            
            // Only process annotations that have appearance streams
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
                        
                        data.push((rect, stream_content));
                    }
                }
            }
        }
        
        data
    };
    
    // If we have appearance streams, merge them into page content
    if !annotation_data.is_empty() {
        // Get current content using the helper function
        let current_content = {
            let page = doc.get_object(page_id)
                .map_err(|e| format!("无法获取页面对象: {}", e))?;
            
            if let Object::Dictionary(dict) = page {
                match dict.get(b"Contents") {
                    Ok(contents) => {
                        get_content_from_object(doc, contents)?
                    }
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            }
        };
        
        // Build new content with appearance streams
        let mut new_content = current_content;
        
        for (rect, stream_content) in &annotation_data {
            new_content.extend_from_slice(b"\nq\n");
            
            // If we have a rectangle, apply translation to position the annotation
            if let Some([x1, y1, _x2, _y2]) = rect {
                // Translate to annotation position
                // Use 6 decimal places for precision
                let transform = format!("1 0 0 1 {:.6} {:.6} cm\n", x1, y1);
                new_content.extend_from_slice(transform.as_bytes());
            }
            
            new_content.extend_from_slice(stream_content);
            new_content.extend_from_slice(b"\nQ\n");
        }
        
        // Create new stream with the merged content
        let new_stream = Stream::new(lopdf::Dictionary::new(), new_content);
        let new_id = doc.add_object(new_stream);
        
        // Update page to reference the new content stream
        let page = doc.get_object_mut(page_id)
            .map_err(|e| format!("无法修改页面: {}", e))?;
        
        if let Object::Dictionary(dict) = page {
            dict.set(b"Contents", Object::Reference(new_id));
        }
    }
    
    // Always remove annotations from page (this is the flattening)
    let page = doc.get_object_mut(page_id)
        .map_err(|e| format!("无法修改页面: {}", e))?;
    
    if let Object::Dictionary(dict) = page {
        dict.remove(b"Annots");
    }
    
    Ok(())
}
