mod kem;
mod tcp;
mod msg;
mod comms;

use std::{
    sync::mpsc, thread
};
use screen_info::DisplayInfo;
use eframe::egui::{self, Widget};
use comms::*;

const WIN_SIZE: [f32; 2] = [500.0, 500.0];

struct MainWindow {
    host: String,
    chat_history: Vec<msg::Message>,
    incoming: mpsc::Receiver<msg::Message>,
    known_peers: Vec<msg::Recipient>,
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
        thread::spawn(move || request_handler_thread(ctx, sender));

        Self {
            host,
            chat_history: Vec::new(),
            incoming: receiver,
            known_peers: Vec::new(),
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

            ui.vertical_centered(|ui| {
                ui.heading(format!("HOST: {}", &self.host));
            });

            ui.separator();

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
                    .max_width(WIN_SIZE[0] - 16.0)
                    .max_height(200.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui|
                {
                    self.chat_history.iter().for_each(|msg| {
                        ui.horizontal_wrapped(|ui| {
                            ui.monospace(egui::RichText::new(
                                format!("[{}]", msg.author())
                            ).color(egui::Color32::LIGHT_RED));
                            ui.monospace(format!(": {}", msg.content()));
                        });
                    });
                })
            );

            ui.horizontal(|ui| {
                egui::TextEdit::singleline(&mut self.draft)
                    .desired_width(382.0)
                    .code_editor()
                    .lock_focus(false)
                    .ui(ui);

                if ui.button("Send Message").clicked() {
                    self.chat_history.push(
                        msg::Message::new(
                            String::from("You"),
                            self.draft.clone()
                        )
                    );
                    self.draft = String::new();
                }
            });
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
        win.max_inner_size = win.min_inner_size;
        win.maximize_button = Some(false);
        win.resizable = Some(false);

        let (centerx, centery) = calculate_center_screen(WIN_SIZE[0], WIN_SIZE[1]);
        win.position = Some(egui::pos2(centerx, centery));

        options.viewport = win;
    }

    eframe::run_native(
        &format!("Whisperer - {}", &host), 
        options, 
        Box::new(|cc| Box::new(MainWindow::new(cc, host, send, recv)))
    ).unwrap_or(());
}

#[inline]
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