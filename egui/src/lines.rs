pub struct LinesState {
    lines: Vec<String>,
    filtered_lines: Vec<(String,u32)>, /// (line, id)
    selected_lines: Vec<bool>,
    is_dirty: bool,
}

impl LinesState {
    pub fn new() -> Self {
        Self {
            lines: vec![],
            filtered_lines: vec![],
            selected_lines: vec![],
            is_dirty: true,
        }
    }

    pub fn add_lines(&mut self, lines: Vec<String>) {
        self.lines.extend(lines);
        self.selected_lines.resize(self.lines.len(), false);
        self.is_dirty = true;
    }

    pub fn clear_lines(&mut self) {
        self.lines.clear();
        self.selected_lines.clear();
        self.is_dirty = true;
    }

    pub fn toggle_line_selection(&mut self, idx: usize) {
        self.selected_lines[idx] = !self.selected_lines[idx];
        log::info!("{} {}", idx, self.selected_lines[idx]);
    }

    pub fn is_selected(&self, idx: usize) -> bool {
        self.selected_lines[idx]
    }

    pub fn get_filtered_lines(&mut self, pattern: &String) -> &Vec<(String,u32)> {
        if self.is_dirty {
            if pattern.is_empty() {
                self.filtered_lines = self.lines
                    .iter()
                    .enumerate()
                    .map(|(idx,line)| (line.clone(), idx as u32)).collect();
            } else {
                self.filtered_lines = self.lines
                    .iter()
                    .enumerate()
                    .map(|(idx,line)| (line.clone(), idx as u32))
                    .filter(|(line,_)| line.contains(pattern))
                    .collect();
            }
        }

        &self.filtered_lines
    }
}