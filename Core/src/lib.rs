use std::collections::HashMap;
use std::fs::File;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::JoinHandle;
use crate::filewatch::FileWatch;

use simplelog::*;
use crate::client::dirwatchclient::DirWatchClient;
use crate::client::filewatchclient::FileWatchClient;
use crate::client::WatchClient;
use crate::dirwatch::DirWatch;

mod filewatch;
mod dirwatch;
mod client;

const LOG_FILENAME: &str = "tailor.log";
const INVALID_CLIENT_ID : i32 = -1;
pub const MSG_TYPE_OPEN_FILE: u32 = 0;
pub const MSG_TYPE_ADD_LINES: u32 = 1;

pub type RustCallback = Box<dyn Fn(i32, u32, Vec<String>) + Send + Sync>;
type CCallback = Box<dyn Fn(i32, u32, u32, *const *mut c_char) + Send + Sync>;

static mut C_CALLBACK: Option<CCallback> = None;

pub enum Message {
    /// New lines in file we are watching
    NewLines(Vec<String>),
    /// New file is open for watching
    NewFile(PathBuf),
}

struct ClientInfo {
    tx: Sender<bool>,
    handle: JoinHandle<()>,
}

pub struct Tailor {
    clients: HashMap<i32,ClientInfo>,
    message_rx: Option<Receiver<Message>>,
}

impl Tailor {
    /// Rust API: Create Tailor instance
    pub fn new() -> Result<Self, String> {
        let _ = std::fs::remove_file(LOG_FILENAME);
        let _ = CombinedLogger::init(
            vec![
                TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
                WriteLogger::new(LevelFilter::Info, Config::default(), File::create(LOG_FILENAME).unwrap()),
            ]
        );

        Ok(Self {
            clients: HashMap::new(),
            message_rx: None,
        })
    }

    pub fn set_message_tx(&mut self, rx: Receiver<Message>) {
        self.message_rx = Some(rx)
    }

    pub fn watch(&mut self, path: PathBuf, message_tx: Sender<Message>) -> i32 {
        let (client_tx,client_rx) = channel();
        let mut client: Box<dyn WatchClient + Send> = if path.is_file() {
            Box::new(FileWatchClient::new(path, client_rx, message_tx))
        } else {
            Box::new(DirWatchClient::new(path, client_rx, message_tx))
        };

        let max_client_id = self.clients.iter().fold(0, |max, (key,_)| if *key > max { *key } else { max }) + 1;
        let client_info = ClientInfo {
            tx: client_tx,
            handle: std::thread::spawn(move | | { client.start(max_client_id); }),
        };
        self.clients.insert(max_client_id, client_info);
        max_client_id
    }

    pub fn stop(&mut self, client_id: i32) {
        if let Some(client_info) = &self.clients.remove(&client_id)
        {
            if client_info.tx.send(true).is_err()
            {
                log::warn!("Failed to send stop message to client thread.")
            }
        }
    }
}

// unsafe fn call_callback(client_id: i32, msg_type: u32, msg: Vec<String>) {
//     if let Some(tx) = RUST_CALLBACK.as_ref() {
//         (callback)(client_id, MSG_TYPE_ADD_LINES, msg);
//     } else if let Some(callback) = C_CALLBACK.as_ref() {
//         log::info!("Preparing strings");
//         let mut strings = vec![];
//         for string in msg {
//             let cstring = CString::new(&*string)
//                 .unwrap_or_else(|_| CString::new("Tailor: error to convert string to CString.").unwrap());
//             strings.push(cstring);
//         }

//         let mut out = strings.into_iter().map(|s| s.into_raw()).collect::<Vec<_>>();
//         out.shrink_to_fit();

//         let out = out;

//         log::info!("Sending strings");
//         let ptr = out.as_ptr();
//         (callback)(client_id, msg_type, out.len() as u32, ptr);

//         for elem in out {
//             let s = CString::from_raw(elem);
//             std::mem::drop(s);
//         }
//         log::info!("Sent strings");
//     }
// }

fn wrap_instance(instance: *mut Tailor) -> Box<Tailor> {
    unsafe { Box::from_raw(instance) }
}

fn unwrap_instance(instance: Box<Tailor>) {
    let _ = Box::into_raw(instance);
}

/// C API: Initialise Tailor instance
///
/// Instance must be destroyed by calling tailor_destroy() in the end
#[no_mangle]
pub extern fn tailor_init() -> *const Tailor {
    log::info!("Initializing Tailor instance.");
    if let Ok(instance) = Tailor::new() {
        let boxed = Box::new(instance);
        Box::into_raw(boxed)
    } else {
        let ret: *const Tailor = std::ptr::null();
        ret
    }
}

/// C API: Destroy Tailor instance
///
/// # Safety
///
/// This is a C function, unsafe by definition.
#[no_mangle]
pub unsafe extern fn tailor_destroy(instance: *mut Tailor) {
    log::info!("Destroying Tailor instance.");
    let instance = Box::from_raw(instance);
    for (_,client_info) in instance.clients {
        if client_info.tx.send(true).is_err() {
            log::warn!("Failed to send stop to client");
        } else {
            if client_info.handle.join().is_err() {
                log::warn!("Failed to join client thread");
            }
        }
    }
}

#[no_mangle]
pub unsafe extern fn tailor_set_new_lines_callback(instance: *mut Tailor, callback: unsafe extern fn(i32, u32, u32, *const *mut c_char)) {
    let instance = wrap_instance(instance);
    C_CALLBACK = Some(Box::new(move |client_id: i32, msg_type: u32, strings_count: u32, msg: *const *mut c_char| {
        callback(client_id, msg_type, strings_count, msg);
    }));
    unwrap_instance(instance);
}

/// C API: Watch directory or file under given path
///
/// *path* absolute path to the directory or file to watch
///
/// *new_line_callback* function to be called when changes in watched path
///
/// returns true if watch started successfully
///
/// # Safety
///
/// This is a C function, unsafe by definition.
#[no_mangle]
pub unsafe extern fn tailor_watch_path(_instance: *mut Tailor, _path: *const c_char) -> i32 {
    // let mut instance = wrap_instance(instance);

    // let path_c_str = CStr::from_ptr(path);
    // let path_str = path_c_str.to_str();
    // if let Ok(unwrapped_str) = path_str {
    //     let dest_path = PathBuf::from(unwrapped_str);
    //     let client_id = instance.watch(dest_path);
    //     unwrap_instance(instance);
    //     return client_id;
    // } else {
    //     log::info!("C API watch_path: path is broken.");
    // }

    // unwrap_instance(instance);

    INVALID_CLIENT_ID
}

#[no_mangle]
pub unsafe extern fn tailor_stop_watch(instance: *mut Tailor, client_id: i32) {
    let mut instance = wrap_instance(instance);
    instance.stop(client_id);
    unwrap_instance(instance);
}
