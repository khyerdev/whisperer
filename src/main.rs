mod vector;
mod sem;

use vector as vect;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, BufReader, BufWriter};
use std::thread;

fn main() {
    let port = TcpListener::bind("0.0.0.0:9998").unwrap();
    
    thread::spawn(|| {
        thread::sleep(std::time::Duration::from_secs(2));
        let mut connection = TcpStream::connect("192.168.40.126:9998").unwrap();
    
        connection.write_all("test".as_bytes()).unwrap();
        println!("sent");

        let mut buf = String::new();
        connection.read_to_string(&mut buf).unwrap();
        println!("{buf}")
    });
    
    for req in port.incoming() {
        let mut stream = req.unwrap();
        stream.write_all("response".as_bytes()).unwrap();
        println!("responded");
    }
}