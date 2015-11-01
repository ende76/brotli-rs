extern crate brotli;

use std::io;
use std::io::{Read, Write};
use brotli::Decompressor;
use std::fs;
use std::path::Path;

fn visit_dirs(dir: &Path) -> io::Result<()> {
	if try!(fs::metadata(dir)).is_dir() {
		for entry in try!(fs::read_dir(dir)) {
			let entry = try!(entry);
			if try!(fs::metadata(entry.path())).is_dir() {
				;
			} else {
				if entry.file_name().to_str().unwrap().starts_with("id") {
					println!("{:?}:", &entry.path());
					let mut input = Vec::new();
					let res = Decompressor::new(std::fs::File::open(&entry.path()).unwrap()).read_to_end(&mut input);

					println!("output length = {:?}", input.len());
					println!("res = {:?}\n===========\n", res);
				}

			}
		}
	}
	Ok(())
}

fn main() {
	// let mut input = Vec::new();
	// let res = Decompressor::new(std::fs::File::open("data/asyoulik.txt.compressed").unwrap()).read_to_end(&mut input);

	// match res {
	// 	Ok(_) => {
	// 		std::io::stdout().write(&input).unwrap();
	// 	},
	// 	Err(_) => println!("{:?}", res),
	// };

	// for i in 1..7 {
	// 	let _ = visit_dirs(Path::new(&format!("afl-findings/fuzzer0{}/crashes", i)));
	// 	let _ = visit_dirs(Path::new(&format!("afl-findings/fuzzer0{}/hangs", i)));
	// }

	let mut table = vec![(0, 0); 704];

	for insert_and_copy_length in 0..704 {
		let (mut insert_length_code, mut copy_length_code) = match insert_and_copy_length {
			0...63 => (0, 0),
			64...127 => (0, 8),
			128...191 => (0, 0),
			192...255 => (0, 8),
			256...319 => (8, 0),
			320...383 => (8, 8),
			384...447 => (0, 16),
			448...511 => (16, 0),
			512...575 => (8, 16),
			576...639 => (16, 8),
			640...703 => (16, 16),
			_ => unreachable!(),
		};

		insert_length_code += 0x07 & (insert_and_copy_length >> 3);
		copy_length_code += 0x07 & insert_and_copy_length;

		table[insert_and_copy_length] = (insert_length_code, copy_length_code)
	}

	println!("{:?}", table);
}