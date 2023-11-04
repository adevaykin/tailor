#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod windows;
mod panels;
mod session;
mod highlight;
mod widgets;

use app_dirs2::*; // or app_dirs::* if you've used package alias in Cargo.toml

const APP_INFO: AppInfo = AppInfo{name: "Tailor", author: "Alexander Devaikin"};

use tailor::{Tailor, Message};
use windows::Windows;
use std::path::{PathBuf};
use std::sync::mpsc::{channel, Receiver};
use eframe::{App, egui, Frame};
use egui::{CentralPanel, Context, FontId, TopBottomPanel, Label, TextEdit, Button, TextFormat, Color32, SidePanel};
use egui::text::{LayoutJob, LayoutSection};
use egui_file::FileDialog;
use crate::panels::Panels;
use crate::session::Session;
use crate::widgets::recents::RecentsBox;

struct TailorApp {
    panels: Panels,
    windows: Windows,
    session: Session,
    is_dirty: bool,
    open_file_dialog: Option<FileDialog>,
    recents_box: RecentsBox,
    next_open_file: Option<PathBuf>,
    tailor: Tailor,
    message_rx: Option<Receiver<Message>>,
    lines: Vec<String>,
    filter_text: String,
    search_text: String,
}

impl TailorApp {
    fn new(tailor: Tailor) -> Self {
        Self {
            panels: Panels::default(),
            windows: Windows::default(),
            session: Session::default(),
            is_dirty: false,
            open_file_dialog: None,
            recents_box: RecentsBox::default(),
            next_open_file: None,
            tailor,
            message_rx: None,
            lines: vec![],
            filter_text: String::new(),
            search_text: String::new(),
        }
    }
}

impl App for TailorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.is_dirty {
            if let Some(path) = &self.next_open_file {
                let (tx, rx) = channel();
                self.message_rx = Some(rx);
                self.tailor.watch(path.clone(), tx);
                self.lines.clear();
                self.session = Session::new(path.clone());
                self.recents_box.update_recents(path.as_path());
            }
            self.is_dirty = false;
        }

        if self.recents_box.is_dirty(self.session.get_path().as_path()) {
            self.next_open_file = Some(self.recents_box.get_selected_recent_path());
            self.is_dirty = true;
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
                ctx.request_repaint();
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

                self.recents_box.draw(ui);

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

        let frame = egui::containers::Frame {
            inner_margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
            outer_margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
            rounding: egui::Rounding { nw: 0.0, ne: 0.0, sw: 0.0, se: 0.0 },
            shadow: eframe::epaint::Shadow { extrusion: 0.0, color: Color32::BLACK },
            fill: self.session.get_colors().background(),
            stroke: egui::Stroke::new(0.0, Color32::BLACK),
        };
        CentralPanel::default().frame(frame).show(ctx, |ui| {
            self.panels.draw(ui, &mut self.session);

            egui::ScrollArea::both().show_rows(ui, 12.0, self.lines.len(),
   |ui, row_range| {
                    for row in row_range {
                        let text_format = TextFormat {
                            background: self.session.get_highlight( & self.lines[row]).background(),
                            color: self.session.get_highlight( & self.lines[row]).foreground(),
                            font_id: FontId::monospace(12.0),
                            ..Default::default()
                        };
                        let layout_job = LayoutJob {
                            sections: vec![LayoutSection {
                                leading_space: 0.0,
                                byte_range: 0..self.lines[row].len(),
                                format: text_format,
                            }],
                            text: self.lines[row].clone(),
                            break_on_newline: false,
                            ..Default::default()
                        };
                        let line_label = Label::new(layout_job).wrap(false);
                        ui.add(line_label);
                    }
                    ui.add(Label::new(""));
            });
        });

        self.windows.draw(ctx);

        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(self.session.get_path().display().to_string());
                ui.add(TextEdit::singleline(&mut self.filter_text).hint_text("Filter").desired_width(120.0));
                ui.add(TextEdit::singleline(&mut self.search_text).hint_text("Search").desired_width(120.0));
            });
        });
    }
}

fn init_data_path() {
    if let Ok(data_path) = get_app_root(AppDataType::UserData, &APP_INFO) {
        if !data_path.exists() && std::fs::create_dir_all(&data_path).is_err() {
            log::warn!("Unable to create data path: {:?}. Settings and sessions will not be persisted.", data_path);
        }
    }
}

fn main() {
    env_logger::init();
    init_data_path();

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
