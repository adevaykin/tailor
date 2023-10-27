use egui::Ui;
use crate::session::Session;

pub mod session_settings;

#[derive(Default)]
pub struct Panels {
    pub session_settings: session_settings::SessionSettingsPanel,
}

impl Panels {
    pub fn draw(&mut self, ui: &mut Ui, session: &mut Session) {
        self.session_settings.show(ui, session );
    }
}
