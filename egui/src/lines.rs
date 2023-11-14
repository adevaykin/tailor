pub struct LinesState {
    lines: Vec<String>,
    filtered_lines: Vec<String>,
    is_dirty: bool,
}

impl LinesState {
    pub fn new() -> Self {
        Self {
            lines: vec![],
            filtered_lines: vec![],
            is_dirty: true,
        }
    }

    pub fn add_lines(&mut self, lines: Vec<String>) {
        self.lines.extend(lines);
        self.is_dirty = true;
    }

    pub fn clear_lines(&mut self) {
        self.lines.clear();
        self.is_dirty = true;
    }

    pub fn get_filtered_lines(&mut self, pattern: &String) -> &Vec<String> {
        if self.is_dirty {
            if pattern.is_empty() {
                self.filtered_lines = self.lines.clone();
            } else {
                self.filtered_lines = self.lines
                    .clone()
                    .into_iter()
                    .filter(|line| line.contains(pattern))
                    .collect();
            }
        }

        &self.filtered_lines
    }
}