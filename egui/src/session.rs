use std::path::PathBuf;
use egui::Color32;
use regex::Regex;

pub struct Colors {
    foreground: [f32; 3],
    background: [f32; 3],
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            foreground: [0.0, 0.0, 0.0],
            background: [1.0, 1.0, 1.0],
        }
    }
}

impl Colors {
    pub fn foreground(&self) -> Color32 {
        
    }
}

pub struct Highlight {
    pattern: Regex,
    colors: Colors,
}

impl Highlight {
    pub fn new(pattern: String, colors: Colors) -> Result<Self, String> {
        if let Ok(pattern) = Regex::new(format!("({})", pattern).as_str()) {
            return Ok(Self {
                pattern,
                colors,
            });
        }

        Err("Failed to create Highlight".into())
    }
    pub fn is_matching(&self, line: &String) -> bool {
        self.pattern.is_match(line)
    }
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
            highlights: vec![
                Highlight::new(String::from("ERROR"), Colors{ foreground: [1.0, 0.0, 0.0], background: [0.0, 0.0, 0.0] }).unwrap(),
                Highlight::new(String::from("WARN"), Colors{ foreground: [1.0, 1.0, 0.0], background: [0.0, 0.0, 0.0] }).unwrap(),
                Highlight::new(String::from("DEBUG"), Colors{ foreground: [0.0, 1.0, 0.0], background: [0.0, 0.0, 0.0] }).unwrap(),
                Highlight::new(String::from("INFO"), Colors{ foreground: [0.0, 0.0, 1.0], background: [0.0, 0.0, 0.0] }).unwrap(),
            ],
        }
    }
}

impl Session {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            colors: Colors::default(),
            highlights: vec![
                Highlight::new(String::from("ERROR"), Colors{ foreground: [1.0, 0.0, 0.0], background: [0.0, 0.0, 0.0] }).unwrap(),
                Highlight::new(String::from("WARN"), Colors{ foreground: [1.0, 1.0, 0.0], background: [0.0, 0.0, 0.0] }).unwrap(),
                Highlight::new(String::from("DEBUG"), Colors{ foreground: [0.0, 1.0, 0.0], background: [0.0, 0.0, 0.0] }).unwrap(),
                Highlight::new(String::from("INFO"), Colors{ foreground: [0.0, 0.0, 1.0], background: [0.0, 0.0, 0.0] }).unwrap(),
            ],
        }
    }

    pub fn get_colors(&self, line: &String) -> &Colors {
        for highlight in &self.highlights {
            if highlight.is_matching(line) {
                return &highlight.colors;
            }
        }

        &self.colors
    }
}