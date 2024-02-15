mod vector;
mod kem;
mod tcp;

use vector as vect;
use tcp::StreamReader;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::sync::{Arc, Mutex};

const MAX_CONTENT_LENGTH: usize = 2048;
const KEY_SIZE: usize = 16;

fn main() {
    let port = TcpListener::bind("0.0.0.0:9998").unwrap();
    
    thread::spawn(|| {
        tcp::check_availability("192.168.40.126:9998", || {});
        let mut stream = TcpStream::connect("192.168.40.126:9998").unwrap();

        let public_key = vect::rand_byte_vector(KEY_SIZE);
        stream.write_all(&["PUBLICKEY\0".as_bytes(), &public_key, &[255u8]].concat()).unwrap();
        
        let mut recv_key = [0u8; MAX_CONTENT_LENGTH];
        stream.read(&mut recv_key).unwrap();
        let mut recv_key = recv_key.to_vec();
        vect::truncate_until_terminator(&mut recv_key, 255u8);
        drop(stream);

        let mut stream = TcpStream::connect("192.168.40.126:9998").unwrap();

        let base_key = vect::rand_byte_vector(KEY_SIZE);
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
        let bytes = kem::encrypt(bytes, private_key);

        stream.write_all(&["MESSAGE\0".as_bytes(), &bytes, &[255u8]].concat()).unwrap();
        let mut empty = [0u8; 1];
        stream.read(&mut empty).unwrap();
        drop(stream);
    });
    
    let base_key = Arc::new(vect::rand_byte_vector(KEY_SIZE));
    let private_key: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

    for req in port.incoming() {
        let base_key = Arc::clone(&base_key);
        let private_key = Arc::clone(&private_key);

        thread::spawn(move || {
            let mut stream = req.unwrap();

            let mut data = [0u8; MAX_CONTENT_LENGTH];
            stream.read(&mut data).unwrap();
            let mut data = data.to_vec();
            match data[0] {
                22u8 => stream.write_all(&[6u8]).unwrap(),
                _ => {
                    vect::truncate_until_terminator(&mut data, 255u8);
                    let protocol = vect::erase_until_terminator(&mut data, 0u8);
                    let protocol = vect::bytes_to_string(protocol);
                    let protocol = protocol.as_str();
            
                    match protocol {
                        "PUBLICKEY" => {
                            let combined_key = vect::and_vector(base_key.to_vec(), data);
                            stream.write_all(&[combined_key.as_slice(), &[255u8]].concat()).unwrap();
                        },
                        "COMBINEKEY" => {
                            let mut mutex = private_key.lock().unwrap();
                            *mutex = vect::and_vector(base_key.to_vec(), data);
                            drop(mutex);

                            stream.write_all(&[0u8]).unwrap();
                            println!("{:?}", &private_key);
                        },
                        "MESSAGE" => {
                            let key = {
                                let mutex = private_key.lock().unwrap();
                                mutex.clone()
                            };

                            let message = kem::decrypt(data, key);
                            let message = vect::bytes_to_string(message);
                            println!("Got: {message}");
                            stream.write_all(&[0u8]).unwrap()
                        },
                        _ => stream.write_all(&[0u8]).unwrap()
                    }        
                }
            }
        });
    }
}