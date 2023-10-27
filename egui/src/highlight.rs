use egui::Color32;
use regex::Regex;

pub struct Colors {
    pub foreground: [f32; 3],
    pub background: [f32; 3],
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
        Color32::from_rgb((self.foreground[0]*255.0) as u8, (self.foreground[1]*255.0) as u8, (self.foreground[2]*255.0) as u8)
    }

    pub fn background(&self) -> Color32 {
        Color32::from_rgb((self.background[0]*255.0) as u8, (self.background[1]*255.0) as u8, (self.background[2]*255.0) as u8)
    }
}

pub struct Highlight {
    pattern: String,
    regex: Regex,
    colors: Colors,
}

impl Default for Highlight {
    fn default() -> Self {
        Self {
            pattern: String::from(""),
            regex: Regex::new("").unwrap(),
            colors: Colors::default(),
        }
    }

}

impl Highlight {
    pub fn new(pattern: String, colors: Colors) -> Result<Self, String> {
        if let Ok(regex) = Regex::new(format!("{}", pattern).as_str()) {
            return Ok(Self {
                pattern,
                regex,
                colors,
            });
        }

        Err("Failed to create Highlight".into())
    }
    pub fn is_matching(&self, line: &String) -> bool {
        self.regex.is_match(line)
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
        if let Ok(regex) = Regex::new(format!("{}", self.pattern).as_str()) {
            self.regex = regex;
            return Ok(());
        }

        Err(())
    }
}