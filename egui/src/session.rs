use std::path::PathBuf;
use crate::highlight::{Colors, Highlight};

fn default_highlights() -> Vec<Highlight> {
    vec![
        Highlight::new(String::from("ERROR"), Colors { foreground: [1.0, 0.0, 0.0], background: [0.0, 0.0, 0.0] }).unwrap(),
        Highlight::new(String::from("WARN"), Colors { foreground: [255.0, 255.0, 0.0], background: [0.0, 0.0, 0.0] }).unwrap(),
        Highlight::new(String::from("DEBUG"), Colors { foreground: [0.0, 255.0, 0.0], background: [0.0, 0.0, 0.0] }).unwrap(),
        Highlight::new(String::from("INFO"), Colors { foreground: [0.0, 0.0, 255.0], background: [0.0, 0.0, 0.0] }).unwrap(),
    ]
}

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
        Self {
            path,
            colors: Colors::default(),
            highlights: default_highlights(),
        }
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