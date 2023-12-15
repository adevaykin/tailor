#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod highlight;
mod lines;
mod panels;
mod session;
mod widgets;
mod windows;

use app_dirs2::*; // or app_dirs::* if you've used package alias in Cargo.toml

const APP_INFO: AppInfo = AppInfo {
    name: "Tailor",
    author: "Alexander Devaikin",
};

use crate::lines::LinesState;
use crate::panels::main::MainPanel;
use crate::session::Session;
use crate::widgets::recents::RecentsBox;
use eframe::{egui, App, Frame};
use egui::{Align, Button, Context, Layout, TextEdit, TopBottomPanel};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use tailor::{Message, Tailor};
use windows::Windows;

struct TailorClient {
    #[allow(dead_code)]
    handle: std::thread::JoinHandle<()>,
}

impl TailorClient {
    fn new(
        tailor: &mut Tailor,
        path: &Path,
        ctx: Context,
        log_contents: Arc<Mutex<LinesState>>,
    ) -> Self {
        let (message_tx, message_rx) = channel();
        let client_handle = std::thread::spawn(move || {
            while match message_rx.recv_timeout(std::time::Duration::from_secs(2)) {
                Ok(msg) => {
                    if let Ok(mut lines) = log_contents.lock() {
                        match msg {
                            Message::NewLines(recv_lines) => {
                                (*lines).add_lines(recv_lines);
                            }
                            Message::NewFile(_path) => {
                                (*lines).clear_lines();
                            }
                        }
                    }

                    ctx.request_repaint();
                    true
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => true,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    log::error!("Tailor thread disconnected");
                    false
                }
            } {}
        });

        tailor.watch(PathBuf::from(path), message_tx);

        Self {
            handle: client_handle,
        }
    }
}

struct TailorApp {
    windows: Windows,
    session: Session,
    file_pick_channel: (Sender<PathBuf>, Receiver<PathBuf>),
    is_dirty: bool,
    recents_box: RecentsBox,
    next_open_file: Option<PathBuf>,
    tailor: Tailor,
    tailor_client: Option<TailorClient>,
    log_contents: Arc<Mutex<LinesState>>,
    log_panel: MainPanel,
    settings_panel: panels::session_settings::SessionSettingsPanel,
    filter_text: String,
    search_text: String,
    search_regex: Option<Regex>,
}

impl TailorApp {
    fn new(tailor: Tailor) -> Self {
        Self {
            windows: Windows::default(),
            session: Session::default(),
            file_pick_channel: channel(),
            is_dirty: true,
            recents_box: RecentsBox::default(),
            next_open_file: None,
            tailor,
            tailor_client: None,
            log_contents: Arc::new(Mutex::new(LinesState::new())),
            log_panel: MainPanel::new(),
            settings_panel: panels::session_settings::SessionSettingsPanel::default(),
            filter_text: String::new(),
            search_text: String::new(),
            search_regex: None,
        }
    }
}

impl App for TailorApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if let Ok(path) = self.file_pick_channel.1.try_recv() {
            self.next_open_file = Some(path);
            self.is_dirty = true;
        }

        if self.is_dirty {
            if let Some(path) = &self.next_open_file {
                if let Ok(mut lines) = self.log_contents.lock() {
                    (*lines).clear_lines();
                }
                self.tailor_client = Some(TailorClient::new(
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
                    if ui.button("📃 File")
                        .on_hover_text("Pick single file to follow")
                        .clicked() {
                        let sender = self.file_pick_channel.0.clone();
                        let task = rfd::AsyncFileDialog::new().pick_file();
                        std::thread::spawn(move || futures::executor::block_on(async move {
                            let file = task.await;
                            if let Some(file) = file {
                                sender.send(file.path().to_path_buf()).unwrap();
                            }
                        }));
                    }
                    if ui.button("📂 Folder")
                        .on_hover_text("Pick folder with files to follow")
                        .clicked() {
                        let sender = self.file_pick_channel.0.clone();
                        let task = rfd::AsyncFileDialog::new().pick_folder();
                        std::thread::spawn(move || futures::executor::block_on(async move {
                            let file = task.await;
                            if let Some(file) = file {
                                sender.send(file.path().to_path_buf()).unwrap();
                            }
                        }));
                    }
                });

                ui.horizontal(|ui| {
                    self.recents_box.draw(ui);
                    let session_settings_button =
                        Button::new("🎨").selected(self.settings_panel.get_is_visible());
                    if ui.button("☜")
                        .on_hover_text("Reveal in file manager")
                        .clicked() {
                        let _ = open::that(self.session.get_path());
                    }
                    if ui.add(session_settings_button)
                        .on_hover_text("Session settings")
                        .clicked() {
                        self.settings_panel.toggle_is_visible();
                    }
                });

                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    let about_button =
                        Button::new("About").selected(self.windows.about.get_is_visible());
                    if ui.add(about_button).clicked() {
                        self.windows.about.toggle_is_visible();
                    }
                });
            });
        });

        if let Ok(mut log_contents) = self.log_contents.lock() {
            self.log_panel.draw(
                &mut self.session,
                ctx,
                &mut log_contents,
                &self.filter_text,
                &self.search_regex,
            );
        }
        self.settings_panel.draw(ctx, &mut self.session);
        self.windows.draw(ctx);

        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.label(self.session.get_path().display().to_string());
                });

                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    if ui
                        .add(
                            TextEdit::singleline(&mut self.search_text)
                                .hint_text("Search")
                                .desired_width(120.0),
                        )
                        .changed()
                    {
                        if !self.search_text.is_empty() {
                            if let Ok(regex) =
                                Regex::new(format!(r"(?i){}", &self.search_text).as_str())
                            {
                                self.search_regex = Some(regex);
                            }
                        } else {
                            self.search_regex = None;
                        }
                    }
                    ui.add(
                        TextEdit::singleline(&mut self.filter_text)
                            .hint_text("Filter")
                            .desired_width(120.0),
                    );
                });
            });
        });
    }
}

fn init_data_path() {
    if let Ok(data_path) = get_app_root(AppDataType::UserData, &APP_INFO) {
        if !data_path.exists() && std::fs::create_dir_all(&data_path).is_err() {
            log::warn!(
                "Unable to create data path: {:?}. Settings and sessions will not be persisted.",
                data_path
            );
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
            let native_options = eframe::NativeOptions::default();
            let _ = eframe::run_native(
                "Tailor",
                native_options,
                Box::new(|_cc| Box::new(TailorApp::new(tailor))),
            );
        }
        Err(msg) => {
            println!("Failed to create Tailor instance:");
            println!("{}", msg);
        }
    }
}
