use std::error::Error;
use std::fmt;
use std::fmt::{ Display, Formatter };
use std::io;
use std::io::{ BufReader, ErrorKind, Read };
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
	current_byte: Option<u8>,
}

impl<R: Read> BitReader<R> {
	pub fn new(inner: R) -> BitReader<R> {
		BitReader{
			inner: BufReader::new(inner),
			bit_pos: 0,
			current_byte: None,
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

		match (self.current_byte, self.read_exact(buf)) {
			(Some(byte), Ok(())) => {
				self.current_byte = Some(buf[0]);
				Ok((byte >> self.bit_pos) | (buf[0] << (8 - self.bit_pos)))
			},
			(None, Ok(())) => Ok(buf[0]),
			(_, Err(_)) => Err(BitReaderError::Unspecified),
		}
	}

	pub fn read_u16(&mut self) -> Result<u16, BitReaderError> {
		let mut buf = &mut [0u8; 2];

		match (self.current_byte, self.read_exact(buf)) {
			(Some(byte), Ok(())) => {
				self.current_byte = Some(buf[1]);
				Ok(((byte as u16) >> self.bit_pos) | ((buf[0] as u16) << (8 - self.bit_pos)) | ((buf[1] as u16) << (16 - self.bit_pos)))
			},
			(None, Ok(())) => Ok(((buf[1] as u16) << 8) | (buf[0] as u16)),
			(_, Err(_)) => Err(BitReaderError::Unspecified),
		}
	}

	pub fn read_u32(&mut self) -> Result<u32, BitReaderError> {
		match (self.read_u16(), self.read_u16()) {
			(Ok(my_u16_0), Ok(my_u16_1)) => Ok((my_u16_1 as u32) << 16 | (my_u16_0 as u32)),
			(_, _) => Err(BitReaderError::Unspecified),
		}
	}

	pub fn read_bit(&mut self) -> Result<bool, BitReaderError> {
		match (self.current_byte, self.bit_pos) {
			(Some(byte), bit_pos) => {
				self.bit_pos = (self.bit_pos + 1) % 8;
				if self.bit_pos == 0 {
					self.current_byte = None;
				}
				Ok(byte >> bit_pos & 1 == 1)
			},
			(None, _) => {
				let mut buf = &mut [0u8; 1];
				match self.read_exact(buf) {
					Ok(()) => {
						self.current_byte = Some(buf[0]);
						self.bit_pos = 1;
						Ok(buf[0] & 1 == 1)
					},
					Err(_) => Err(BitReaderError::Unspecified),
				}
			}
		}

	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum BitReaderError {
	Unspecified
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
	#[test]
	fn should_read_one_u8() {
		use super::*;
		use std::io::{ Cursor };

		let expected = 0x1f;
		let mut br = BitReader::new(Cursor::new(vec![expected, 0x8b]));

		match br.read_u8() {
			Ok(byte) => assert_eq!(expected, byte),
			Err(_) => panic!("Should have read one byte"),
		}
	}

	#[test]
	fn should_read_two_u8() {
		use super::*;
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
		use super::*;
		use std::io::{ Cursor };

		let expected = 0x8b1f;
		let mut br = BitReader::new(Cursor::new(vec![(expected & 0x00FF) as u8, (expected >> 8) as u8]));

		match br.read_u16() {
			Ok(my_u16) => assert_eq!(expected, my_u16),
			_ => panic!("Should have read one u16"),
		}
	}

	#[test]
	fn should_read_one_set_bit() {
		use super::*;
		use std::io::{ Cursor };

		let mut br = BitReader::new(Cursor::new(vec![3]));

		match br.read_bit() {
			Ok(my_bit) => assert!(my_bit),
			_ => panic!("Should have read one set bit"),
		}
	}

	#[test]
	fn should_read_some_bits() {
		use super::*;
		use std::io::{ Cursor };

		let mut br = BitReader::new(Cursor::new(vec![134, 1]));

		match (br.read_bit(), br.read_bit(), br.read_bit(), br.read_bit(), br.read_bit(),
		       br.read_bit(), br.read_bit(), br.read_bit(), br.read_bit(), br.read_bit()) {
			(Ok(my_bit_0), Ok(my_bit_1), Ok(my_bit_2), Ok(my_bit_3), Ok(my_bit_4),
			 Ok(my_bit_5), Ok(my_bit_6), Ok(my_bit_7), Ok(my_bit_8), Ok(my_bit_9)) => {
				assert!(!my_bit_0);
				assert!(my_bit_1);
				assert!(my_bit_2);
				assert!(!my_bit_3);
				assert!(!my_bit_4);
				assert!(!my_bit_5);
				assert!(!my_bit_6);
				assert!(my_bit_7);
				assert!(my_bit_8);
				assert!(!my_bit_9);
			},
			_ => panic!("Should have read 10 bits"),
		}
	}

	#[test]
	fn should_read_u8_after_bit() {
		use super::*;
		use std::io::{ Cursor };

		let mut br = BitReader::new(Cursor::new(vec![0b10001101, 0b00010101]));

		match (br.read_bit(), br.read_u8()) {
			(Ok(my_bit), Ok(my_u8)) => {
				assert!(my_bit);
				assert_eq!(0b11000110, my_u8);
			},
			_ => panic!("Should have read one set bit and one u8"),
		}
	}

	#[test]
	fn should_read_u16_after_bit() {
		use super::*;
		use std::io::{ Cursor };

		let mut br = BitReader::new(Cursor::new(vec![0b10001101, 0b00010101, 0b00010101]));

		match (br.read_bit(), br.read_u16()) {
			(Ok(my_bit), Ok(my_u16)) => {
				assert!(my_bit);
				assert_eq!(0b1000101011000110, my_u16);
			},
			_ => panic!("Should have read one set bit and one u16"),
		}
	}

	#[test]
	fn should_read_u32() {
		use super::*;
		use std::io::{ Cursor };

		let mut br = BitReader::new(Cursor::new(vec![0x8e, 0x30, 0x04, 0x56]));

		match br.read_u32() {
			Ok(my_u32) => {
				assert_eq!(0x5604308e, my_u32);
			},
			_ => panic!("Should have read one u32"),
		}
	}
}



