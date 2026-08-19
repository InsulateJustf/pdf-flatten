use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq)]
pub enum Language {
    Chinese,
    English,
}

fn detect_language() -> Language {
    // Method 1: Check environment variables (Unix/Linux/macOS)
    let env_vars = ["LANG", "LC_ALL", "LC_MESSAGES", "LANGUAGE"];
    for var in &env_vars {
        if let Ok(val) = std::env::var(var) {
            if val.starts_with("zh") || val.contains("CN") || val.contains("TW") || val.contains("HK") {
                return Language::Chinese;
            }
            if !val.is_empty() && !val.starts_with("zh") {
                return Language::English;
            }
        }
    }
    
    // Method 2: Windows-specific detection
    #[cfg(target_os = "windows")]
    {
        // Check PreferredUILanguages
        if let Ok(ui_lang) = std::env::var("PreferredUILanguages") {
            if ui_lang.to_lowercase().starts_with("zh") {
                return Language::Chinese;
            }
        }
        
        // Check common Windows environment variables
        for var in &["UI_LANGUAGE", "MUI", "SystemLocale"] {
            if let Ok(val) = std::env::var(var) {
                if val.to_lowercase().contains("zh") || val.to_lowercase().contains("chinese") {
                    return Language::Chinese;
                }
            }
        }
        
        // Try to use sys_locale crate if available, or fallback to checking
        // the user's default locale via a simple heuristic
        if let Ok(locale) = sys_locale::get_locale() {
            if locale.to_lowercase().starts_with("zh") {
                return Language::Chinese;
            }
        }
    }
    
    // Default to English
    Language::English
}

static LANG: LazyLock<Language> = LazyLock::new(detect_language);

pub fn lang() -> Language {
    *LANG
}

pub fn is_chinese() -> bool {
    *LANG == Language::Chinese
}

// UI strings
pub fn title() -> &'static str {
    if is_chinese() { "PDF 注释扁平化工具" } else { "PDF Annotation Flattener" }
}

pub fn drop_hint() -> &'static str {
    if is_chinese() { "拖拽 PDF 文件到此处，或点击选择文件" } else { "Drop PDF files here, or click to select" }
}

pub fn no_files() -> &'static str {
    if is_chinese() { "暂无文件，请添加 PDF 文件" } else { "No files added. Please add PDF files." }
}

pub fn col_filename() -> &'static str {
    if is_chinese() { "文件名" } else { "Filename" }
}

pub fn col_pages() -> &'static str {
    if is_chinese() { "页数" } else { "Pages" }
}

pub fn col_annotations() -> &'static str {
    if is_chinese() { "注释" } else { "Annotations" }
}

pub fn col_status() -> &'static str {
    if is_chinese() { "状态" } else { "Status" }
}

pub fn col_action() -> &'static str {
    if is_chinese() { "操作" } else { "Action" }
}

pub fn keep_original() -> &'static str {
    if is_chinese() { "保留原始文件" } else { "Keep original files" }
}

pub fn open_output_dir() -> &'static str {
    if is_chinese() { "处理后打开输出目录" } else { "Open output directory after processing" }
}

pub fn btn_start() -> &'static str {
    if is_chinese() { "开始处理" } else { "Start Processing" }
}

pub fn btn_add_files() -> &'static str {
    if is_chinese() { "选择文件" } else { "Add Files" }
}

pub fn btn_clear() -> &'static str {
    if is_chinese() { "清除列表" } else { "Clear List" }
}

pub fn status_pending() -> &'static str {
    if is_chinese() { "等待中" } else { "Pending" }
}

pub fn status_ok() -> &'static str {
    if is_chinese() { "完成" } else { "OK" }
}

pub fn status_error() -> &'static str {
    if is_chinese() { "错误" } else { "ERR" }
}

pub fn processing_done() -> &'static str {
    if is_chinese() { "处理完成" } else { "Processing complete" }
}

pub fn processing(current: usize, total: usize) -> String {
    if is_chinese() {
        format!("处理中: {}/{}", current, total)
    } else {
        format!("Processing: {}/{}", current, total)
    }
}

// Error messages
pub fn err_load_pdf(e: &str) -> String {
    if is_chinese() {
        format!("无法加载 PDF: {}", e)
    } else {
        format!("Failed to load PDF: {}", e)
    }
}

pub fn err_save_pdf(e: &str) -> String {
    if is_chinese() {
        format!("无法保存 PDF: {}", e)
    } else {
        format!("Failed to save PDF: {}", e)
    }
}

pub fn err_process_page(page: u32, e: &str) -> String {
    if is_chinese() {
        format!("处理第 {} 页时出错: {}", page, e)
    } else {
        format!("Error processing page {}: {}", page, e)
    }
}

pub fn err_read_file(path: &str, e: &str) -> String {
    if is_chinese() {
        format!("无法读取 {}: {}", path, e)
    } else {
        format!("Failed to read {}: {}", path, e)
    }
}

pub fn err_process_file(name: &str, e: &str) -> String {
    if is_chinese() {
        format!("处理 {} 时出错: {}", name, e)
    } else {
        format!("Error processing {}: {}", name, e)
    }
}

pub fn err_get_page(e: &str) -> String {
    if is_chinese() {
        format!("无法获取页面对象: {}", e)
    } else {
        format!("Failed to get page object: {}", e)
    }
}

pub fn err_modify_page(e: &str) -> String {
    if is_chinese() {
        format!("无法修改页面: {}", e)
    } else {
        format!("Failed to modify page: {}", e)
    }
}

pub fn err_deref_object(e: &str) -> String {
    if is_chinese() {
        format!("无法解引用对象: {}", e)
    } else {
        format!("Failed to dereference object: {}", e)
    }
}
