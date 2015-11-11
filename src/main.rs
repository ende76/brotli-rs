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
	// let res = Decompressor::new(std::fs::File::open("data/alice29.txt.compressed").unwrap()).read_to_end(&mut input);

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


	{
		let dir = Path::new("data");

		if fs::metadata(dir).unwrap().is_dir() {
			for entry in fs::read_dir(dir).unwrap() {
				let entry = entry.unwrap();
				if fs::metadata(entry.path()).unwrap().is_dir() {
					;
				} else {
					if entry.file_name().to_str().unwrap().ends_with("compressed") {
						println!("{:?}:", &entry.path());
						let mut input = Vec::new();
						let res = Decompressor::new(std::fs::File::open(&entry.path()).unwrap()).read_to_end(&mut input);

						println!("output length = {:?}", input.len());
						println!("res = {:?}\n===========\n", res);
					}

				}
			}
		}
	}
}
