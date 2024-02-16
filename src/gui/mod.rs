mod interactive;
mod messagebox;

use eframe::egui;
use screen_info::DisplayInfo;

pub use Type::*;
pub enum Type {
    MessageBox,
    Interactive,
    ErrorMessage
}
impl Type {
    pub fn show(&self, author: String, content: String) {
        let result = match self {
            MessageBox => {
                let mut options = eframe::NativeOptions::default();
                options.follow_system_theme = true;
                {
                    let mut win = egui::ViewportBuilder::default();
                    win.min_inner_size = Some(egui::vec2(400.0, 200.0));
                    win.max_inner_size = win.min_inner_size;
                    win.maximize_button = Some(false);
                    win.resizable = Some(false);

                    let (centerx, centery) = calculate_center_screen(400, 200);
                    win.position = Some(egui::pos2(centerx, centery));

                    options.viewport = win;
                }

                eframe::run_native("Message", options, Box::new(|cc| Box::new(interactive::InfoMessage::new(cc, content, author)))).unwrap_or(());
            },
            Interactive => {

            },
            ErrorMessage => {

            }
        };
    }
}

#[inline]
fn calculate_center_screen(x: u32, y: u32) -> (f32, f32) {
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
        {size.0 / 2 - {x / 2}} as f32,
        {size.1 / 2 - {y / 2}} as f32
    )
}