use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::sync::mpsc::{channel, Receiver, Sender};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify::event::ModifyKind;

use crate::Message;

extern crate simplelog;

const STANDBY_WATCH_TIMEOUT_MS: u64 = 2000;
const ACTIVE_WATCH_TIMEOUT_MS: u64 = 100;

pub struct FileWatch {
    path: PathBuf,
    rx: Receiver<bool>,
    tx: Sender<bool>,
    message_tx: Sender<Message>,
    file_size: u64,
    read_offset: u64,
    should_join: bool,
    watch_timeout: u64,
}

impl FileWatch {
    pub fn new(path: &Path, message_tx: Sender<Message>) -> Self {
        let (tx, rx) = channel();
        FileWatch {
            path: PathBuf::from(path),
            rx,
            tx,
            message_tx,
            file_size: 0,
            read_offset: 0,
            should_join: false,
            watch_timeout: STANDBY_WATCH_TIMEOUT_MS,
        }
    }

    pub fn watch(&mut self) -> notify::Result<()> {
        log::info!("Watching file {}", self.path.to_str().unwrap());
        if let Err(error) = self.read_file() {
            log::error!("{}", error);
            return Ok(())
        }

        let (tx, rx) = channel();
        let config = Config::default()
            .with_poll_interval(Duration::from_secs(1));
        let mut watcher: RecommendedWatcher = Watcher::new(tx, config)?;
        watcher.watch(&self.path, RecursiveMode::NonRecursive)?;

        loop {
            match rx.recv_timeout(Duration::from_millis(self.watch_timeout)) {
                Ok(event) => {
                    if event.is_ok() {
                        if !self.process_event(&event.unwrap()) {
                            self.should_join = true;
                        }
                    } else {
                        log::error!("Error receiving Notify: {:?}", event);
                    }
                },
                _ => {
                    if let Err(error) = self.read_file() {
                        log::error!("{}", error);
                        self.should_join = true;
                    }
                }
            }

            if let Ok(msg) = self.rx.try_recv() {
                self.should_join = msg;
            }

            if self.should_join {
                break;
            }
        }

        // TODO: make this function void instead
        Ok(())
    }

    pub fn get_tx(&self) -> Sender<bool> {
        self.tx.clone()
    }

    fn process_event(&mut self, event: &Event) -> bool {
        match event.kind {
            EventKind::Modify(ModifyKind::Data(_)) => {
                if let Err(error) = self.read_file() {
                    log::error!("{}", error);
                    return false;
                }
            },
            EventKind::Create(_) => {
                if let Err(error) = self.read_file() {
                    log::error!("{}", error);
                    return false;
                }
            },
            EventKind::Remove(_) => return false,
            EventKind::Modify(ModifyKind::Name(_)) => return false,
            _ => ()
        }

        true
    }

    fn read_file(&mut self) -> Result<(),String> {
        let file = match File::open(&self.path) {
            Ok(opened_file) => opened_file,
            Err(error) => {
                self.read_offset = 0;
                return Err(format!("Could not open file for reading: {}. Error: {}", self.path.to_str().unwrap_or("Unknown file path"), error));
            }
        };

        match file.metadata() {
            Ok(x) => {
                self.file_size = x.len();
                if self.read_offset > self.file_size {
                    self.read_offset = 0;
                }
            },
            Err(e) => {
                self.file_size = 0;
                self.read_offset = 0;
                return Err(format!("Could not read file metadata due to an error: {}", e));
            }
        };

        if self.read_offset == self.file_size {
            return Ok(());
        }

        let mut reader = BufReader::new(file);
        if reader.seek(SeekFrom::Start(self.read_offset)).is_err() {
            self.read_offset = 0;
            if self.message_tx.send(Message::NewFile(self.path.clone())).is_err() {
                return Err("Failed to send data to file watch client: file seek reset".to_string());
            }
        }
        let mut lines_to_send = vec![];
        for line in reader.lines() {
            match line {
                Ok(s) => {
                    self.read_offset += s.len() as u64;
                    if self.file_size > self.read_offset {
                        self.read_offset += 1;
                    }
                    lines_to_send.push(s);
                },
                Err(_) => {
                    return Err("Failed to read line.".to_string());
                }
            }
        }

        if !lines_to_send.is_empty() {
            self.watch_timeout = ACTIVE_WATCH_TIMEOUT_MS;
            if self.message_tx.send(Message::NewLines(lines_to_send)).is_err() {
                self.should_join = true;
                return Err("Failed to send data to file watch client: new lines".to_string());
            }
        } else {
            self.watch_timeout = std::cmp::min(self.watch_timeout * 2, STANDBY_WATCH_TIMEOUT_MS);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::thread::JoinHandle;
    use std::time::Duration;
    use crate::{FileWatch, Message};

    const TEST_DIR: &str = "./test/filewatch";
    const WAIT_TIMEOUT_MS: u64 = 1000;

    struct Context {
        test_file: PathBuf,
        rx: Receiver<Message>,
        watcher_tx: Sender<bool>,
        handle: JoinHandle<()>,
    }

    fn create_test_file(filename: &str, contents: Option<&Vec<String>>) -> PathBuf {
        let mut file_path = PathBuf::from(TEST_DIR);
        assert_eq!(std::fs::create_dir_all(&file_path).is_ok(), true);
        file_path.push(filename);

        if std::fs::remove_file(&file_path).is_err() {
            log::error!("Failed to remove test file.");
        }

        let mut file = File::create(file_path.as_path()).unwrap();

        if let Some(strings) = contents {
            for s in strings {
                assert_eq!(file.write(s.as_bytes()).is_ok(), true);
                assert_eq!(file.write(String::from('\n').as_bytes()).is_ok(), true);
            }
        }

        file_path
    }

    fn create_context(file_name: &str, file_contents: Option<&Vec<String>>) -> Context {
        let test_file = create_test_file(file_name, file_contents);
        let (tx, rx) = channel();
        let mut file_watch = FileWatch::new(test_file.as_path(), tx);
        let watcher_tx = file_watch.get_tx();
        let handle = std::thread::spawn(move || {
            assert_eq!(file_watch.watch().is_ok(), true);
        });

        Context {
            test_file,
            rx,
            watcher_tx,
            handle
        }
    }

    #[test]
    fn initial_read() {
        let contents = vec![String::from("Line1"), String::from("Line2")];
        let context = create_context("initial_read.txt", Some(&contents));

        let initial_read = context.rx.recv_timeout(Duration::from_millis(WAIT_TIMEOUT_MS));
        assert_eq!(initial_read.is_ok(), true);

        if let Message::NewLines(read_contents) = initial_read.unwrap() {
            for line in 0..read_contents.len() {
                assert_eq!(read_contents[line], contents[line]);
            }
        }
    }

    #[test]
    fn new_content() {
        let contents = vec![String::from("Line1"), String::from("Line2")];
        let context = create_context("new_content.txt", Some(&contents));

        let _initial_read = context.rx.recv_timeout(Duration::from_millis(WAIT_TIMEOUT_MS));
        let file = OpenOptions::new().write(true).append(true).open(context.test_file);
        assert_eq!(file.is_ok(), true);
        let mut file = file.unwrap();
        let new_content = String::from("New content");
        assert_eq!(file.write(new_content.as_bytes()).is_ok(), true);
        assert_eq!(file.flush().is_ok(), true);

        let recv_contents = context.rx.recv_timeout(Duration::from_millis(WAIT_TIMEOUT_MS));
        assert_eq!(recv_contents.is_ok(), true);
        if let Message::NewLines(msgs) = recv_contents.unwrap() {
            assert_eq!(msgs[0], new_content);
        }
    }

    #[test]
    fn removed_content() {
        let contents = vec![String::from("Line1"), String::from("Line2")];
        let context = create_context("removed_content.txt", Some(&contents));

        let _initial_read = context.rx.recv_timeout(Duration::from_millis(WAIT_TIMEOUT_MS));
        let file = File::create(context.test_file);
        assert_eq!(file.is_ok(), true);
        let mut file = file.unwrap();
        let new_content = String::from("New content");
        assert_eq!(file.write(new_content.as_bytes()).is_ok(), true);

        let recv_contents = context.rx.recv_timeout(Duration::from_millis(WAIT_TIMEOUT_MS));
        assert_eq!(recv_contents.is_ok(), true);
        if let Message::NewLines(msgs) = recv_contents.unwrap() {
            assert_eq!(msgs[0], new_content);
        }
    }

    #[test]
    fn removed_file() {
        let context = create_context("removed_file.txt", None);
        let _initial_read = context.rx.recv_timeout(Duration::from_millis(WAIT_TIMEOUT_MS));

        assert_eq!(std::fs::remove_file(context.test_file).is_ok(), true);
        assert_eq!(context.handle.join().is_ok(), true);
    }

    #[test]
    fn request_exit() {
        let context = create_context("removed_file.txt", None);
        assert_eq!(context.watcher_tx.send(true).is_ok(), true);
        assert_eq!(context.handle.join().is_ok(), true);
    }
}
