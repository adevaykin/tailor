use egui::Color32;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize,Deserialize)]
pub struct Colors {
    pub foreground: [f32; 3],
    pub background: [f32; 3],
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            foreground: [1.0, 1.0, 1.0],
            background: [0.0, 0.0, 0.0],
        }
    }
}

impl Colors {
    pub fn foreground(&self) -> Color32 {
        Color32::from_rgb((self.foreground[0]*255.0) as u8, (self.foreground[1]*255.0) as u8, (self.foreground[2]*255.0) as u8)
    }

    pub fn background(&self) -> Color32 {
        Color32::from_rgb((self.background[0]*255.0) as u8, (self.background[1]*255.0) as u8, (self.background[2]*255.0) as u8)
    }
}

struct SerializableRegex {
    regex: Regex,
}

impl SerializableRegex {
    pub fn new(regex: Regex) -> Self {
        Self {
            regex,
        }
    }
}

impl Serialize for SerializableRegex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.regex.to_string())
    }
}

impl<'de> Deserialize<'de> for SerializableRegex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self {
            regex: Regex::new(&s).unwrap_or(Regex::new("").unwrap()),
        })
    }
}

#[derive(Serialize,Deserialize)]
pub struct Highlight {
    pattern: String,
    regex: SerializableRegex,
    colors: Colors,
}

impl Default for Highlight {
    fn default() -> Self {
        Self {
            pattern: String::from(""),
            regex: SerializableRegex::new(Regex::new("").unwrap()),
            colors: Colors::default(),
        }
    }

}

impl Highlight {
    pub fn new(pattern: String, colors: Colors) -> Result<Self, String> {
        if let Ok(regex) = Regex::new(format!(r"(?i){}", pattern.as_str()).as_str()) {
            return Ok(Self {
                pattern,
                regex: SerializableRegex::new(regex),
                colors,
            });
        }

        Err("Failed to create Highlight".into())
    }
    pub fn is_matching(&self, line: &str) -> bool {
        self.regex.regex.is_match(line)
    }

    pub fn get_mut_colors(&mut self) -> &mut Colors {
        &mut self.colors
    }

    pub fn get_colors(&self) -> &Colors {
        &self.colors
    }

    pub fn get_pattern(&mut self) -> &mut String {
        &mut self.pattern
    }

    pub fn update_regex(&mut self) -> Result<(), ()> {
        if let Ok(regex) = Regex::new(format!(r"(?i){}", self.pattern.as_str()).as_str()) {
            self.regex = SerializableRegex::new(regex);
            return Ok(());
        }

        Err(())
    }
}