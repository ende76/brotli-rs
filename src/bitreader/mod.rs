use std::error::Error;
use std::fmt;
use std::fmt::{ Display, Formatter };
use std::io;
use std::io::{ BufRead, BufReader, ErrorKind, Read };
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
/// let byte: u8 = br.read_u8().unwrap();
#[derive(Debug)]
pub struct BitReader<R: Read> {
	inner: BufReader<R>,
	bit_pos: u8,
}

impl<R: Read> BitReader<R> {
	pub fn new(inner: R) -> BitReader<R> {
		BitReader{
			inner: BufReader::new(inner),
			bit_pos: 0,
		}
	}

	fn read_exact(&mut self, mut buf: &mut [u8]) -> io::Result<()> {
		while !buf.is_empty() {
			match self.inner.read(buf) {
				Ok(0) => break,
				Ok(n) => { let tmp = buf; buf = &mut tmp[n..]; }
				Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
				Err(e) => return Err(e),
			}
		}
		if !buf.is_empty() {
			Err(io::Error::new(ErrorKind::Other, "failed to fill whole buffer"))
		} else {
			Ok(())
		}
	}

	pub fn read_u8(&mut self) -> Result<u8, BitReaderError> {
		let mut buf = &mut [0u8];
		match self.read_exact(buf) {
			Ok(()) => Ok(buf[0]),
			Err(_) => Err(BitReaderError::Other),
		}
	}

	pub fn read_u16(&mut self) -> Result<u16, BitReaderError> {
		let mut buf = &mut [0u8; 2];
		match self.read_exact(buf) {
			Ok(()) => Ok(((buf[1] as u16) << 8) | (buf[0] as u16)),
			Err(_) => Err(BitReaderError::Other),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum BitReaderError {
	Other
}

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
	fn should_read_one_u8() {
		use std::io::{ BufRead, BufReader, Cursor };

		let expected = 0x1f;
		let mut br = BitReader::new(Cursor::new(vec![expected, 0x8b]));

		match br.read_u8() {
			Ok(byte) => assert_eq!(expected, byte),
			Err(_) => panic!("Should have read one byte"),
		}
	}

	#[test]
	fn should_read_two_u8() {
		use std::io::{ Cursor };

		let (expected0, expected1) = (0x1f, 0x8b);
		let mut br = BitReader::new(Cursor::new(vec![expected0, expected1]));

		match (br.read_u8(), br.read_u8()) {
			(Ok(my_u8_0), Ok(my_u8_1)) => assert_eq!((expected0, expected1), (my_u8_0, my_u8_1)),
			_ => panic!("Should have read two bytes"),
		}
	}

	#[test]
	fn should_read_one_u16() {
		use std::io::{ Cursor };

		let expected = 0x8b1f;
		let mut br = BitReader::new(Cursor::new(vec![(expected & 0x00FF) as u8, (expected >> 8) as u8]));

		match br.read_u16() {
			Ok(my_u16) => assert_eq!(expected, my_u16),
			_ => panic!("Should have read one u16"),
		}
	}
}
