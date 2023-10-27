#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod windows;
mod panels;
mod session;

use tailor::{Tailor, Message};
use windows::Windows;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use eframe::{App, egui, Frame};
use egui::{CentralPanel, Color32, ComboBox, Context, FontId, TopBottomPanel, Label, TextEdit, Button};
use egui::text::LayoutJob;
use egui_file::FileDialog;
use crate::panels::Panels;
use crate::session::{Highlight, Session};

struct TailorApp {
    panels: Panels,
    windows: Windows,
    session: Session,
    is_dirty: bool,
    open_file_dialog: Option<FileDialog>,
    current_file: String,
    recents: Vec<String>,
    next_open_file: Option<PathBuf>,
    tailor: Tailor,
    message_rx: Option<Receiver<Message>>,
    lines: Vec<String>,
    filter_text: String,
    search_text: String,
}

impl TailorApp {
    fn new(tailor: Tailor) -> Self {
        let ret = Self {
            panels: Panels::default(),
            windows: Windows::default(),
            session: Session::default(),
            is_dirty: false,
            open_file_dialog: None,
            current_file: String::new(),
            recents: vec![String::from("Test 1"), String::from("Test 2"), String::from("Test 3")],
            next_open_file: None,
            tailor,
            message_rx: None,
            lines: vec![],
            filter_text: String::new(),
            search_text: String::new(),
        };

        ret
    }

    fn get_line_color(highlights: &Vec<Highlight>, msg: &String) -> Color32 {
        if msg.contains("DEBUG") || msg.contains("debug") {
            return Color32::GREEN;
        }

        if msg.contains("WARNING") || msg.contains("WARN") || msg.contains("warning") {
            return Color32::YELLOW;
        }

        if msg.contains("ERROR") || msg.contains("ERR") || msg.contains("error") || msg.contains("Error") {
            return Color32::RED;
        }

        Color32::BLACK
    }
}

impl App for TailorApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if self.is_dirty {
            if let Some(path) = &self.next_open_file {
                let (tx, rx) = channel();
                self.message_rx = Some(rx);
                self.tailor.watch(path.clone(), tx);
                self.lines.clear();
                self.session = Session::new(path.clone());
                if let Some(path_str) = path.to_str() {
                    self.current_file = String::from(path_str);
                }
            }
            self.is_dirty = false;
        }

        if let Some(message_rx) = self.message_rx.as_ref() {
            if let Ok(msg) = message_rx.try_recv() {
                match msg {
                    Message::NewLines(lines) => {
                        self.lines.reserve(self.lines.len() + lines.len()*2);
                        for line in lines {
                            self.lines.push(line.clone());
                        }
                    },
                    Message::NewFile(_path) => {
                        self.lines.clear();
                    }
                }
            }
        }

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Open");
                if (ui.button("📃 File")).clicked() {
                    let mut dialog = FileDialog::open_file(Some(PathBuf::from("../")));
                    dialog.open();
                    self.open_file_dialog = Some(dialog);
                }
                if (ui.button("📂 Folder")).clicked() {
                    let mut dialog = FileDialog::select_folder(Some(PathBuf::from("../")));
                    dialog.open();
                    self.open_file_dialog = Some(dialog);
                }

                ComboBox::from_label("Recent")
                    .selected_text(format!("{}", "test"))
                    .show_ui(ui, |ui| {
                        for recent in &self.recents {
                            ui.selectable_value(&mut self.current_file, recent.clone(), recent);
                        }
                    });

                let session_settings_button = Button::new("🎨")
                    .selected(self.panels.session_settings.get_is_visible());
                if ui.add(session_settings_button).clicked() {
                    self.panels.session_settings.toggle_is_visible();
                }

                let about_button = Button::new("About")
                    .selected(self.windows.about.get_is_visible());
                if ui.add(about_button).clicked() {
                    self.windows.about.toggle_is_visible();
                }
            });

            if let Some(dialog) = &mut self.open_file_dialog {
                if dialog.show(ctx).selected() {
                    if let Some(file) = dialog.path() {
                        self.next_open_file = Some(PathBuf::from(file));
                        self.is_dirty = true;
                    }
                }
            }
        });

        CentralPanel::default().show(ctx, |ui| {
            self.panels.draw(ui, &mut self.session);

            egui::ScrollArea::both().show_rows(ui, 12.0, self.lines.len(),
   |ui, row_range| {
                    for row in row_range {
                        let layout_job = LayoutJob::simple(
                            self.lines[row].clone(),
                            FontId::monospace(12.0),
                            self.session.get_highlight(&self.lines[row]).foreground(),
                            0.0);
                        let line_label = Label::new(layout_job).wrap(false);
                        ui.add(line_label);
                    }
                    ui.add(Label::new(""));
            });
        });

        self.windows.draw(ctx);

        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(self.current_file.as_str());
                ui.add(TextEdit::singleline(&mut self.filter_text).hint_text("Filter").desired_width(120.0));
                ui.add(TextEdit::singleline(&mut self.search_text).hint_text("Search").desired_width(120.0));
            });
        });
    }
}

fn main() {
    env_logger::init();

    match Tailor::new() {
        Ok(tailor) => {
            let _ = eframe::run_native(
                "Tailor",
                eframe::NativeOptions::default(),
                Box::new(|_cc| Box::new(TailorApp::new(tailor))),
            );
        },
        Err(msg) => 
        {
            println!("Failed to create Tailor instance:");
            println!("{}", msg);
        }
    }
}
