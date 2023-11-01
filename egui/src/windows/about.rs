use egui::{Color32, Context, FontId};
use egui::text::LayoutJob;

#[derive(Default)]
pub struct AboutWindow {
    is_visible: bool,
}

impl AboutWindow {
    pub fn get_is_visible(&self) -> bool {
        self.is_visible
    }
    pub fn toggle_is_visible(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn show(&mut self, ctx: &Context) {
        if !self.is_visible {
            return;
        }

        egui::Window::new("About")
            .collapsible(false)
            .resizable(true)
            .default_width(400.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let title = "Tailor";
        let title_job = LayoutJob::simple(
            title.to_owned(),
            FontId::proportional(24.0),
            Color32::BLACK,
            120.0
        );

        let text = "Dynamic log tail tool.\n\nPowered by Rust and egui.";
        let job = LayoutJob::single_section(
            text.to_owned(),
            egui::TextFormat {
                ..Default::default()
            },
        );

        ui.vertical_centered(|ui| {
            ui.label(title_job);
            ui.label(job);
        });
    }
}
