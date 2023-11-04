use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::path::{Component, PathBuf};
use app_dirs2::{AppDataType, get_app_root};
use serde::{Deserialize, Serialize};
use crate::APP_INFO;
use crate::highlight::{Colors, Highlight};

fn default_highlights() -> Vec<Highlight> {
    vec![
        Highlight::new(String::from("ERROR"), Colors { foreground: [1.0, 1.0, 1.0], background: [0.75, 0.0, 0.0] }).unwrap(),
        Highlight::new(String::from("WARN"), Colors { foreground: [0.84, 0.66, 0.39], background: [0.0, 0.0, 0.0] }).unwrap(),
        Highlight::new(String::from("DEBUG"), Colors { foreground: [0.35, 0.76, 0.35], background: [0.0, 0.0, 0.0] }).unwrap(),
        Highlight::new(String::from("INFO"), Colors { foreground: [0.53, 0.53, 0.86], background: [0.0, 0.0, 0.0] }).unwrap(),
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
        let default_path = PathBuf::from("");
        if let Some(loaded) = Self::try_load(&default_path)
        {
            return loaded;
        }

        Self {
            path: PathBuf::new(),
            colors: Colors::default(),
            highlights: default_highlights(),
        }
    }
}

impl Session {
    pub fn new(path: PathBuf) -> Self {
        if let Some(loaded) = Self::try_load(&path)
        {
            return loaded;
        }

        Self {
            path,
            colors: Colors::default(),
            highlights: default_highlights(),
        }
    }

    pub fn save(&self) {
        if let Ok(session_save_path) = Self::get_save_path(&self.path) {
            let session_json = serde_json::to_string(&self).unwrap();
            let _ = std::fs::write(session_save_path, session_json);
        }
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

    pub fn remove_highlight(&mut self, index: usize) {
        let _ = self.highlights.remove(index);
    }

    pub fn get_highlight(&self, line: &str) -> &Colors {
        for highlight in &self.highlights {
            if highlight.is_matching(line) {
                return highlight.get_colors();
            }
        }

        &self.colors
    }

    fn get_save_path(path: &PathBuf) -> Result<PathBuf, Box<dyn Error>> {
        let data_path = get_app_root(AppDataType::UserData, &APP_INFO)?;
        if path.display().to_string().is_empty() {
            return Ok(data_path.join("default.json"))
        }

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
        Ok(data_path.join(session_save_name))
    }

    fn try_load(path: &PathBuf) -> Option<Session> {
        if let Ok(session_save_path) = Self::get_save_path(path) {
            if let Ok(loaded_session_json) = std::fs::read_to_string(session_save_path)
            {
                if let Ok(loaded_session) = serde_json::from_str(&loaded_session_json) {
                    return Some(loaded_session);
                }
            }
        }

        None
    }
}
