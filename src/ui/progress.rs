use crate::app::ProcessingState;
use crate::i18n;

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
                i18n::processing_done().to_string()
            } else {
                i18n::processing(current, total)
            });

        ui.add(progress_bar);
    }
}
