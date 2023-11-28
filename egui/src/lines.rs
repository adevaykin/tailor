use std::collections::HashSet;

pub struct LinesState {
    lines: Vec<String>,
    filtered_lines: Vec<(String, u32)>,
    /// (line, id)
    selected_lines: HashSet<usize>,
    is_dirty: bool,
}

impl LinesState {
    pub fn new() -> Self {
        Self {
            lines: vec![],
            filtered_lines: vec![],
            selected_lines: HashSet::new(),
            is_dirty: true,
        }
    }

    pub fn add_lines(&mut self, lines: Vec<String>) {
        self.lines.extend(lines);
        self.is_dirty = true;
    }

    pub fn clear_lines(&mut self) {
        self.lines.clear();
        self.selected_lines.clear();
        self.is_dirty = true;
    }

    pub fn toggle_single_line_selection(&mut self, idx: usize) {
        if self.selected_lines.contains(&idx) {
            if self.selected_lines.len() > 1 {
                self.selected_lines.clear();
                self.selected_lines.insert(idx);
            } else {
                self.selected_lines.clear();
            }
        } else {
            self.selected_lines.clear();
            self.selected_lines.insert(idx);
        }
    }

    pub fn toggle_add_selection(&mut self, idx: usize) {
        if self.selected_lines.contains(&idx) {
            self.selected_lines.remove(&idx);
        } else {
            self.selected_lines.insert(idx);
        }
    }

    pub fn toggle_add_range_selection(&mut self, idx: usize) {
        if self.selected_lines.is_empty() {
            self.selected_lines.insert(idx);
        } else {
            let min_idx = if let Some(min) = self.selected_lines.iter().min() {
                *min
            } else {
                idx
            };
            let max_idx = if let Some(max) = self.selected_lines.iter().max() {
                *max
            } else {
                idx
            };
            if idx < min_idx {
                for i in idx..max_idx {
                    self.selected_lines.insert(i);
                }
            }
            if idx > max_idx {
                for i in min_idx..=idx {
                    self.selected_lines.insert(i);
                }
            }
        }
    }

    pub fn is_selected(&self, idx: usize) -> bool {
        self.selected_lines.contains(&idx)
    }

    pub fn get_selected_text(&self) -> String {
        self.selected_lines
            .iter()
            .map(|idx| self.lines[*idx].clone())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn get_filtered_lines(&mut self, pattern: &String) -> &Vec<(String, u32)> {
        if self.is_dirty {
            if pattern.is_empty() {
                self.filtered_lines = self
                    .lines
                    .iter()
                    .enumerate()
                    .map(|(idx, line)| (line.clone(), idx as u32))
                    .collect();
            } else {
                self.filtered_lines = self
                    .lines
                    .iter()
                    .enumerate()
                    .map(|(idx, line)| (line.clone(), idx as u32))
                    .filter(|(line, _)| line.contains(pattern))
                    .collect();
            }
        }

        &self.filtered_lines
    }
}
