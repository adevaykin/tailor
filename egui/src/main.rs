#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod windows;
mod panels;
mod session;
mod highlight;
mod widgets;
mod lines;

use app_dirs2::*; // or app_dirs::* if you've used package alias in Cargo.toml

const APP_INFO: AppInfo = AppInfo{name: "Tailor", author: "Alexander Devaikin"};

use tailor::{Tailor, Message};
use windows::Windows;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use eframe::{App, egui, Frame};
use egui::{CentralPanel, Context, FontId, TopBottomPanel, Label, TextEdit, Button, TextFormat, Color32, Layout, Align};
use egui::text::{LayoutJob, LayoutSection};
use egui_file::FileDialog;
use objc::{msg_send,class,sel,sel_impl};
use objc::runtime::{Class, Object};
use regex::Regex;
use crate::lines::LinesState;
use crate::panels::Panels;
use crate::session::Session;
use crate::widgets::recents::RecentsBox;

struct TailorClinet {
    handle: std::thread::JoinHandle<()>,
}

impl TailorClinet {
    fn new(tailor: &mut Tailor, path: &PathBuf, ctx: Context, log_contents: Arc<Mutex<LinesState>>) -> Self {
        let (message_tx, message_rx) = channel();
        let client_handle = std::thread::spawn(move || {
            while match message_rx.recv_timeout(std::time::Duration::from_secs(2)) {
                Ok(msg) => {
                    if let Ok(mut lines) = log_contents.lock() {
                        match msg {
                            Message::NewLines(recv_lines) => {
                                (*lines).add_lines(recv_lines);
                            },
                            Message::NewFile(_path) => {
                                (*lines).clear_lines();
                            }
                        }
                    }

                    ctx.request_repaint();
                    true
                },
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    true
                },
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    log::error!("Tailor thread disconnected");
                    false
                }
            } {}
        });

        tailor.watch(path.clone(), message_tx);

        Self {
            handle: client_handle,
        }
    }
}

fn find_ranges(line: &String, regex: &Regex) -> Vec<(usize, usize)> {
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

struct TailorApp {
    panels: Panels,
    windows: Windows,
    session: Session,
    is_dirty: bool,
    open_file_dialog: Option<FileDialog>,
    recents_box: RecentsBox,
    next_open_file: Option<PathBuf>,
    tailor: Tailor,
    tailor_client: Option<TailorClinet>,
    log_contents: Arc<Mutex<LinesState>>,
    filter_text: String,
    search_text: String,
    search_regex: Option<Regex>,
}

impl TailorApp {
    fn new(tailor: Tailor) -> Self {
        Self {
            panels: Panels::default(),
            windows: Windows::default(),
            session: Session::default(),
            is_dirty: true,
            open_file_dialog: None,
            recents_box: RecentsBox::default(),
            next_open_file: None,
            tailor,
            tailor_client: None,
            log_contents: Arc::new(Mutex::new(LinesState::new())),
            filter_text: String::new(),
            search_text: String::new(),
            search_regex: None,
        }
    }
}

impl App for TailorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
            if self.is_dirty {
                if let Some(path) = &self.next_open_file {
                    if let Ok(mut lines) = self.log_contents.lock() {
                        (*lines).clear_lines();
                    }
                    self.tailor_client = Some(TailorClinet::new(
                        &mut self.tailor,
                        path,
                        ctx.clone(),
                        self.log_contents.clone(),
                    ));
                    self.session = Session::new(path.clone());
                    self.recents_box.update_recents(path.as_path());
                }

                self.is_dirty = false;
            }

        if self.recents_box.is_dirty(self.session.get_path().as_path()) {
            self.next_open_file = Some(self.recents_box.get_selected_recent_path());
            self.is_dirty = true;
        }

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
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
                });

                ui.horizontal(|ui| {
                    self.recents_box.draw(ui);
                    let session_settings_button = Button::new("🎨")
                        .selected(self.panels.session_settings.get_is_visible());
                    if ui.add(session_settings_button).clicked() {
                        self.panels.session_settings.toggle_is_visible();
                    }
                });

                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    let about_button = Button::new("About")
                        .selected(self.windows.about.get_is_visible());
                    if ui.add(about_button).clicked() {
                        self.windows.about.toggle_is_visible();
                    }
                });
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

            if let Ok(mut log_contents) = self.log_contents.lock() {
                let filtered_lines = log_contents.get_filtered_lines(&self.filter_text);
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show_rows(ui, 12.0, filtered_lines.len(),
           |ui, row_range| {
                           for row in row_range {
                               let text_format = TextFormat {
                                   background: self.session.get_highlight(&filtered_lines[row]).background(),
                                   color: self.session.get_highlight(&filtered_lines[row]).foreground(),
                                   font_id: FontId::monospace(12.0),
                                   ..Default::default()
                               };
                               let inverted_text_format = TextFormat {
                                   background: self.session.get_highlight(&filtered_lines[row]).foreground(),
                                   color: self.session.get_highlight(&filtered_lines[row]).background(),
                                   font_id: FontId::monospace(12.0),
                                   ..Default::default()
                               };

                               let found_ranges = if let Some(regex) = &self.search_regex {
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
            }
        });

        self.windows.draw(ctx);

        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.label(self.session.get_path().display().to_string());
                });

                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    if ui.add(TextEdit::singleline(&mut self.search_text)
                        .hint_text("Search").desired_width(120.0))
                        .changed() {
                        if !self.search_text.is_empty() {
                            if let Ok(regex) = Regex::new(format!(r"(?i){}", &self.search_text).as_str()) {
                                self.search_regex = Some(regex);
                            }
                        } else {
                            self.search_regex = None;
                        }
                    }
                    ui.add(TextEdit::singleline(&mut self.filter_text).hint_text("Filter").desired_width(120.0));
                });
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

    // let ns_app: *mut Object = unsafe { msg_send![class!(NSApplication), sharedApplication] };
    // log::info!("NSApp: {:?}", ns_app);
    // let menu: *mut Object = unsafe {
    //     let menu_class = class!(NSMenu);
    //     log::info!("MenuClass: {:?}", menu_class);
    //     let menu: *mut Object = msg_send![menu_class, alloc];
    //     let is_kind_of_nsmenu: bool = msg_send![menu, isKindOfClass: menu_class];
    //     log::info!("IsKindOfNSMenu: {:?}", is_kind_of_nsmenu);
    //     let menu: *mut Object = msg_send![menu, initWithTitle:"MainMenu"];
    //     log::info!("Menu: {:?}", menu);
    //     menu
    // };
    // log::info!("Menu: {:?}", menu);
    // let _: () = unsafe { msg_send![ns_app, setMainMenu: menu] };

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
