#![allow(unused)] // because rust-analyzer is smarter than gpt-4.5 and can clearly understand that im using everything
use crate::msg;
use std::{path::PathBuf, fs, env::var};

const KEY_SIZE: usize = 16;

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
                if &msg.author() == "You" { entry.push(0u8 as char) }
                entry.push_str(&msg.content());

                entry.push('\n');
            }

            entry.push('\n');
            entry.bytes().for_each(|b| buf.push(b));
        }

        buf
    }).unwrap()
}

pub fn get_data() -> (Vec<msg::Recipient>, Vec<msg::ChatHistory>) {
    let root = root_path().unwrap(); // will panic on macos
    let mut recipients: Vec<msg::Recipient> = Vec::new();
    let mut histories: Vec<msg::ChatHistory> = Vec::new();

    let s = std::path::MAIN_SEPARATOR;
    let recipient_file = PathBuf::from(format!("{}{s}RECIPIENTS", root.to_string_lossy()));
    let history_file = PathBuf::from(format!("{}{s}HISTORY", root.to_string_lossy()));

    if recipient_file.exists() && history_file.exists() {
        if let Ok(recipient_data) = fs::read(recipient_file) {
            let mut ip = String::new();
            let mut alias = String::new();
            let mut key: Vec<u8> = Vec::new();
            let mut mode: u8 = 0;

            for byte in recipient_data.iter() {
                match mode {
                    0 => {
                        if byte == &0 {
                            mode = 1
                        } else {
                            ip.push(*byte as char);
                        }
                    },
                    1 => {
                        if byte == &0 {
                            mode = 2
                        } else {
                            alias.push(*byte as char);
                        }
                    },
                    2 => {
                        if byte == &10 {
                            let mut rec = msg::Recipient::from(ip.clone());
                            if alias.len() > 0 { rec.set_alias(Some(alias.clone())) }
                            if key.len() == KEY_SIZE { rec.set_private_key(key.clone()) }

                            ip.clear();
                            alias.clear();
                            key.clear();

                            recipients.push(rec);
                            mode = 0;
                        } else { key.push(*byte) }
                    },
                    _ => unreachable!()
                }
            }
        } else { return (Vec::new(), Vec::new()); }

        if let Ok(history_data) = fs::read(history_file) {
            let mut ip = String::new();
            let mut message_read = String::new();
            let mut messages: Vec<msg::Message> = Vec::new();
            let mut newline_count: u8 = 0;
            let mut mode: u8 = 0;
            let mut you = false;
            
            for byte in history_data.iter() {
                match mode {
                    0 => {
                        if byte == &10 {
                            mode = 1
                        } else {
                            ip.push(*byte as char)
                        }
                    },
                    1 => {
                        if byte == &10 {
                            newline_count += 1;
                            if newline_count == 2 {
                                let rec_clone = {
                                    let mut rec_clone: Option<msg::Recipient> = None;
                                    for rec in recipients.iter() {
                                        if &rec.ip() == &ip {
                                            rec_clone = Some(rec.clone());
                                            break
                                        }
                                    }
                                    rec_clone.unwrap() // will panic if recipients isnt read to memory properly
                                };

                                let mut chathistory = msg::ChatHistory::new(rec_clone);
                                for message in messages.iter() {
                                    chathistory.push_msg(message.clone());
                                }
                                histories.push(chathistory);

                                ip.clear();
                                message_read.clear();
                                messages.clear();
                                newline_count = 0;
                                mode = 0;
                            } else {
                                let author = match you {
                                    true => String::from("You"),
                                    false => ip.clone(),
                                };
                                messages.push(msg::Message::new(author, message_read.clone()));
                                message_read.clear();
                            }
                            you = false;
                        } else {
                            newline_count = 0;
                            if byte == &0 {
                                you = true;
                            } else {
                                message_read.push(*byte as char);
                            }
                        }
                    },
                    _ => unreachable!()
                }
            }
        } else { return (Vec::new(), Vec::new()); }
    }

    (recipients, histories)
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