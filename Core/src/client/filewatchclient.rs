use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;
use crate::{FileWatch, Message};
use crate::client::WatchClient;

pub struct FileWatchClient {
    owner_rx: Receiver<bool>,
    message_tx: Sender<Message>,
    _watchable_thread: JoinHandle<()>,
    watchable_rx: Receiver<Message>,
    watchable_tx: Sender<bool>,
}

impl FileWatchClient {
    pub fn new(path: PathBuf, owner_rx: Receiver<bool>, message_tx: Sender<Message>) -> Self {
        let (tx,rx) = channel();
        let mut watchable = FileWatch::new(path.as_path(), tx);
        let watchable_tx = watchable.get_tx();
        let thread = std::thread::spawn(move || {
            if watchable.watch().is_err() { log::error!("Failed to start watching file {}.", path.to_str().unwrap_or("UNKNOWN")) }
        });

        FileWatchClient {
            owner_rx,
            message_tx,
            _watchable_thread: thread,
            watchable_rx: rx,
            watchable_tx,
        }
    }
}

impl Drop for FileWatchClient {
    fn drop(&mut self) {
        if self.watchable_tx.send(true).is_err() {
            log::warn!("Failed to send stop to file watchable.");
        }
    }
}

impl WatchClient for FileWatchClient {
    fn start(&mut self, _client_id: i32) {
        loop {
            match self.watchable_rx.recv_timeout(Duration::from_secs(1)) {
                Ok(msg) => {
                    if self.message_tx.send(msg).is_err() {
                        log::error!("Failed to send new lines to owner.");
                    }
                },
                Err(_) => {
                    log::info!("FileWatchClient exited.")
                }
            }

            if let Ok(msg) = self.owner_rx.try_recv() {
                if msg {
                    break;
                }
            }
        }

        if self.watchable_tx.send(true).is_err() { log::error!("Failed to send kill message to watchable_tx.") }
    }
}
