use egui::color_picker::color_edit_button_rgb;
use egui::{TextEdit, Ui};
use crate::highlight::{Colors, Highlight};
use crate::session::Session;

#[derive(Default)]
pub struct SessionSettingsPanel {
    is_visible: bool,
    colors: Colors,
    highlights: Vec<Highlight>,
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
            session.save();
            return;
        }

        egui::SidePanel::right("session_settings")
            .resizable(true)
            .default_width(250.0)
            .width_range(250.0..=250.0)
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Session");
                });
                ui.horizontal(|ui| {
                    ui.label("Text:");
                    color_edit_button_rgb(ui, &mut session.get_colors().foreground);
                    ui.label("Background:");
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
                        color_edit_button_rgb(ui, &mut highlight.get_mut_colors().foreground);
                        color_edit_button_rgb(ui, &mut highlight.get_mut_colors().background);
                    });
                }
                if ui.button("+").clicked() {
                    session.get_highlights().push(Highlight::default());
                }

                if ui.button("Save").clicked() {
                    session.save();
                }
            });
    }
}