use crate::app::ProcessingState;

pub struct ProgressView;

impl ProgressView {
    pub fn show(
        ui: &mut egui::Ui,
        progress: f32,
        current: usize,
        total: usize,
        state: &ProcessingState,
    ) {
        match state {
            ProcessingState::Idle => return,
            ProcessingState::Processing | ProcessingState::Done => {}
        }

        ui.add_space(4.0);

        // Progress bar
        let progress_bar = egui::ProgressBar::new(progress)
            .show_percentage()
            .text(if *state == ProcessingState::Done {
                "✅ 处理完成".to_string()
            } else {
                format!("处理中: {}/{}", current, total)
            });

        ui.add(progress_bar);
    }
}
