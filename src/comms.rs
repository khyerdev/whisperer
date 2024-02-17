use crate::{
    msg, kem,
    tcp::{
        self,
        vector as vect,
        StreamReader
    }
};
use std::{
    io::Write, net::TcpListener, sync::{mpsc, Arc, Mutex}, thread
};
use eframe::egui;

const KEY_SIZE: usize = 16;

pub fn request_handler_thread(win_ctx: egui::Context, sender: mpsc::Sender<msg::Message>) {
    let port = TcpListener::bind("0.0.0.0:9998").unwrap();

    let base_key = Arc::new(vect::rand_byte_vector(KEY_SIZE));
    let private_key: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

    for req in port.incoming() {
        let base_key = Arc::clone(&base_key);
        let private_key = Arc::clone(&private_key);

        let sender = sender.clone();
        let win_ctx = win_ctx.clone();
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

                    let message = msg::Message::new(author, message);

                    sender.send(message).unwrap();
                    win_ctx.request_repaint();
                },
                tcp::Protocol::Unknown => stream.write_all(&[0u8]).unwrap()
            });
        });
    }
}