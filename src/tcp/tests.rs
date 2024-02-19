#![cfg(test)]

use super::*;
use std::{
    net::TcpListener,
    io::Write,
    thread,
    sync::{Arc, Mutex}
};

const KEY_SIZE: usize = 16;
const TEST_IP: &'static str = "192.168.40.126:9998"; // Your ip here

#[test]
fn sending_receiving() {
    let port = TcpListener::bind("0.0.0.0:9998").unwrap();
    
    let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
    let sent = Arc::new(Mutex::new(String::new()));
    let received = Arc::new(Mutex::new(String::new()));

    let to_send = Arc::clone(&sent);
    let sending = thread::spawn(move || {
        check_availability(TEST_IP).unwrap();

        let public_key = vect::rand_byte_vector(KEY_SIZE);
        let recv_key = send_public_key(TEST_IP, public_key.clone()).unwrap();

        let base_key = vect::rand_byte_vector(KEY_SIZE);
        let private_key = vect::and_vector(base_key.clone(), recv_key);
        println!("{:?}", private_key.clone());
        
        let combined_key = vect::and_vector(base_key, public_key);
        send_mixed_key(TEST_IP, combined_key).unwrap();

        let message = "you will be forever alone";

        let mut mutex = to_send.lock().unwrap();
        *mutex = message.to_string();
        drop(mutex);

        encrypted_send(TEST_IP, message, private_key).unwrap();
    });
    threads.push(sending);
    
    let base_key = Arc::new(vect::rand_byte_vector(KEY_SIZE));
    let private_key: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

    let mut test_count: u8 = 0;

    for req in port.incoming() {
        let base_key = Arc::clone(&base_key);
        let private_key = Arc::clone(&private_key);

        let to_recv = Arc::clone(&received);
        let handle = thread::spawn(move || {
            let mut stream = req.unwrap();

            stream.parse_incoming(|stream, protocol, data| match protocol {
                Protocol::PublicKey => {
                    let combined_key = vect::and_vector(base_key.to_vec(), data);
                    stream.write_all(&[combined_key.as_slice(), &[255u8]].concat()).unwrap();
                },
                Protocol::CombineKey => {
                    let mut mutex = private_key.lock().unwrap();
                    *mutex = vect::and_vector(base_key.to_vec(), data);
                    drop(mutex);

                    stream.write_all(&[0u8]).unwrap();
                    println!("{:?}", &private_key);
                },
                Protocol::Message => {
                    let key = {
                        let mutex = private_key.lock().unwrap();
                        mutex.clone()
                    };

                    let message = kem::decrypt(data, key);
                    let message = vect::bytes_to_string(message);
                    
                    let mut mutex = to_recv.lock().unwrap();
                    *mutex = message.to_string();
                    drop(mutex);

                    stream.write_all(&[0u8]).unwrap()
                },
                _ => stream.write_all(&[0u8]).unwrap()
            });
        });
        threads.push(handle);
        test_count += 1;
        if test_count >= 4 {break}
    }

    for handle in threads {
        handle.join().unwrap();
    }

    let (sent, received) = {
        let mutex = sent.lock().unwrap();
        let sent = mutex.clone();
        
        let mutex = received.lock().unwrap();
        let received = mutex.clone();

        (sent, received)
    };
    
    assert_eq!(sent, received);
}