use crate::pdf::{flatten_pdf, get_pdf_info, PdfInfo};
use crate::ui::{FileList, ProgressView};

#[derive(PartialEq)]
pub enum ProcessingState {
    Idle,
    Processing,
    Done,
}

pub struct PdfFlattenApp {
    files: Vec<PdfInfo>,
    keep_original: bool,
    open_output_dir: bool,
    state: ProcessingState,
    current_file: usize,
    total_files: usize,
    progress: f32,
    error_message: Option<String>,
}

impl PdfFlattenApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure fonts with Chinese support
        let mut fonts = egui::FontDefinitions::default();
        
        // Try to load system Chinese fonts
        #[cfg(target_os = "windows")]
        {
            // On Windows, try to load Microsoft YaHei or SimSun
            let font_paths = [
                "C:/Windows/Fonts/msyh.ttc",      // Microsoft YaHei
                "C:/Windows/Fonts/simsun.ttc",     // SimSun
                "C:/Windows/Fonts/simhei.ttf",     // SimHei
            ];
            
            for path in &font_paths {
                if let Ok(font_data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "chinese".to_owned(),
                        egui::FontData::from_owned(font_data),
                    );
                    fonts.families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, "chinese".to_owned());
                    fonts.families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .push("chinese".to_owned());
                    break;
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            // On macOS, try to load PingFang or Hiragino Sans
            let font_paths = [
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/STHeiti Light.ttc",
                "/System/Library/Fonts/Hiragino Sans GB.ttc",
            ];
            
            for path in &font_paths {
                if let Ok(font_data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "chinese".to_owned(),
                        egui::FontData::from_owned(font_data),
                    );
                    fonts.families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, "chinese".to_owned());
                    fonts.families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .push("chinese".to_owned());
                    break;
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            // On Linux, try common CJK font locations
            let font_paths = [
                "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
                "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            ];
            
            for path in &font_paths {
                if let Ok(font_data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "chinese".to_owned(),
                        egui::FontData::from_owned(font_data),
                    );
                    fonts.families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, "chinese".to_owned());
                    fonts.families
                        .entry(egui::FontFamily::Monospace)
                        .or_default()
                        .push("chinese".to_owned());
                    break;
                }
            }
        }
        
        cc.egui_ctx.set_fonts(fonts);
        
        Self {
            files: Vec::new(),
            keep_original: true,
            open_output_dir: true,
            state: ProcessingState::Idle,
            current_file: 0,
            total_files: 0,
            progress: 0.0,
            error_message: None,
        }
    }

    fn add_files(&mut self, paths: Vec<std::path::PathBuf>) {
        for path in paths {
            if path.extension().map_or(false, |ext| ext == "pdf") {
                match get_pdf_info(&path) {
                    Ok(info) => {
                        if !self.files.iter().any(|f| f.path == path) {
                            self.files.push(info);
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("无法读取 {}: {}", path.display(), e));
                    }
                }
            }
        }
    }

    fn process_files(&mut self) {
        if self.files.is_empty() {
            return;
        }

        self.state = ProcessingState::Processing;
        self.total_files = self.files.len();
        self.current_file = 0;
        self.progress = 0.0;
        self.error_message = None;

        // Process files sequentially
        for (i, file) in self.files.iter_mut().enumerate() {
            self.current_file = i + 1;
            self.progress = (i as f32) / (self.total_files as f32);

            match flatten_pdf(&file.path, self.keep_original) {
                Ok(_) => {
                    file.status = "✅".to_string();
                }
                Err(e) => {
                    file.status = "❌".to_string();
                    self.error_message = Some(format!("处理 {} 时出错: {}", file.name, e));
                }
            }
        }

        self.progress = 1.0;
        self.state = ProcessingState::Done;

        // Open output directory if requested
        if self.open_output_dir {
            if let Some(first) = self.files.first() {
                if let Some(parent) = first.path.parent() {
                    let _ = open::that(parent);
                }
            }
        }
    }

    fn clear_list(&mut self) {
        self.files.clear();
        self.state = ProcessingState::Idle;
        self.progress = 0.0;
        self.error_message = None;
    }
}

impl eframe::App for PdfFlattenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Title
            ui.heading("PDF 注释扁平化工具");
            ui.add_space(8.0);

            // Drop zone
            let drop_zone = ui.vertical_centered(|ui| {
                let response = ui.allocate_response(
                    egui::vec2(ui.available_width(), 60.0),
                    egui::Sense::click(),
                );

                let rect = response.rect;
                let bg_color = if response.hovered() {
                    egui::Color32::from_rgb(230, 240, 255)
                } else {
                    egui::Color32::from_rgb(245, 245, 250)
                };

                ui.painter().rect_filled(rect, 4.0, bg_color);
                ui.painter().rect_stroke(
                    rect,
                    4.0,
                    egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(180, 180, 200)),
                );

                ui.put(
                    rect,
                    egui::Label::new(
                        egui::RichText::new("拖拽 PDF 文件到此处，或点击选择文件")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(100, 100, 120)),
                    ),
                );

                if response.clicked() {
                    let files = rfd::FileDialog::new()
                        .add_filter("PDF", &["pdf"])
                        .pick_files();
                    if let Some(files) = files {
                        return Some(files);
                    }
                }

                // Handle dropped files
                let dropped = ctx.input(|i| i.raw.dropped_files.clone());
                if !dropped.is_empty() {
                    let paths: Vec<_> = dropped.into_iter().filter_map(|f| f.path).collect();
                    if !paths.is_empty() {
                        return Some(paths);
                    }
                }

                None
            });

            if let Some(paths) = drop_zone.inner {
                self.add_files(paths);
            }

            ui.add_space(8.0);

            // File list
            FileList::show(ui, &mut self.files);

            ui.add_space(8.0);

            // Options
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.keep_original, "保留原始文件");
                ui.checkbox(&mut self.open_output_dir, "处理后打开输出目录");
            });

            ui.add_space(8.0);

            // Buttons
            ui.horizontal(|ui| {
                let can_process = !self.files.is_empty()
                    && self.state != ProcessingState::Processing;

                if ui
                    .add_enabled(can_process, egui::Button::new("开始处理"))
                    .clicked()
                {
                    self.process_files();
                }

                if ui.button("选择文件").clicked() {
                    let files = rfd::FileDialog::new()
                        .add_filter("PDF", &["pdf"])
                        .pick_files();
                    if let Some(files) = files {
                        self.add_files(files);
                    }
                }

                if ui.button("清除列表").clicked() {
                    self.clear_list();
                }
            });

            ui.add_space(8.0);

            // Progress
            ProgressView::show(
                ui,
                self.progress,
                self.current_file,
                self.total_files,
                &self.state,
            );

            // Error message
            if let Some(err) = &self.error_message {
                ui.add_space(8.0);
                ui.colored_label(egui::Color32::RED, err);
            }
        });

        // Request repaint during processing
        if self.state == ProcessingState::Processing {
            ctx.request_repaint();
        }
    }
}
