use std::ffi::{OsStr};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, SystemTime};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify::event::{CreateKind, DataChange, ModifyKind};

pub struct DirWatch {
    path: PathBuf,
    rx: Receiver<bool>,
    tx: Sender<bool>,
    parent_tx: Sender<PathBuf>,
    last_reported_file: PathBuf,
    should_join: bool,
}

impl DirWatch {
    pub fn new(path: &Path, parent_tx: Sender<PathBuf>) -> Self {
        let (tx, rx) = channel();
        DirWatch {
            path: PathBuf::from(path),
            rx,
            tx,
            parent_tx,
            last_reported_file: PathBuf::new(),
            should_join: false,
        }
    }

    pub fn watch(&mut self) -> notify::Result<()> {
        log::info!("Watching directory {}", self.path.as_path().to_str().unwrap_or("UNKNOWN"));
        if let Some(file) = self.pick_latest_file() {
            self.last_reported_file = PathBuf::from(file.file_name().unwrap_or_default());
            if self.parent_tx.send(file).is_err() { self.should_join = true }
        };

        let (tx, rx) = channel();
        let config = Config::default()
            .with_poll_interval(Duration::from_secs(1));
        let mut watcher: RecommendedWatcher = Watcher::new(tx, config)?;
        watcher.watch(&self.path, RecursiveMode::NonRecursive)?;

        loop {
            if let Ok(event) = rx.recv_timeout(Duration::from_secs(1)) {
                if event.is_ok() {
                    self.process_event(&event.unwrap());
                } else {
                    log::error!("Failed to receive Nofity event: {:?}", event);
                }
            }

            if let Ok(msg) = self.rx.try_recv() {
                self.should_join = msg;
            }

            if self.should_join {
                break;
            }
        }

        Ok(())
    }

    pub fn get_tx(&self) -> Sender<bool> {
        self.tx.clone()
    }

    fn process_event(&mut self, event: &Event) {
        match &event.kind {
            EventKind::Create(CreateKind::File) => {
                if let Some(latest_file) = self.pick_latest_file() {
                    self.process_file(&latest_file)
                }
            },
            EventKind::Modify(ModifyKind::Data(DataChange::Content)) => {
                if let Some(latest_file) = self.pick_latest_file() {
                    self.process_file(&latest_file)
                }
            },
            _ => ()
        }
    }

    fn process_file(&mut self, file: &Path) {
        if file.is_file() {
            match file.file_name() {
                Some(filename) => {
                    if !Self::filename_allowed(filename) {
                        return;
                    }

                    let filename_path_buf = PathBuf::from(filename);
                    if self.last_reported_file != filename_path_buf {
                        self.last_reported_file = filename_path_buf;
                        // TODO: handle error correctly. E.g. return Result and stop loop on error
                        self.parent_tx.send(file.to_path_buf()).unwrap();
                    }
                },
                None => ()
            };
        }
    }

    fn filename_allowed(filename: &OsStr) -> bool {
        match Path::new(filename).file_name() {
            Some(name) => {
                !name.to_str().unwrap_or("").starts_with('.')
            },
            None => false
        }
    }

    fn pick_latest_file(&self) -> Option<PathBuf> {
        let dir = std::fs::read_dir(&self.path);
        if let Err(err) = dir {
            log::error!("Not a directory: {}", err);
            return None;
        }

        let mut newest_file: Option<(PathBuf,SystemTime)> = None;

        let dir = dir.unwrap();
        for entry in dir {
            if entry.is_err() {
                continue;
            }

            let dir_entry = entry.unwrap();
            if !dir_entry.path().is_file() {
                continue;
            }

            if !Self::filename_allowed(dir_entry.path().as_os_str()) {
                continue;
            }

            let file = File::open(dir_entry.path());
            if file.is_err() {
                continue;
            }

            let metadata = file.unwrap().metadata();
            if metadata.is_err() {
                continue;
            }

            let modified = metadata.unwrap().modified();
            if modified.is_err() {
                continue;
            }

            let modified = modified.unwrap();
            if newest_file.is_none() {
                newest_file = Some((dir_entry.path().to_path_buf(), modified));
                continue;
            }

            let (_,current_newest_file_time) = newest_file.as_ref().unwrap();
            if current_newest_file_time < &modified {
                newest_file = Some((dir_entry.path().to_path_buf(), modified));
            }
        }

        newest_file.map(|(path,_)| path)
    }
}

#[cfg(test)]
mod test {
    use std::ffi::OsStr;
    use std::fs::File;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
    use std::thread::JoinHandle;
    use std::time::Duration;
    use crate::DirWatch;

    const TEST_DIR: &str = "./test/dirwatch";
    const WAIT_TIMEOUT_MS: u64 = 3000;

    struct Context {
        test_dir: PathBuf,
        rx: Receiver<PathBuf>,
        watcher_tx: Sender<bool>,
        handle: JoinHandle<()>,
    }

    fn create_empty_test_dir(dir_name: &str) -> PathBuf {
        let mut dir_path = PathBuf::from(TEST_DIR);
        dir_path.push(dir_name);
        if dir_path.as_path().is_dir() {
            assert_eq!(std::fs::remove_dir_all(dir_path.as_path()).is_ok(), true);
        }

        assert_eq!(std::fs::create_dir_all(dir_path.as_path()).is_ok(), true);

        dir_path
    }

    fn create_test_file(dir: &Path, filename: &str, contents: Option<&Vec<String>>) {
        let mut file_path = dir.to_path_buf();
        file_path.push(filename);
        let mut file = File::create(file_path.as_path()).unwrap();

        if let Some(strings) = contents {
            for s in strings {
                assert_eq!(file.write(s.as_bytes()).is_ok(), true);
            }
        }

    }

    fn create_context(test_name: &str) -> Context {
        let test_dir = create_empty_test_dir(test_name);
        let (tx, rx) = channel();
        let mut dir_watch = DirWatch::new(test_dir.as_path(), tx);
        let watcher_tx = dir_watch.get_tx();
        let handle = std::thread::spawn(move || {
            assert_eq!(dir_watch.watch().is_ok(), true);
        });

        Context {
            test_dir,
            rx,
            watcher_tx,
            handle
        }
    }

    #[test]
    fn empty_dir() {
        let context = create_context("empty_dir");

        assert_eq!(context.rx.recv_timeout(Duration::from_millis(WAIT_TIMEOUT_MS)), Err(RecvTimeoutError::Timeout));
    }

    #[test]
    fn pick_latest_file() {
        let dir_path = create_empty_test_dir("pick_latest_file");
        create_test_file(&dir_path, "file1.txt", None);
        create_test_file(&dir_path, "file2.txt", None);

        let (tx, rx) = channel();
        let mut dir_watch = DirWatch::new(dir_path.as_path(), tx);
        let dir_watch_tx = dir_watch.get_tx();
        let handle = std::thread::spawn(move || {
            assert_eq!(dir_watch.watch().is_ok(), true);
        });

        let latest_file = rx.recv_timeout(Duration::from_millis(WAIT_TIMEOUT_MS));
        assert_eq!(latest_file.is_ok(), true);

        let latest_file_path = latest_file.unwrap();
        assert_eq!(latest_file_path.file_name().unwrap_or(OsStr::new("NO_FILENAME")), "file2.txt");
        assert_eq!(dir_watch_tx.send(true).is_ok(), true);
        assert_eq!(handle.join().is_ok(), true);
    }

    #[test]
    fn report_created() {
        let context = create_context("report_created");

        create_test_file(&context.test_dir, "file1.txt", None);
        let created_file = context.rx.recv_timeout(Duration::from_millis(WAIT_TIMEOUT_MS));
        assert_eq!(created_file.is_ok(), true);
        let created_file_path = created_file.unwrap();
        assert_eq!(created_file_path.file_name().unwrap_or_else(|| OsStr::new("NO_FILENAME")), "file1.txt");

        std::thread::sleep(Duration::from_secs(1));

        create_test_file(&context.test_dir, "file2.txt", None);
        let created_file = context.rx.recv_timeout(Duration::from_millis(WAIT_TIMEOUT_MS));
        assert_eq!(created_file.is_ok(), true);
        let created_file_path = created_file.unwrap();
        assert_eq!(created_file_path.file_name().unwrap_or_else(|| OsStr::new("NO_FILENAME")), "file2.txt");
    }

    #[test]
    fn report_changed() {
        let context = create_context("report_changed");

        create_test_file(&context.test_dir, "file1.txt", None);
        std::thread::sleep(Duration::from_secs(1));
        let created_file1 = context.rx.recv_timeout(Duration::from_millis(WAIT_TIMEOUT_MS));

        create_test_file(&context.test_dir, "file2.txt", None);
        std::thread::sleep(Duration::from_secs(1));
        let _created_file2 = context.rx.recv_timeout(Duration::from_millis(WAIT_TIMEOUT_MS));

        let file_to_change = File::create(created_file1.as_ref().unwrap());
        assert_eq!(file_to_change.is_ok(), true);
        let mut file_to_change = file_to_change.unwrap();
        assert_eq!(file_to_change.write(String::from("Test line").as_bytes()).is_ok(), true);

        std::thread::sleep(Duration::from_secs(1));

        let changed_file = context.rx.recv_timeout(Duration::from_millis(WAIT_TIMEOUT_MS));
        assert_eq!(changed_file.is_ok(), true);
        assert_eq!(changed_file.unwrap().file_name().unwrap_or_else(|| OsStr::new("INVALID1")), created_file1.unwrap().file_name().unwrap_or_else(|| OsStr::new("INVALID2")));
    }

    #[test]
    fn request_exit() {
        let context = create_context("request_exit");

        assert_eq!(context.watcher_tx.send(true).is_ok(), true);
        assert_eq!(context.handle.join().is_ok(), true);
    }
}
