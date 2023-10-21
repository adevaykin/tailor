use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Duration;
use crate::{DirWatch, FileWatch, Message};
use crate::client::WatchClient;

pub struct DirWatchClient {
    owner_rx: Receiver<bool>,
    message_tx: Sender<Message>,
    watchable_tx: Sender<bool>,
    watchable_rx: Receiver<PathBuf>,
    _watchable_thread: JoinHandle<()>,
    file_watchable_rx: Option<Receiver<Message>>,
    file_watchable_tx: Option<Sender<bool>>,
    file_watchable_thread: Option<JoinHandle<()>>,
}

impl DirWatchClient {
    pub fn new(path: PathBuf, owner_rx: Receiver<bool>, message_tx: Sender<Message>) -> Self {
        let (tx,rx) = channel();
        let mut watchable = DirWatch::new(path.as_path(), tx);
        let watchable_tx = watchable.get_tx();
        let thread = std::thread::spawn(move || {
            if watchable.watch().is_err() { log::error!("Failed to start watching directory {}.", path.to_str().unwrap_or("UNKNOWN")) }
        });

        DirWatchClient {
            owner_rx,
            message_tx,
            watchable_tx,
            watchable_rx: rx,
            _watchable_thread: thread,
            file_watchable_rx: None,
            file_watchable_tx: None,
            file_watchable_thread: None,
        }
    }

    fn start_filewatch(&mut self, _client_id: i32, path: PathBuf) {
        if self.message_tx.send(Message::NewFile(path.clone())).is_err() {
            log::error!("Failed to send message NewFile to owner.");
        }

        let (tx,rx) = channel();
        let file_path = path.clone();
        let mut watchable = FileWatch::new(file_path.as_path(), tx);
        let watchable_tx = watchable.get_tx();
        let thread = std::thread::spawn(move || {
            if watchable.watch().is_err() { log::error!("Failed to start watching file {}.", file_path.to_str().unwrap_or("UNKNOWN")) }
        });

        self.file_watchable_rx = Some(rx);
        self.file_watchable_tx = Some(watchable_tx);
        self.file_watchable_thread = Some(thread);
        log::info!("Started watching file {:?}", path);
    }

    fn kill_current_file_watchable(&mut self) {
        if let Some(tx) = self.file_watchable_tx.as_ref() {
            if tx.send(true).is_err() {
                log::warn!("Failed to kill current file watchable.");
            }
            self.file_watchable_tx = None;
        }

        self.file_watchable_thread = None;
        self.file_watchable_rx = None;
    }
}

impl Drop for DirWatchClient {
    fn drop(&mut self) {
        self.kill_current_file_watchable();
        if self.watchable_tx.send(true).is_err()
        {
            log::warn!("Failed to send stop to watchable");
        }
    }
}

impl WatchClient for DirWatchClient {
    fn start(&mut self, client_id: i32) {
        loop {
            if let Ok(msg) = self.watchable_rx.try_recv() {
                self.kill_current_file_watchable();
                self.start_filewatch(client_id, msg);
            }

            if let Some(rx) = self.file_watchable_rx.as_ref() {
                match rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(msg) => {
                        if self.message_tx.send(msg).is_err() {
                            log::error!("Dir warch client failed to send message to owner.");
                        }
                    },
                    Err(RecvTimeoutError::Disconnected) => {
                        log::error!("File watcher disconnected.");
                        break;
                    },
                    Err(_) => ()
                }
            }

            if let Ok(msg) = self.owner_rx.try_recv() {
                if msg {
                    break;
                }
            }
        }
    }
}
