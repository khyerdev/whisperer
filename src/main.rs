mod kem;
mod tcp;
mod msg;
mod comms;

use std::{
    sync::mpsc, thread
};
use screen_info::DisplayInfo;
use eframe::egui::{self, Widget};

const WIN_SIZE: [f32; 2] = [600.0, 400.0];

struct MainWindow {
    host: String,
    chat_history: Vec<msg::Message>,
    incoming: mpsc::Receiver<msg::Message>,
    known_peers: Vec<msg::Recipient>,
    current_peer: msg::Recipient,
    draft: String
}
impl MainWindow {
    fn new(
        cc: &eframe::CreationContext<'_>,
        host: String,
        sender: mpsc::Sender<msg::Message>,
        receiver: mpsc::Receiver<msg::Message>
    ) -> Self {
        let ctx = cc.egui_ctx.clone();
        thread::spawn(move || comms::request_handler_thread(ctx, sender));

        let mut peers = Vec::new();
        peers.push(msg::Recipient::from(host.clone()));

        let mut debug = msg::Recipient::from("255.255.255.255");
        debug.set_alias("AAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        peers.push(debug);

        Self {
            host: host.clone(),
            chat_history: Vec::new(),
            incoming: receiver,
            known_peers: peers,
            current_peer: msg::Recipient::from(host),
            draft: String::new()
        }
    }
}
impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.incoming.try_recv() {
            Ok(message) => {
                self.chat_history.push(
                    msg::Message::new(
                        message.author(),
                        message.content()
                    )
                );
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

                let action = match self.current_peer.alias() {
                    Some(_) => "Change",
                    None => "Set"
                };
                if egui::Button::new(format!("{action} alias"))
                    .min_size(egui::vec2(80.0, 15.0))
                    .ui(ui)
                    .clicked()
                {
                    self.current_peer.set_alias("test");
                    msg::modify_alias(self.current_peer.ip(), "test", &mut self.known_peers);
                }
                ui.menu_button("Add", |_ui| {

                });
                let _ = ui.button("Remove");
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
    let (send, recv): (mpsc::Sender<msg::Message>, mpsc::Receiver<msg::Message>) = mpsc::channel();

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