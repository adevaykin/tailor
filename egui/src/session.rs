use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Component, PathBuf};
use app_dirs2::{AppDataType, get_app_root};
use serde::{Deserialize, Serialize};
use crate::APP_INFO;
use crate::highlight::{Colors, Highlight};

fn default_highlights() -> Vec<Highlight> {
    vec![
        Highlight::new(String::from("ERROR"), Colors { foreground: [1.0, 0.0, 0.0], background: [0.0, 0.0, 0.0] }).unwrap(),
        Highlight::new(String::from("WARN"), Colors { foreground: [255.0, 255.0, 0.0], background: [0.0, 0.0, 0.0] }).unwrap(),
        Highlight::new(String::from("DEBUG"), Colors { foreground: [0.0, 255.0, 0.0], background: [0.0, 0.0, 0.0] }).unwrap(),
        Highlight::new(String::from("INFO"), Colors { foreground: [0.0, 0.0, 255.0], background: [0.0, 0.0, 0.0] }).unwrap(),
    ]
}

#[derive(Serialize,Deserialize)]
pub struct Session {
    path: PathBuf,
    colors: Colors,
    highlights: Vec<Highlight>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            colors: Colors::default(),
            highlights: default_highlights(),
        }
    }
}

impl Session {
    pub fn new(path: PathBuf) -> Self {
        Self::try_load(&path);
        Self {
            path,
            colors: Colors::default(),
            highlights: default_highlights(),
        }
    }

    fn try_load(path: &PathBuf) -> Option<Session> {
        if let Ok(data_path) = get_app_root(AppDataType::UserData, &APP_INFO) {
            let mut hasher = DefaultHasher::new();
            path.hash(&mut hasher);
            let h = hasher.finish();
            let last_path_component = match path.components().next_back() {
                Some(Component::Normal(dir_name)) => {
                    if let Some(dir_name) = dir_name.to_str() {
                        dir_name.to_string()
                    } else {
                        "".to_string()
                    }
                },
                _ => "".to_string(),
            };
            let session_save_name = format!("{}_{}.json", h, last_path_component);
            let session_save_path = data_path.join(session_save_name);
            if let Ok(loaded_session_json) = std::fs::read_to_string(&session_save_path)
            {
                if let Ok(loaded_session) = serde_json::from_str(&loaded_session_json) {
                    return Some(loaded_session);
                }
            }
        }

        None
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn get_colors(&mut self) -> &mut Colors {
        &mut self.colors
    }

    pub fn get_highlights(&mut self) -> &mut Vec<Highlight> {
        &mut self.highlights
    }

    pub fn get_highlight(&self, line: &String) -> &Colors {
        for highlight in &self.highlights {
            if highlight.is_matching(line) {
                return highlight.get_colors();
            }
        }

        &self.colors
    }
}