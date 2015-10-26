extern crate brotli;

use std::io::Read;
use brotli::Decompressor;

fn main() {
    let mut input = vec![];
    let _ = Decompressor::new(&b"\x1b\x30\x30\x30\x24\x30\xe2\xd9\x30\x30".to_vec() as &[u8]).read_to_end(&mut input);
}