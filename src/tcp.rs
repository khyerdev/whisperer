use std::net::TcpStream;

pub trait StreamWrapper {
    fn send(&mut self, message: &str);
}

impl StreamWrapper for TcpStream {
    fn send(&mut self, message: &str) {
        
    }
}