use crate::{
    msg, kem, Event, KNOWN_PEERS,
    tcp::{
        self,
        vector as vect,
        StreamReader
    }
};
use std::{
    io::Write, net::TcpListener, sync::{mpsc, Arc}, thread
};
use eframe::egui::Context;

const KEY_SIZE: usize = 16;

pub fn request_handler_thread(win_ctx: Context, sender: mpsc::Sender<Event>) {
    let port = TcpListener::bind("0.0.0.0:9998").unwrap();

    let base_key = Arc::new(vect::rand_byte_vector(KEY_SIZE));

    for req in port.incoming() {

        let base_key = Arc::clone(&base_key);

        let sender = sender.clone();
        let win_ctx = win_ctx.clone();
        thread::spawn(move || {
            let mut stream = req.unwrap();

            stream.parse_incoming(|stream, protocol, data| match protocol {
                tcp::Protocol::PublicKey => {
                    println!("GENERATE COMBINED KEY AND SHIP");
                    let combined_key = vect::and_vector(base_key.to_vec(), data);
                    stream.write_all(&[combined_key.as_slice(), &[255u8]].concat()).unwrap();
                },
                tcp::Protocol::CombineKey => {
                    println!("GENERATE PUBLIC KEY FROM COMBINED KEY");
                    let author = stream.peer_addr().unwrap().to_string();
                    let author = trim_port(author);

                    let private_key = vect::and_vector(base_key.to_vec(), data);

                    unsafe {
                        let mut peers = KNOWN_PEERS.write().unwrap();
                        let mut written = false;
                        for peer in peers.iter_mut() {
                            if peer.ip() == author.clone() {
                                written = true;
                                println!("OVERWRITE PRIVATE KEY");
                                peer.set_private_key(private_key.clone());
                                sender.send(Event::OverwritePeer(peer.clone())).unwrap();
                                win_ctx.request_repaint();
                                break
                            }
                        }
                        if !written {
                            println!("SAVE NEW RECIPIENT");
                            let mut incoming = msg::Recipient::from(author);
                            incoming.set_private_key(private_key);
                            peers.push(incoming);
                            drop(peers);
                            sender.send(Event::UpdateChatHistory).unwrap();
                            win_ctx.request_repaint();
                        }
                    }

                    stream.write_all(&[0u8]).unwrap();
                },
                tcp::Protocol::Message => {
                    println!("MESSAGE RECEIVED ON BACKEND");
                    let author = stream.peer_addr().unwrap().to_string();
                    let author = trim_port(author);

                    let mut can_show = true;
                    let key = unsafe {
                        let rlock = KNOWN_PEERS.read().unwrap();
                        let mut key: Option<Vec<u8>> = None;
                        for peer in rlock.iter() {
                            if peer.ip() == author.clone() {
                                println!("FOUND DECRYPT KEY");
                                key = peer.private_key();
                                break
                            }
                        }
                        drop(rlock);
                        let key = match key {
                            Some(k) => k,
                            None => {
                                println!("NO KEY FOUND, REBUILDING");
                                can_show = false;
                                let new_key = make_keypair(author.clone()).unwrap();
                                let mut wlock = KNOWN_PEERS.write().unwrap();
                                let mut existing = false;
                                for peer in wlock.iter_mut() {
                                    if peer.ip() == author.clone() {
                                        println!("REWRITING KEY");
                                        peer.set_private_key(new_key.clone());
                                        existing = true;
                                        break
                                    }
                                }
                                drop(wlock);
                                if !existing {
                                    println!("ADDING NEW RECIPIENT FROM REBUILT KEY");
                                    let mut addition = msg::Recipient::from(author.clone());
                                    addition.set_private_key(new_key.clone());
                                    KNOWN_PEERS.write().unwrap().push(addition);
                                }
                                new_key
                            }
                        };
                        key
                    };

                    stream.write_all(&[0u8]).unwrap();
                    if can_show {
                        let message = kem::decrypt(data, key);
                        let message = vect::bytes_to_string(message);
                        let message = msg::Message::new(author, message);
                        sender.send(Event::IncomingMsg(message)).unwrap();
                        win_ctx.request_repaint();
                    } else {
                        println!("REQUEST RESEND");
                        tcp::request_resend(&format!("{author}:9998")).unwrap();
                    }
                },
                tcp::Protocol::Resend => {
                    let author = stream.peer_addr().unwrap().to_string();
                    let author = trim_port(author);
                    stream.write_all(&[0u8]).unwrap();

                    sender.send(Event::ResendLast(author)).unwrap();
                    win_ctx.request_repaint();
                },
                tcp::Protocol::Unknown => stream.write_all(&[1u8]).unwrap()
            });
        });
    }
}

pub fn send_message(peer: msg::Recipient, msg: String, key_callback: mpsc::Sender<Event>, ctx_update: Context) {
    let ip = format!("{}:9998", peer.ip());
    let key = match peer.private_key() {
        Some(key) => key,
        None => {
            let key = make_keypair(&ip).unwrap();
            key_callback.send(Event::StoreKey(ip.clone(), key.clone())).unwrap();
            ctx_update.request_repaint();
            key
        }
    };

    tcp::encrypted_send(&ip, &msg, key).unwrap();
}

pub fn make_keypair(ip: impl ToString) -> Result<Vec<u8>, std::io::Error> {
    let ip = format!("{}:9998", ip.to_string());

    let base_key = vect::rand_byte_vector(KEY_SIZE);
    let public_key = vect::rand_byte_vector(KEY_SIZE);
    let mixed_key = tcp::send_public_key(&ip, public_key.clone())?;

    let combined_key = vect::and_vector(base_key.clone(), public_key);
    tcp::send_mixed_key(&ip, combined_key)?;

    Ok(vect::and_vector(mixed_key, base_key))
}

#[inline(always)]
fn trim_port(ip: String) -> String {
    let parts: Vec<&str> = ip.split_terminator(':').collect();
    parts[0].to_string()
}