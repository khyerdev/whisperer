#![allow(unused)] // because rust-analyzer is smarter than gpt-4.5 and can clearly understand that im using everything
use crate::msg;
use std::{path::PathBuf, fs, env::var};

pub fn set_data(recipient_list: Vec<msg::Recipient>, chat_history: Vec<msg::ChatHistory>) {
    let path = root_path().unwrap(); // will panic on macos
    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }

    let s = std::path::MAIN_SEPARATOR;

    fs::write(format!("{}{s}RECIPIENTS", path.to_string_lossy()), {
        let mut buf: Vec<u8> = Vec::new();

        for rec in recipient_list.iter() {
            if &rec.ip() == "None" {continue}
            let mut entry = String::new();
            entry.push_str(&rec.ip());
            entry.push('\0');

            if let Some(alias) = rec.alias() {
                entry.push_str(&alias)
            }
            entry.push('\0');

            entry.bytes().for_each(|b| buf.push(b));
            if let Some(key) = rec.private_key() {
                key.iter().for_each(|b| buf.push(*b));
            }

            buf.push(10);
        }
        buf.pop().unwrap();

        buf
    }).unwrap();

    fs::write(format!("{}{s}HISTORY", path.to_string_lossy()), {
        let mut buf: Vec<u8> = Vec::new();

        for history in chat_history.iter() {
            if &history.peer().ip() == "None" {continue}
            let mut entry = String::new();
            entry.push_str(&history.peer().ip());
            entry.push('\n');

            for msg in history.history().iter() {
                if &msg.author() == "You" { entry.push(255u8 as char) }
                entry.push_str(&msg.content());

                entry.push('\n');
            }

            entry.push('\n');
            entry.bytes().for_each(|b| buf.push(b));
        }
        buf.pop().unwrap();

        buf
    }).unwrap()
}

fn root_path() -> Option<PathBuf> {
    let mut path: Option<PathBuf> = None;
    #[cfg(target_os = "linux")]
    {
        let home = var("HOME").unwrap();
        path = Some(PathBuf::from(format!("{home}/.local/share/whisperer")));
    }
    #[cfg(target_os = "windows")]
    {
        let local = var("LOCALAPPDATA").unwrap();
        path = Some(PathBuf::from(format!("{local}/whisperer")));
    }
    path
}