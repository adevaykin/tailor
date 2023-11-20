use egui::{CentralPanel, Color32, Context, FontId, Label, TextFormat, Ui};
use egui::text::{LayoutJob, LayoutSection};
use regex::Regex;
use crate::lines::LinesState;
use crate::session::Session;

fn find_ranges(line: &str, regex: &Regex) -> Vec<(usize, usize)> {
    let captures = regex.find_iter(line);
    captures.map(|c| (c.start(), c.end())).collect()
}

fn fill_empty_ranges(ranges: Vec<(usize, usize)>, total_len: usize) -> Vec<(usize, usize, bool)> {
    let mut result = vec![];
    let mut last = 0;
    for (start, end) in ranges {
        if start > last {
            result.push((last, start, false));
        }
        result.push((start, end, true));
        last = end;
    }
    if total_len > last {
        result.push((last, total_len, false));
    }
    result
}

pub struct MainPanel {

}

impl MainPanel {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn draw(&mut self, session: &mut Session, ctx: &Context, log_contents: &mut LinesState,
    filter_text: &String, search_pattern: &Option<Regex>) {
        let frame = egui::containers::Frame {
            inner_margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
            outer_margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
            rounding: egui::Rounding { nw: 0.0, ne: 0.0, sw: 0.0, se: 0.0 },
            shadow: eframe::epaint::Shadow { extrusion: 0.0, color: Color32::BLACK },
            fill: session.get_colors().background(),
            stroke: egui::Stroke::new(0.0, Color32::BLACK),
        };
        CentralPanel::default().frame(frame).show(ctx, |ui| {
            let filtered_lines = log_contents.get_filtered_lines(filter_text);
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show_rows(ui, 12.0, filtered_lines.len(),
       |ui, row_range| {
                       for row in row_range {
                           let text_format = TextFormat {
                               background: session.get_highlight(&filtered_lines[row]).background(),
                               color: session.get_highlight(&filtered_lines[row]).foreground(),
                               font_id: FontId::monospace(12.0),
                               ..Default::default()
                           };
                           let inverted_text_format = TextFormat {
                               background: session.get_highlight(&filtered_lines[row]).foreground(),
                               color: session.get_highlight(&filtered_lines[row]).background(),
                               font_id: FontId::monospace(12.0),
                               ..Default::default()
                           };

                           let found_ranges = if let Some(regex) = search_pattern {
                               find_ranges(&filtered_lines[row], regex)
                           } else {
                               vec![]
                           };

                           let found_ranges = fill_empty_ranges(found_ranges, filtered_lines[row].len());
                           let mut layout_secions = vec![];
                           for (start, end, invert) in found_ranges {
                               let format = if invert { inverted_text_format.clone() } else { text_format.clone() };
                               layout_secions.push(LayoutSection {
                                   leading_space: 0.0,
                                   byte_range: start..end,
                                   format,
                               });
                           }

                           let layout_job = LayoutJob {
                               sections: layout_secions,
                               text: filtered_lines[row].clone(),
                               break_on_newline: false,
                               ..Default::default()
                           };
                           let line_label = Label::new(layout_job).wrap(false);
                           ui.add(line_label);
                       }
                       ui.add(Label::new(""));
               });
        });
    }
}