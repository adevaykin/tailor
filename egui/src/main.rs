#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod windows;
mod panels;
mod session;
mod highlight;

use app_dirs2::*; // or app_dirs::* if you've used package alias in Cargo.toml

const APP_INFO: AppInfo = AppInfo{name: "Tailor", author: "Alexander Devaikin"};
const RECENTS_FILENAME: &str = "recents.json";

use tailor::{Tailor, Message};
use windows::Windows;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use eframe::{App, egui, Frame};
use egui::{CentralPanel, ComboBox, Context, FontId, TopBottomPanel, Label, TextEdit, Button, InnerResponse};
use egui::text::LayoutJob;
use egui_file::FileDialog;
use crate::panels::Panels;
use crate::session::Session;

struct RecentsBox {
    recents: Vec<String>,
    selected_recent: String,
}

impl Default for RecentsBox {
    fn default() -> Self {
        Self {
            recents: Self::try_load_recent(),
            selected_recent: String::new(),
        }
    }
}

impl RecentsBox {
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ComboBox::from_label("Recent")
            .selected_text(&self.selected_recent)
            .width(300.0)
            .show_ui(ui, |ui| {
                for recent in &self.recents {
                    ui.selectable_value(&mut self.selected_recent, recent.clone(), recent);
                }
            });
    }

    pub fn is_dirty(&self, prev_path: &Path) -> bool {
        self.selected_recent != prev_path.display().to_string()
    }

    pub fn update_recents(&mut self, path: &Path) {
        self.recents = self.recents.clone().into_iter().filter(|recent| *recent != path.display().to_string()).collect();
        self.recents.insert(0, path.display().to_string());
        self.recents.truncate(10);
        self.selected_recent = path.display().to_string();
        Self::try_save_recents(&self.recents);
    }

    pub fn get_selected_recent_path(&self) -> PathBuf {
        PathBuf::from(&self.selected_recent)
    }

    fn try_load_recent() -> Vec<String> {
        if let Ok(data_path) = get_app_root(AppDataType::UserData, &APP_INFO) {
            if data_path.exists() {
                let recents_path = data_path.join(RECENTS_FILENAME);
                if let Ok(loaded_recents) = std::fs::read_to_string(&recents_path)
                {
                    if let Ok(recents) = serde_json::from_str(&loaded_recents) {
                        return recents;
                    }
                } else {
                    let recents: Vec<String> = vec![];
                    let recents_json = serde_json::to_string(&recents).unwrap();
                    let _ = std::fs::write(&recents_path, recents_json);
                }
            }
        }

        vec![]
    }

    fn try_save_recents(recents: &Vec<String>) {
        if let Ok(data_path) = get_app_root(AppDataType::UserData, &APP_INFO) {
            let recents_path = data_path.join(RECENTS_FILENAME);
            let recents_json = serde_json::to_string(recents).unwrap();
            let _ = std::fs::write(&recents_path, recents_json);
        }
    }
}

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
        let ret = Self {
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
        };

        ret
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
