#![allow(unused)]
use std::ops;
use rand::Rng;

pub fn and_vector<T>(vec1: Vec<T>, vec2: Vec<T>) -> Vec<T>
where T: ops::BitAnd<Output = T> + Copy {
    vec1.iter()
        .zip(vec2.iter())
        .map(|(x, y)| *x & *y)
        .collect()
}

pub fn rand_byte_vector(size: usize) -> Vec<u8> {
    assert!(size > 0);
    let mut vector: Vec<u8> = Vec::with_capacity(size);
    let mut rng = rand::thread_rng();
    for _ in 0..size {
        vector.push(rng.gen_range(0..=255));
    }
    vector
}

pub fn bytes_from_string<S: ToString>(string: S) -> Vec<u8> {
    let string = string.to_string();
    string.chars().map(|c| c as u8).collect()
}

pub fn bytes_to_string<V: Into<Vec<u8>>>(bytes: V) -> String {
    let bytes: Vec<u8> = bytes.into();
    bytes.iter().map(|b| *b as char).collect()
}

pub fn remove_null(bytes: Vec<u8>) -> Vec<u8> {
    bytes.iter().filter(|b| **b != 0).map(|b| *b).collect()
}

pub fn erase_until_terminator<T>(vec: &mut Vec<T>, term: T) -> Vec<T>
where
    T: Sized + Copy + std::cmp::PartialEq
{
    assert!(vec.len() > 0);
    let mut erased: Vec<T> = Vec::new();
    let len = vec.len()-1;
    for _ in 0..len {
        let obj: T = vec.remove(0);
        if obj == term {break};
        erased.push(obj);
    }
    erased
}

pub fn truncate_until_terminator<T>(vec: &mut Vec<T>, term: T) -> Vec<T>
where
    T: Sized + Copy + std::cmp::PartialEq
{
    assert!(vec.len() > 0);
    let mut erased: Vec<T> = Vec::new();
    let mut i = vec.len()-1;
    loop {
        let obj: T = vec.remove(i);
        if obj == term {break};
        erased.push(obj);
        i -= 1;
    }
    erased
}