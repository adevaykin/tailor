use std::env;
use std::path::{PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use libtailor::Tailor;

use colored::*;

enum MessageType {
    Normal,
    Debug,
    Warning,
    Error,
}

fn get_message_type(msg: &String) -> MessageType {
    if msg.contains("DEBUG") || msg.contains("debug") {
        return MessageType::Debug;
    }

    if msg.contains("WARNING") || msg.contains("WARN") || msg.contains("warning") {
        return MessageType::Warning;
    }

    if msg.contains("ERROR") || msg.contains("ERR") || msg.contains("error") || msg.contains("Error") {
        return MessageType::Error;
    }

    MessageType::Normal
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Specify path to file or directory as the argument.");
        return;
    }

    let mut instance = Tailor::new();
    Tailor::set_new_lines_callback(
        Box::new(|client_id, msg_type, msg| {
            if msg_type == tailor::MSG_TYPE_ADD_LINES {
                for string in msg {
                    match get_message_type(&string) {
                        MessageType::Debug => println!("{}", string.cyan()),
                        MessageType::Warning => println!("{}", string.black().on_yellow()),
                        MessageType::Error => println!("{}", string.black().on_red()),
                        _ => println!("{}", string),
                    }
                }
            }
        })
    );

    let watch_path = PathBuf::from(args[1].as_str());
    let client_id = instance.watch(watch_path);

    let (tx,rx) = channel();
    ctrlc::set_handler(move || {
        tx.send(()).expect("Failed to send Ctrl+C signal.");
    }).expect("Failed to set Ctrl+C handler.");
    loop {
        match rx.recv_timeout(Duration::from_secs(3)) {
            Ok(_) => {
                instance.stop(client_id);
                break;
            },
            Err(_) => ()
        }
    }
}
