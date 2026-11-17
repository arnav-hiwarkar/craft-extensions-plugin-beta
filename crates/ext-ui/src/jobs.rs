use craft_ext_core::id::ExtensionId;
use craft_ext_host::jobs::{JobBoard, JobInfo, Status};
use std::time::Duration;

const REPAINT: Duration = Duration::from_millis(100);

pub fn status_text(job: &JobInfo, tr: &dyn Fn(&str) -> String) -> String {
    match job.status() {
        Status::Working => tr("Working…"),
        Status::Note(note) => note.to_owned(),
        Status::Done => tr("Done"),
        Status::Failed(why) => format!("{}: {why}", tr("Failed")),
        Status::Cancelled => tr("Cancelled"),
    }
}

pub fn jobs_window(ctx: &egui::Context, board: &JobBoard, name: &dyn Fn(&ExtensionId) -> String, tr: &dyn Fn(&str) -> String) {
    let jobs = board.list();
    if jobs.is_empty() {
        return;
    }
    if jobs.iter().any(|j| !j.state.finished()) {
        ctx.request_repaint_after(REPAINT);
    }
    egui::Window::new(tr("Plugin jobs"))
        .id(egui::Id::unique("craft-plugin-jobs"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-12.0, -36.0))
        .default_open(true)
        .resizable(false)
        .show(ctx, |ui| {
            for job in &jobs {
                ui.horizontal(|ui| {
                    ui.strong(&job.label);
                    ui.weak(name(&job.plugin));
                });
                ui.horizontal(|ui| {
                    if !job.state.finished() {
                        match job.progress {
                            Some(p) => {
                                ui.add(egui::ProgressBar::new(p).desired_width(140.0).show_percentage());
                            }
                            None => {
                                ui.spinner();
                            }
                        }
                    }
                    ui.label(status_text(job, tr));
                    if !job.state.finished() && ui.small_button(tr("Cancel")).clicked() {
                        board.stop(job.id);
                    }
                });
            }
            if jobs.iter().any(|j| j.state.finished()) && ui.button(tr("Clear finished")).clicked() {
                board.clear_finished();
            }
        });
}
