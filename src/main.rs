extern crate brotli;

use std::io::Read;
use brotli::Decompressor;

fn main() {
    let mut input = vec![];
    let _ = Decompressor::new(&b"\x30\x30\x40\x00\x00\x00\x00\x00".to_vec() as &[u8]).read_to_end(&mut input);
}