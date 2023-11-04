use std::path::{Path, PathBuf};
use app_dirs2::{AppDataType, get_app_root};
use egui::ComboBox;
use crate::APP_INFO;

const RECENTS_FILENAME: &str = "recents.json";

pub struct RecentsBox {
    recents: Vec<String>,
    selected_recent: String,
}

impl Default for RecentsBox {
    fn default() -> Self {
        Self {
            recents: Self::try_load_recent(),
            selected_recent: String::new(),
        }
    }
}

impl RecentsBox {
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ComboBox::from_label("🔁")
            .selected_text(&self.selected_recent)
            .width(300.0)
            .show_ui(ui, |ui| {
                for recent in &self.recents {
                    ui.selectable_value(&mut self.selected_recent, recent.clone(), recent);
                }
            });
    }

    pub fn is_dirty(&self, prev_path: &Path) -> bool {
        self.selected_recent != prev_path.display().to_string()
    }

    pub fn update_recents(&mut self, path: &Path) {
        self.recents = self.recents.clone().into_iter().filter(|recent| *recent != path.display().to_string()).collect();
        self.recents.insert(0, path.display().to_string());
        self.recents.truncate(10);
        self.selected_recent = path.display().to_string();
        Self::try_save_recents(&self.recents);
    }

    pub fn get_selected_recent_path(&self) -> PathBuf {
        PathBuf::from(&self.selected_recent)
    }

    fn try_load_recent() -> Vec<String> {
        if let Ok(data_path) = get_app_root(AppDataType::UserData, &APP_INFO) {
            if data_path.exists() {
                let recents_path = data_path.join(RECENTS_FILENAME);
                if let Ok(loaded_recents) = std::fs::read_to_string(&recents_path)
                {
                    if let Ok(recents) = serde_json::from_str(&loaded_recents) {
                        return recents;
                    }
                } else {
                    let recents: Vec<String> = vec![];
                    let recents_json = serde_json::to_string(&recents).unwrap();
                    let _ = std::fs::write(&recents_path, recents_json);
                }
            }
        }

        vec![]
    }

    fn try_save_recents(recents: &Vec<String>) {
        if let Ok(data_path) = get_app_root(AppDataType::UserData, &APP_INFO) {
            let recents_path = data_path.join(RECENTS_FILENAME);
            let recents_json = serde_json::to_string(recents).unwrap();
            let _ = std::fs::write(recents_path, recents_json);
        }
    }
}