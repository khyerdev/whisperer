mod vector;
mod sem;

use vector as vect;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

fn main() {
    let port = TcpListener::bind("0.0.0.0:9998").unwrap();
    
    thread::spawn(|| {
        let mut stream = TcpStream::connect("192.168.40.126:9998").unwrap();

        let public_key = vect::rand_byte_vector(16);
        stream.write_all(&["PUBLICKEY\0".as_bytes(), &public_key, &[255u8]].concat()).unwrap();
        
        let mut recv_key = [0u8; 2048];
        stream.read(&mut recv_key).unwrap();
        let mut recv_key = recv_key.to_vec();
        vect::truncate_until_terminator(&mut recv_key, 255u8);
        drop(stream);

        let mut stream = TcpStream::connect("192.168.40.126:9998").unwrap();

        let base_key = vect::rand_byte_vector(16);
        let combined_key = vect::and_vector(base_key.clone(), public_key);
        stream.write_all(&["COMBINEKEY\0".as_bytes(), &combined_key, &[255u8]].concat()).unwrap();

        let mut empty = [0u8; 1];
        stream.read(&mut empty).unwrap();
        drop(stream);
        assert_eq!(empty, [0u8]);

        let private_key = vect::and_vector(base_key, recv_key);

        println!("{:?}", private_key.clone());

        let mut stream = TcpStream::connect("192.168.40.126:9998").unwrap();

        let message = "you will be forever alone";
        println!("Sent: {message}");
        let bytes = vect::bytes_from_string(message);
        let bytes = sem::encrypt(bytes, private_key);

        stream.write_all(&["MESSAGE\0".as_bytes(), &bytes, &[255u8]].concat()).unwrap();
        let mut empty = [0u8; 1];
        stream.read(&mut empty).unwrap();
        drop(stream);
    });
    
    let base_key = vect::rand_byte_vector(16);
    let mut private_key: Vec<u8> = Vec::new();

    for req in port.incoming() {
        let mut stream = req.unwrap();

        let mut data = [0u8; 2048];
        stream.read(&mut data).unwrap();
        let mut data = data.to_vec();
        vect::truncate_until_terminator(&mut data, 255u8);
        let protocol = vect::erase_until_terminator(&mut data, 0u8);
        let protocol = vect::bytes_to_string(protocol);
        let protocol = protocol.as_str();

        match protocol {
            "PUBLICKEY" => {
                let base = &base_key;
                let combined_key = vect::and_vector(base.clone(), data);
                stream.write_all(&[combined_key.as_slice(), &[255u8]].concat()).unwrap();
            },
            "COMBINEKEY" => {
                private_key = vect::and_vector(base_key.clone(), data);
                stream.write_all(&[0u8]).unwrap();
                println!("{:?}", private_key);
            },
            "MESSAGE" => {
                let key = &private_key;
                let message = sem::decrypt(data, key.clone());
                let message = vect::bytes_to_string(message);
                println!("Got: {message}");
                stream.write_all(&[0u8]).unwrap()
            },
            _ => stream.write_all(&[0u8]).unwrap()
        }        
    }
}

fn _read_buffer(stream: &mut TcpStream) -> String {
    let mut buf: Vec<u8> = Vec::new();
    stream.read(&mut buf).unwrap();
    String::from_utf8_lossy(&buf).to_string()
}