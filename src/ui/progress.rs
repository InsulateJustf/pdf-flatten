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

        let progress_bar = egui::ProgressBar::new(progress)
            .show_percentage()
            .text(if *state == ProcessingState::Done {
                "Processing complete".to_string()
            } else {
                format!("Processing: {}/{}", current, total)
            });

        ui.add(progress_bar);
    }
}
