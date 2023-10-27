use egui::color_picker::color_edit_button_rgb;
use egui::{TextEdit, Ui};
use crate::session::Session;

#[derive(Default)]
pub struct SessionSettingsPanel {
    is_visible: bool,
}

impl SessionSettingsPanel {
    pub fn toggle_is_visible(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn get_is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn show(&mut self, ui: &mut Ui, session: &mut Session) {
        if !self.is_visible {
            return;
        }

        egui::SidePanel::right("session_settings")
            .resizable(true)
            .default_width(250.0)
            .width_range(120.0..=250.0)
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Session");
                });
                ui.horizontal(|ui| {
                    ui.label("Text:");
                    color_edit_button_rgb(ui, &mut session.get_colors().foreground);
                    ui.label("BackgroundS:");
                    color_edit_button_rgb(ui, &mut session.get_colors().background);
                });
                ui.separator();
                for highlight in session.get_highlights() {
                    ui.horizontal(|ui| {
                        let pattern_edit = TextEdit::singleline(highlight.get_pattern())
                            .hint_text("Regex Pattern")
                            .desired_width(145.0);
                        if ui.add(pattern_edit).changed() {
                            highlight.update_regex().unwrap();
                        }
                        color_edit_button_rgb(ui, &mut highlight.get_colors().foreground);
                        color_edit_button_rgb(ui, &mut highlight.get_colors().background);
                    });
                }
            });
    }
}