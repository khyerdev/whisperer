mod kem;
mod tcp;
mod msg;
mod comms;

use std::{sync::mpsc, thread};
use screen_info::DisplayInfo;
use eframe::egui::{self, Widget};

const WIN_SIZE: [f32; 2] = [600.0, 400.0];

struct MainWindow {
    host: String,
    chat_history: Vec<msg::Message>,
    new_event: mpsc::Sender<Event>,
    listener: mpsc::Receiver<Event>,
    known_peers: Vec<msg::Recipient>,
    current_peer: msg::Recipient,
    draft: String,
    new_alias: String,
    new_peer: String,
    thinking: bool
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

        let mut peers = Vec::new();
        peers.push(msg::Recipient::from(host.clone()));

        // 28 character limit
        let mut debug = msg::Recipient::from("255.255.255.255");
        debug.set_alias(Some(String::from("AAAAAAAAAAAAAAAAAAAAAAAAAAAA")));
        peers.push(debug);

        Self {
            host: host.clone(),
            chat_history: Vec::new(),
            new_event: sender,
            listener: receiver,
            known_peers: peers,
            current_peer: msg::Recipient::from(host),
            draft: String::new(),
            new_alias: String::new(),
            new_peer: String::new(),
            thinking: false
        }
    }
}
impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.listener.try_recv() {
            Ok(event) => match event {
                Event::IncomingMsg(msg) => {
                    self.chat_history.push(
                        msg::Message::new(
                            msg.author(),
                            msg.content()
                        )
                    );
                }
                Event::StoreKey(_) => todo!(),
                Event::NewPeerResult(_) => todo!(),
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
                    for peer in self.known_peers.iter() {
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
                
                ui.menu_button(format!("{s}{action} alias{s}"), |ui| {
                    let l = self.new_alias.len();
                    let col = match l {
                        0..=23 => egui::Color32::GRAY,
                        24..=28 => egui::Color32::YELLOW,
                        _ => egui::Color32::RED,
                    };

                    ui.text_edit_singleline(&mut self.new_alias);
                    ui.horizontal(|ui| {
                        if ui.add_enabled(l > 0 && l <= 28, egui::Button::new(format!("{action}"))).clicked() {
                            self.current_peer.set_alias(Some(self.new_alias.clone()));
                            msg::modify_alias(self.current_peer.ip(), Some(self.new_alias.clone()), &mut self.known_peers);

                            self.new_alias.clear();
                            ui.close_menu();
                        }
                        if ui.add_enabled(action == "Change", egui::Button::new("Remove")).clicked() {
                            self.current_peer.set_alias(None);
                            msg::modify_alias(self.current_peer.ip(), None, &mut self.known_peers);

                            self.new_alias.clear();
                            ui.close_menu();
                        }
                        ui.label(egui::RichText::new(format!("{l}/28")).color(col));
                    });
                });

                ui.menu_button("Add", |ui| {
                    let l = self.new_peer.len();
                    ui.text_edit_singleline(&mut self.new_peer);
                    ui.horizontal(|ui| {
                        if ui.add_enabled(l > 0 && l <= 28 && msg::is_valid_ip(&self.new_peer), egui::Button::new(format!("Verify and add"))).clicked() {
                            self.thinking = true;

                            match tcp::check_availability(&format!("{}:9998", &self.new_peer)) {
                                Ok(_) => {
                                    self.new_peer.clear();
                                    ui.close_menu();
                                },
                                Err(_) => self.new_peer = String::from("FAIL: Offline/invalid IP"),
                            }

                            self.thinking = false;
                            
                        }
                    });
                    if self.thinking {
                        ui.spinner();
                    }
                });

                ui.menu_button("Remove", |ui| {

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
                    self.chat_history.iter().for_each(|msg| {
                        let col = match msg.author().as_str() {
                            "You" => egui::Color32::LIGHT_BLUE,
                            _ => egui::Color32::LIGHT_RED
                        };

                        let author = match msg::find_alias(msg.author(), &self.known_peers) {
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
                })
            );

            let l = self.draft.len();
            ui.horizontal(|ui| {
                egui::TextEdit::singleline(&mut self.draft)
                    .desired_width(width - 102.0)
                    .code_editor()
                    .lock_focus(false)
                    .ui(ui);

                ui.add_enabled_ui(l > 0 && l <= 2000, |ui|
                    if ui.button("Send Message").clicked() {
                        self.chat_history.push(
                            msg::Message::new(
                                String::from("You"),
                                self.draft.clone()
                            )
                        );
                        
                        let peer = self.current_peer.clone();
                        let msg = self.draft.clone();
                        thread::spawn(move || comms::send_message(peer, msg));
                        
                        self.draft.clear();
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
}

fn main() {
    let (send, recv): (mpsc::Sender<Event>, mpsc::Receiver<Event>) = mpsc::channel();

    let host = tcp::get_local_ip();
    
    let mut options = eframe::NativeOptions::default();
    {
        let mut win = egui::ViewportBuilder::default();
        win.min_inner_size = Some(egui::vec2(WIN_SIZE[0], WIN_SIZE[1]));
        win.inner_size = Some(egui::vec2(WIN_SIZE[0], WIN_SIZE[1]));

        let (centerx, centery) = calculate_center_screen(WIN_SIZE[0], WIN_SIZE[1]);
        win.position = Some(egui::pos2(centerx, centery));
        //win.resizable = Some(false);

        options.viewport = win;
    }

    eframe::run_native(
        "Whisperer", 
        options, 
        Box::new(|cc| Box::new(MainWindow::new(cc, host, send, recv)))
    ).unwrap_or(());
}

// because complexity
#[inline(always)]
fn calculate_center_screen(x: f32, y: f32) -> (f32, f32) {
    let display = DisplayInfo::all().unwrap();
    let mut res: Option<(u32, u32)> = None;
    for info in display {
        match info.is_primary {
            true => {
                res = Some((info.width, info.height));
                break;
            },
            false => ()
        }
    }
    let size = res.expect("No primary monitor");
    (
        {size.0 as f32 / 2.0 - {x / 2.0}},
        {size.1 as f32 / 2.0 - {y / 2.0}}
    )
}

enum Event {
    IncomingMsg(msg::Message),
    StoreKey(Vec<u8>),
    NewPeerResult(Option<msg::Recipient>)
}