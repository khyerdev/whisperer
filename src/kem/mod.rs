//! Khyernet Encryption Module
#![allow(unused)]
mod matrix;
use matrix::Matrix;
use crate::square_matrix;

pub fn encrypt<T, K>(bytes: T, key: K) -> Vec<u8>
where
    T: Into<Vec<u8>>,
    K: Into<Vec<u8>>
{
    let bytes: Vec<u8> = bytes.into();
    let key: Vec<u8> = key.into();

    let blocklength = (bytes.len() as f32).sqrt().ceil() as usize;
    let mut block: Matrix<u8> = square_matrix!(u8, blocklength);
    let mut encrypted: Vec<u8> = Vec::new();

    bytes.iter().enumerate().for_each(|(i, v)| {
        block[i / blocklength].push(*v);
    });

    (0usize..blocklength).for_each(|col| {
        let mut row = blocklength;
        loop {
            if row == 0 {break}
            row -= 1;
            encrypted.push(*block[row].get(col).unwrap_or(&0));
        }
    });

    let keylen = key.len();
    encrypted.iter().enumerate().map(|(i, v)| v ^ key[i % keylen] ).collect()
}

pub fn decrypt<T, K>(bytes: T, key: K) -> Vec<u8>
where
    T: Into<Vec<u8>>,
    K: Into<Vec<u8>>
{
    let bytes: Vec<u8> = bytes.into();
    let key: Vec<u8> = key.into();

    let blocklength = (bytes.len() as f32).sqrt().ceil() as usize;
    let mut block: Matrix<u8> = square_matrix!(u8, blocklength);
    let mut decrypted: Vec<u8> = Vec::new();

    let keylen = key.len();
    let bytes: Vec<u8> = bytes.iter().enumerate().map(|(i, v)| v ^ key[i % keylen] ).collect();

    let mut enc_iter = bytes.iter();
    (0usize..blocklength).for_each(|_| {
        let mut row = blocklength;
        loop {
            if row == 0 {break}
            row -= 1;
            block[row].push(*enc_iter.next().unwrap());
        }
    });

    (0usize..blocklength).for_each(|row| {
        (0usize..blocklength).for_each(|col| {
            decrypted.push(block[row][col]);
        });
    });

    decrypted
}