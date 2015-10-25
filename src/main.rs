extern crate brotli;

fn main() {
	use std::io::Read;
	use brotli::Decompressor;

    let mut input = vec![];
    let result = Decompressor::new(&b"\xb1".to_vec() as &[u8]).read_to_end(&mut input);

    println!("result = {:?}", result);
}

