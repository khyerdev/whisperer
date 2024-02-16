use eframe::egui;

pub struct InfoMessage {
    message: String,
    author: String
}
impl InfoMessage {
    pub fn new(_cc: &eframe::CreationContext<'_>, message: String, author: String) -> Self {
        Self {
            message, author
        }
    }
}
impl eframe::App for InfoMessage {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

        });
    }
}