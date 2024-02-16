//! Wrappers around TcpStream with khyernet's custom protocol
//! 
//! This module will not work properly unless both ends are using the same protocol
//! 
//! why am i doing this guh this module isnt even gonna be available for outside use
pub mod vector;

use vector as vect;
use std::{
    net::TcpStream,
    io::{
        prelude::*,
        Error, ErrorKind
    }
};
use crate::kem;

/// The maximum content length for tcp transfers, data loss will happen if this is exceeded
pub const MAX_CONTENT_LENGTH: usize = 2048;

pub trait StreamReader {
    /// Handle incoming data. You are expected to respond to `PublicKey` by returning a mixed key.
    /// You are also expected to build your private key from `CombineKey`.
    /// This will panic if something goes wrong, I suggest using it in a thread.
    fn parse_incoming(&mut self, action: impl FnOnce(&mut Self, Protocol, Vec<u8>));
}
impl StreamReader for TcpStream {
    fn parse_incoming(&mut self, action: impl FnOnce(&mut Self, Protocol, Vec<u8>)) {
        let mut data = [0u8; MAX_CONTENT_LENGTH];
        self.read(&mut data).unwrap();
        let mut data = data.to_vec();

        match data[0] {
            22u8 => self.write_all(&[6u8]).unwrap(),
            _ => {
                vect::truncate_until_terminator(&mut data, 255u8);
                let protocol = vect::erase_until_terminator(&mut data, 0u8);
                let protocol = vect::bytes_to_string(protocol);
        
                action(self, Protocol::from(protocol), data);
            }
        }
    }
}

/// Check if `ip` has an open port.
pub fn check_availability(ip: &str) -> Result<(), Error> {
    let mut stream = TcpStream::connect(ip)?;
    stream.write_all(&[22u8])?;

    let mut ack = [255u8; 1];
    stream.read(&mut ack)?;

    Ok(ack_response(ack)?)
}
/// Send an encrypted message using khyernet's custom protocol.
pub fn encrypted_send(ip: &str, message: &str, key: Vec<u8>) -> Result<(), Error> {
    let mut stream = TcpStream::connect(ip)?;

    let bytes = vect::bytes_from_string(message);
    let bytes = kem::encrypt(bytes, key);

    stream.write_all(&["MESSAGE\0".as_bytes(), &bytes, &[255u8]].concat())?;

    let mut empty = [255u8; 1];
    stream.read(&mut empty)?;

    Ok(null_response(empty)?)
}
/// Send a public key to the other end, expect the other end's mixed key back.
pub fn send_public_key(ip: &str, key: Vec<u8>) -> Result<Vec<u8>, Error> {
    let mut stream = TcpStream::connect(ip)?;
    
    stream.write_all(&["PUBLICKEY\0".as_bytes(), &key, &[255u8]].concat())?;
        
    let mut recv_key = [0u8; MAX_CONTENT_LENGTH];
    stream.read(&mut recv_key)?;
    let mut recv_key = recv_key.to_vec();
    vect::truncate_until_terminator(&mut recv_key, 255u8);

    Ok(recv_key)
}
/// Send a mixed key to the other end, expect the other end to form their private key.
pub fn send_mixed_key(ip: &str, key: Vec<u8>) -> Result<(), Error> {
    let mut stream = TcpStream::connect(ip)?;
    stream.write_all(&["COMBINEKEY\0".as_bytes(), &key, &[255u8]].concat())?;

    let mut empty = [255u8; 1];
    stream.read(&mut empty).unwrap();
    
    Ok(null_response(empty)?)
}

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

fn null_response(response: [u8; 1]) -> Result<(), Error> {
    match response {
        [0u8] => Ok(()),
        _ => Err(Error::new(ErrorKind::InvalidData, "Receiving end responded incorrectly"))
    }
}

fn ack_response(response: [u8; 1]) -> Result<(), Error> {
    match response {
        [6u8] => Ok(()),
        _ => Err(Error::new(ErrorKind::ConnectionRefused, "Receiving end did not acknowledge"))
    }
}

pub fn get_local_ip() -> String {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.connect("8.8.8.8:80").unwrap();
    socket.local_addr().unwrap().ip().to_string()
}

#[cfg(test)]
mod tests;