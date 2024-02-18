mod kem;
mod tcp;
mod msg;
mod comms;

use std::{sync::{mpsc, RwLock}, thread};
use eframe::egui;
use once_cell::sync::Lazy;

const WIN_SIZE: [f32; 2] = [600.0, 400.0];
static mut KNOWN_PEERS: Lazy<RwLock<Vec<msg::Recipient>>> = Lazy::new(|| {
    let mut vec: Vec<msg::Recipient> = Vec::new();
    vec.push(msg::Recipient::from("None"));
    RwLock::new(vec)
});

struct MainWindow {
    host: String,
    chat_history: Vec<msg::ChatHistory>,
    new_event: mpsc::Sender<Event>,
    listener: mpsc::Receiver<Event>,
    current_peer: msg::Recipient,
    draft: String,
    new_alias: String,
    new_peer: String,
    thinking: bool,
    sending: bool,
    confirm_remove: bool
}
impl MainWindow {
    fn new(
        cc: &eframe::CreationContext<'_>,
        host: String,
        sender: mpsc::Sender<Event>,
        receiver: mpsc::Receiver<Event>
    ) -> Self {
        let ctx = cc.egui_ctx.clone();
        let send = sender.clone();
        thread::spawn(move || comms::request_handler_thread(ctx, send));

        let mut starting_peers = Vec::new();
        starting_peers.push(msg::Recipient::from("None"));

        let first = starting_peers.first().unwrap();
        let first = first.clone();

        let mut starting_history: Vec<msg::ChatHistory> = Vec::new();
        starting_history.push(msg::ChatHistory::new(msg::Recipient::from("None")));

        // TODO: read from a file

        Self {
            host: host.clone(),
            chat_history: starting_history,
            new_event: sender,
            listener: receiver,
            current_peer: first,
            draft: String::new(),
            new_alias: String::new(),
            new_peer: String::new(),
            thinking: false,
            sending: false,
            confirm_remove: false
        }
    }
}
impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.listener.try_recv() {
            Ok(event) => match event {
                Event::IncomingMsg(msg) => {
                    let mut retries = 0;
                    'retry_loop: loop {
                        for history in self.chat_history.iter_mut() {
                            if history.peer().ip() == msg.author() {
                                history.push_msg(msg.clone());
                                break 'retry_loop;
                            }
                        }
                        let peers = unsafe {KNOWN_PEERS.read().unwrap().clone()};
                        msg::try_refresh_history_list(&mut self.chat_history, &peers, true);
                        retries += 1;
                        if retries == 2 {break} // womp womp no message for you
                    }
                }
                Event::StoreKey(ip, key) => {
                    unsafe {
                        for peer in KNOWN_PEERS.write().unwrap().iter_mut() {
                            if peer.ip() == ip {
                                peer.set_private_key(key);
                                break
                            }
                        }
                    }
                },
                Event::NewPeerResult(rec) => {
                    match rec {
                        Some(rec) => {
                            self.chat_history.push(msg::ChatHistory::new(rec.clone()));
                            self.new_peer = String::from("SUCCESS");
                            self.current_peer = rec;
                        },
                        None => self.new_peer = String::from("FAIL: Offline/invalid IP")
                    }
                    self.thinking = false;
                },
                Event::SendMessage(success) => {
                    if !success {
                        self.sending = false;
                    } else {
                        for history in self.chat_history.iter_mut() {
                            if history.peer() == self.current_peer {
                                history.push_msg(
                                    msg::Message::new(
                                        String::from("You"),
                                        self.draft.clone()
                                    )
                                );
                                break
                            }
                        }
                        
                        let peer = self.current_peer.clone();
                        let msg = self.draft.clone();
                        let callback = self.new_event.clone();
                        let ctx_update = ctx.clone();
                        thread::spawn(move || comms::send_message(peer, msg, callback, ctx_update));
                        
                        self.draft.clear();
                        self.sending = false;
                    }
                }
                Event::ConfirmationExpired => self.confirm_remove = false
            }
            Err(_) => ()
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let width = ui.available_width();
            let height = ui.available_height();

            ui.vertical_centered(|ui| {
                ui.heading(format!("Whisperer @ {} on {:?}", &self.host, ctx.os()));
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Peer:");
                egui::ComboBox::from_id_source("choose-peer")
                    .width(width - 225.0)
                    .selected_text(egui::RichText::new(self.current_peer.full_string()).monospace())
                    .show_ui(ui, |ui|
                {
                    for peer in unsafe {KNOWN_PEERS.read().unwrap().iter()} {
                        ui.selectable_value(
                            &mut self.current_peer,
                            peer.clone(),
                            egui::RichText::new(peer.full_string()).monospace()
                        );
                    }
                });

                let (action, s) = match self.current_peer.alias() {
                    Some(_) => ("Change", ""),
                    None => ("Set", "    ")
                };
                
                ui.add_enabled_ui(&self.current_peer.ip() != "None", |ui| {
                    ui.menu_button(format!("{s}{action} alias{s}"), |ui| {
                        let l = self.new_alias.len();
                        let col = match l {
                            0..=23 => egui::Color32::GRAY,
                            24..=28 => egui::Color32::YELLOW,
                            _ => egui::Color32::RED,
                        };

                        ui.text_edit_singleline(&mut self.new_alias);
                        ui.horizontal(|ui| {
                            if ui.add_enabled(l > 0 && l <= 28 && &self.new_alias != "You", egui::Button::new(format!("{action}"))).clicked() {
                                self.current_peer.set_alias(Some(self.new_alias.clone()));
                                unsafe {
                                    msg::modify_alias(self.current_peer.ip(), Some(self.new_alias.clone()), &mut KNOWN_PEERS.write().unwrap());
                                }
                                for history in self.chat_history.iter_mut() {
                                    if history.peer().ip() == self.current_peer.ip() {
                                        history.update_peer(self.current_peer.clone());
                                    }
                                }
                                self.new_alias.clear();
                                ui.close_menu();
                            }
                            if ui.add_enabled(action == "Change", egui::Button::new("Remove")).clicked() {
                                self.current_peer.set_alias(None);
                                unsafe {
                                    msg::modify_alias(self.current_peer.ip(), None, &mut KNOWN_PEERS.write().unwrap());
                                }
                                for history in self.chat_history.iter_mut() {
                                    if history.peer().ip() == self.current_peer.ip() {
                                        history.update_peer(self.current_peer.clone());
                                    }
                                }
                                self.new_alias.clear();
                                ui.close_menu();
                            }
                            ui.label(egui::RichText::new(format!("{l}/28")).color(col));
                        });
                    });
                });

                ui.menu_button("Add", |ui| {
                    ui.add_enabled(!self.thinking, egui::TextEdit::singleline(&mut self.new_peer));
                    ui.horizontal(|ui| {
                        if ui.add_enabled(msg::is_valid_ip(&self.new_peer) && !self.thinking, egui::Button::new(format!("Verify and add"))).clicked() {
                            self.thinking = true;
                            let mut alread_exists = false;

                            unsafe {
                                for peer in KNOWN_PEERS.read().unwrap().iter() {
                                    if peer.ip() == self.new_peer.clone() {
                                        alread_exists = true;
                                        break
                                    }
                                }
                            }

                            if alread_exists {
                                self.new_peer = String::from("IP already added");
                                self.thinking = false;
                            } else {
                                let ip = self.new_peer.clone();
                                let sender = self.new_event.clone();
                                let update_ctx = ctx.clone();
                                
                                thread::spawn(move || {
                                    match tcp::check_availability(&format!("{}:9998", ip.clone())) {
                                        Ok(_) => {
                                            match comms::make_keypair(ip.clone()) {
                                                Ok(key) => {
                                                    let mut rec = msg::Recipient::from(ip);
                                                    rec.set_private_key(key);
                                                    sender.send(Event::NewPeerResult(Some(rec))).unwrap();
                                                },
                                                Err(_) => sender.send(Event::NewPeerResult(None)).unwrap()
                                            }
                                        },
                                        Err(_) => sender.send(Event::NewPeerResult(None)).unwrap(),
                                    }
                                    update_ctx.request_repaint();
                                });
                            }

                        }
                        if self.thinking {
                            ui.spinner();
                        }
                    });
                });

                ui.add_enabled_ui(&self.current_peer.ip() != "None", |ui| {
                    ui.menu_button("Remove", |ui| {
                        if ui.button("Delete chat history").clicked() {
                            for history in self.chat_history.iter_mut() {
                                if &history.peer() == &self.current_peer {
                                    history.clear_history();
                                    break
                                }
                            }
                            ui.close_menu();
                        }
                        match self.confirm_remove {
                            true => if ui.button("Are you sure?").clicked() {
                                self.confirm_remove = false;
                                unsafe {
                                    let peers_rwlock = KNOWN_PEERS.read().unwrap();
                                    let peers = peers_rwlock.clone();
                                    drop(peers_rwlock);
                                    for (i, peer) in peers.iter().enumerate() {
                                        if peer == &self.current_peer {
                                            KNOWN_PEERS.write().unwrap().remove(i);
                                            for (i, history) in self.chat_history.iter().enumerate() {
                                                if &history.peer() == peer {
                                                    self.chat_history.remove(i);
                                                    break
                                                }
                                            }
                                            self.current_peer = peers[0].clone();
                                            break
                                        }
                                    }
                                }
                                ui.close_menu();
                            },
                            false => if ui.button("Actually remove").clicked() {
                                self.confirm_remove = true;
                                let future_call = self.new_event.clone();
                                let update_ctx = ctx.clone();
                                thread::spawn(move || {
                                    thread::sleep(std::time::Duration::from_secs(1));
                                    future_call.send(Event::ConfirmationExpired).unwrap();
                                    update_ctx.request_repaint();
                                });
                            }
                        }
                    });
                });
            });

            let mut margin = egui::Margin::default();
            margin.top = 5.0;
            margin.bottom = 5.0;
            margin.left = 2.0;
            margin.right = 2.0;
            let rounding = egui::Rounding::default().at_least(5.0);

            egui::Frame::none()
                .fill(egui::Color32::BLACK)
                .inner_margin(margin)
                .rounding(rounding)
                .show(ui, |ui| 
                    egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .max_width(width)
                    .max_height(height - 100.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui|
                {
                    for history in self.chat_history.iter() {
                        if history.peer() == self.current_peer {
                            history.history().iter().for_each(|msg| {
                                let col = match msg.author().as_str() {
                                    "You" => egui::Color32::LIGHT_BLUE,
                                    _ => egui::Color32::LIGHT_RED
                                };
        
                                let author = match msg::find_alias(msg.author(), unsafe {&KNOWN_PEERS.read().unwrap()}) {
                                    Some(alias) => alias,
                                    None => msg.author()
                                };
        
                                ui.horizontal_wrapped(|ui| {
                                    ui.monospace(egui::RichText::new(
                                        format!("[{}]", author)
                                    ).color(col));
                                    ui.monospace(msg.content());
                                });
                            });
                            break
                        }
                    }
                })
            );

            let l = self.draft.len();
            ui.horizontal(|ui| {
                ui.add_enabled(!self.sending, 
                    egui::TextEdit::singleline(&mut self.draft)
                        .desired_width(width - 102.0)
                        .code_editor()
                        .lock_focus(false)
                );

                ui.add_enabled_ui(l > 0 && l <= 2000 && self.current_peer.ip() != String::from("None") && !self.sending, |ui|
                    if ui.button("Send Message").clicked() {
                        self.sending = false;
                        let ip = self.current_peer.ip();
                        let sender = self.new_event.clone();
                        let update_ctx = ctx.clone();
                        thread::spawn(move || {
                            match tcp::check_availability(ip.as_str()) {
                                Ok(()) => sender.send(Event::SendMessage(true)).unwrap(),
                                Err(_) => sender.send(Event::SendMessage(false)).unwrap()
                            }
                            update_ctx.request_repaint();
                        });
                    }
                );
            });

            let col = match l {
                0..=1799 => egui::Color32::GRAY,
                1800..=2000 => egui::Color32::YELLOW,
                _ => egui::Color32::RED,
            };
            ui.label(egui::RichText::new(format!("{l}/2000")).color(col));
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // TODO: save to a file
    }
}

fn main() {
    let (send, recv) = mpsc::channel::<Event>();

    let host = tcp::get_local_ip();
    
    let mut options = eframe::NativeOptions::default();
    options.centered = true;
    {
        let mut win = egui::ViewportBuilder::default();
        win.min_inner_size = Some(egui::vec2(WIN_SIZE[0], WIN_SIZE[1]));
        win.inner_size = Some(egui::vec2(WIN_SIZE[0], WIN_SIZE[1]));

        options.viewport = win;
    }

    eframe::run_native(
        "Whisperer", 
        options, 
        Box::new(|cc| Box::new(MainWindow::new(cc, host, send, recv)))
    ).unwrap_or(());
}

enum Event {
    IncomingMsg(msg::Message),
    StoreKey(String, Vec<u8>),
    NewPeerResult(Option<msg::Recipient>),
    SendMessage(bool),
    ConfirmationExpired
}