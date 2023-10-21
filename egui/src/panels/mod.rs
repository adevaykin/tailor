use egui::Ui;

pub mod session_settings;

#[derive(Default)]
pub struct Panels {
    pub session_settings: session_settings::SessionSettingsPanel,
}

impl Panels {
    pub fn draw(&mut self, ui: &mut Ui) {
        self.session_settings.show(ui);
    }
}
