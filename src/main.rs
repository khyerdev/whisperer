mod kem;
mod tcp;
mod msg;

use tcp::{
    vector as vect,
    StreamReader
};
use std::{
    net::TcpListener,
    io::Write,
    thread,
    sync::{Arc, Mutex}
};
use screen_info::DisplayInfo;
use eframe::egui::{self, Widget};

const KEY_SIZE: usize = 16;
const WIN_SIZE: [f32; 2] = [500.0, 500.0];

struct MainWindow {
    host: String,
    chat_history: Vec<String>,
    known_peers: Vec<msg::Recipient>
}
impl MainWindow {
    fn new(_cc: &eframe::CreationContext<'_>, host: String) -> Self {
        Self {
            host,
            chat_history: Vec::new(),
            known_peers: Vec::new()
        }
    }
}
impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                    self.chat_history.iter().for_each(|s| {
                        ui.monospace(s);
                    });
                })
            );

            ui.vertical_centered(|ui| {
                if egui::Button::new(
                    egui::RichText::new("I AM SEND")
                        .size(24.0)
                ).min_size(egui::vec2(400.0, 50.0))
                    .ui(ui)
                    .clicked()
                {
                    self.chat_history.push(format!("[{}]: kill yourself", &self.host));
                }
            });
        });
    }
}

fn main() {
    thread::spawn(request_handler_thread);

    let host = tcp::get_local_ip();
    
    let mut options = eframe::NativeOptions::default();
    options.follow_system_theme = true;
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

    eframe::run_native("I AM IP ADDRESS", options, Box::new(|cc| Box::new(MainWindow::new(cc, host)))).unwrap_or(());
}

fn request_handler_thread() {
    let port = TcpListener::bind("0.0.0.0:9998").unwrap();

    let base_key = Arc::new(vect::rand_byte_vector(KEY_SIZE));
    let private_key: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

    for req in port.incoming() {
        let base_key = Arc::clone(&base_key);
        let private_key = Arc::clone(&private_key);

        thread::spawn(move || {
            let mut stream = req.unwrap();

            stream.parse_incoming(|stream, protocol, data| match protocol {
                tcp::Protocol::PublicKey => {
                    let combined_key = vect::and_vector(base_key.to_vec(), data);
                    stream.write_all(&[combined_key.as_slice(), &[255u8]].concat()).unwrap();
                },
                tcp::Protocol::CombineKey => {
                    let mut mutex = private_key.lock().unwrap();
                    *mutex = vect::and_vector(base_key.to_vec(), data);
                    drop(mutex);

                    stream.write_all(&[0u8]).unwrap();
                },
                tcp::Protocol::Message => {
                    let key = {
                        let mutex = private_key.lock().unwrap();
                        mutex.clone()
                    };

                    let author = stream.peer_addr().unwrap().to_string();

                    let message = kem::decrypt(data, key);
                    let message = vect::bytes_to_string(message);
                    stream.write_all(&[0u8]).unwrap();

                    println!("{author}: {message}");
                },
                tcp::Protocol::Unknown => stream.write_all(&[0u8]).unwrap()
            });
        });
    }
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