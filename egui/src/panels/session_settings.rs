use egui::color_picker::color_edit_button_rgb;
use egui::Ui;

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

    pub fn show(&mut self, ui: &mut Ui) {
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
                ui.vertical_centered(|ui| {
                    ui.label("Colors");
                });
                ui.label("Default:");
                egui::ScrollArea::vertical().show(ui, |ui| {

                });
                ui.label("Highlights:");
                let mut color: [f32; 3] = [0.0; 3];
                let picker = color_edit_button_rgb(ui, &mut color);
            });
    }
}