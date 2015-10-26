extern crate brotli;

use std::io::Read;
use brotli::Decompressor;

fn main() {
	let mut input = vec![];
	let res = Decompressor::new(std::fs::File::open("id:000012,src:000128,op:havoc,rep:4").unwrap()).read_to_end(&mut input);

	println!("output length = {:?}", input.len());
	println!("res = {:?}", res);
}