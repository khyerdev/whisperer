mod vector;
mod sem;

use vector as vect;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

const MAX_BUF_SIZE: usize = 1024usize;

fn main() {
    let port = TcpListener::bind("0.0.0.0:9998").unwrap();
    
    thread::spawn(|| {
        let mut connection = TcpStream::connect("192.168.40.126:9998").unwrap();
    
        connection.write_all("request".as_bytes()).unwrap();

        println!("{}", read_buffer(&mut connection))
    });
    
    for req in port.incoming() {
        let mut stream = req.unwrap();

        println!("{}", read_buffer(&mut stream));

        stream.write_all("response".as_bytes()).unwrap();
    }
}

fn read_buffer(stream: &mut TcpStream) -> String {
    let mut buf = [0u8; MAX_BUF_SIZE];
    stream.read(&mut buf).unwrap();
    String::from_utf8_lossy(&buf).to_string()
}