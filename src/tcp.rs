//! Wrappers around TcpStream with khyernet's custom protocol
//! 
//! This module will not work properly unless both ends are using the same protocol
//! 
//! why am i doing this guh this module isnt even gonna be available for outside use
use std::net::TcpStream;
use std::io::prelude::*;

/// The maximum content length for tcp transfers, data loss will happen if this is exceeded
pub const MAX_CONTENT_LENGTH: usize = 2048;

pub trait StreamReader {
    /// Handle incoming data. You are expected to respond to `PublicKey` by returning a mixed key.
    /// You are also expected to build your private key from `CombineKey`.
    fn parse_incoming(&mut self, action: impl FnOnce(Protocol, Vec<u8>));
}
impl StreamReader for TcpStream {
    fn parse_incoming(&mut self, action: impl FnOnce(Protocol, Vec<u8>)) {
        todo!()
    }
}

/// Check if `ip` has an open port.
/// Run `error_handler` if it fails
pub fn check_availability(ip: &str, error_handler: impl FnOnce()) {
    match TcpStream::connect(ip) {
        Ok(mut stream) => match stream.write_all(&[22u8]) {
            Ok(_) => {
                let mut ack = [0u8; 1];
                match stream.read(&mut ack) {
                    Ok(_) => match ack {
                        [6u8] => (),
                        _ => error_handler()
                    },
                    Err(_) => error_handler()
                }
            },
            Err(_) => error_handler()
        }
        Err(_) => error_handler()
    }
}
/// Send an encrypted message using khyernet's custom protocol.
/// Run `error_handler` if it fails
//pub fn send_encrypted(ip: &str, message: &str, error_handler: impl FnOnce()) {}
/// Send a public key to the other end, expect the other end's mixed key back.
/// Run `error_handler` if it fails
//pub fn send_public_key(ip: &str, key: Vec<u8>, error_handler: impl FnOnce()) -> Vec<u8> {}
/// Send a mixed key to the other end, expect the other end to form their private key.
/// Run `error_handler` if it fails
//pub fn send_mixed_key(ip: &str, key: Vec<u8>, error_handler: impl FnOnce()) {}

pub enum Protocol {
    PublicKey, CombineKey, Message, Unknown
}
impl From<String> for Protocol {
    fn from(value: String) -> Self {
        let value = value.as_str();
        match value {
            "PUBLICKEY" => Self::PublicKey,
            "COMBINEKEY" => Self::CombineKey,
            "MESSAGE" => Self::Message,
            _ => Self::Unknown
        }
    }
}