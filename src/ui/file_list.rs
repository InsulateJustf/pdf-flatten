use crate::pdf::PdfInfo;

pub struct FileList;

impl FileList {
    pub fn show(ui: &mut egui::Ui, files: &mut Vec<PdfInfo>) {
        egui::Frame::group(ui.style())
            .inner_margin(8.0)
            .show(ui, |ui| {
                if files.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new("暂无文件，请添加 PDF 文件")
                                .italics()
                                .color(egui::Color32::GRAY),
                        );
                    });
                    return;
                }

                // Header
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("#").strong());
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("文件名").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("操作").strong());
                        ui.add_space(30.0);
                        ui.label(egui::RichText::new("状态").strong());
                        ui.add_space(30.0);
                        ui.label(egui::RichText::new("注释").strong());
                        ui.add_space(30.0);
                        ui.label(egui::RichText::new("页数").strong());
                    });
                });

                ui.separator();

                // File rows
                let mut remove_idx = None;
                for (i, file) in files.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}", i + 1));
                        ui.add_space(4.0);
                        ui.label(&file.name);
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.small_button("✕").clicked() {
                                    remove_idx = Some(i);
                                }
                                ui.add_space(20.0);
                                ui.label(&file.status);
                                ui.add_space(20.0);
                                ui.label(format!("{}", file.annotations));
                                ui.add_space(20.0);
                                ui.label(format!("{}", file.pages));
                            },
                        );
                    });
                }

                if let Some(idx) = remove_idx {
                    files.remove(idx);
                }
            });
    }
}
