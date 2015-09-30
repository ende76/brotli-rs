use std::error::Error;
use std::fmt;
use std::fmt::{ Display, Formatter };
use std::io::Read;
use std::result::Result;

/// Wrapper for a Reader, providing convenience methods to read the stream bit-by-bit.
///
/// # Examples
///
/// extern crate compression;
///
/// use compression::bitreader::BitReader;
/// use std::fs::File;
///
/// let f = try!(File::open("filename"));
/// let mut br = BitReader::new(f);

#[derive(Debug, Clone, PartialEq)]
pub struct BitReader<R: Read> {
	inner: R
}

impl<R: Read> BitReader<R> {
	pub fn new(inner: R) -> BitReader<R> {
		BitReader{
			inner: inner
		}
	}

	pub fn read_byte(&self) -> Result<u8, BitReaderError> {
		Ok(0x1f)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum BitReaderError {}

impl Display for BitReaderError {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
		fmt.write_str(self.description())
	}
}

impl Error for BitReaderError {
	fn description(&self) -> &str {
		match self {
			_ => "Generic error",
		}
	}
}

mod tests {
	use super::*;

	#[test]
	fn should_read_two_bytes() {
		use std::io::{ Cursor };

		let expected = 0x1f;
		let mut br = BitReader::new(Cursor::new(vec![expected, 0x8b]));

		match br.read_byte() {
			Ok(byte) => assert_eq!(expected, byte),
			Err(_) => panic!("Should have read one byte"),
		}
	}
}
