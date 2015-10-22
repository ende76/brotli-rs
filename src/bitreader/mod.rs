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
	global_bit_pos: usize,
}

impl<R: Read> BitReader<R> {
	/// Creates a BitReader from a Read.
	pub fn new(inner: R) -> BitReader<R> {
		BitReader{
			inner: BufReader::new(inner),
			bit_pos: 0,
			current_byte: None,
			global_bit_pos: 0,
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
			Err(io::Error::new(ErrorKind::Other, "EOF"))
		} else {
			Ok(())
		}
	}

	/// Reads a u8 from the stream, reading exactly one byte.
	/// Returns a BitReaderError if the stream ends prematurely.
	pub fn read_u8(&mut self) -> Result<u8, BitReaderError> {
		let mut buf = &mut [0u8];

		// println!("bit pos = {:?}", self.global_bit_pos);

		match (self.bit_pos, self.current_byte, self.read_exact(buf)) {
			(_, Some(byte), Ok(())) => {
				self.current_byte = Some(buf[0]);
				self.global_bit_pos += 8;
				Ok((byte >> self.bit_pos) | (buf[0] << (8 - self.bit_pos)))
			},
			(_, None, Ok(())) => {
				self.global_bit_pos += 8;
				Ok(buf[0])
			},
			(0, Some(byte), Err(_)) => {
				self.current_byte = None;
				self.global_bit_pos += 8;
				Ok(byte)
			},
			(_, _, Err(e)) => if e.description() == "EOF" {
				Err(BitReaderError::EOF)
			} else {
				Err(BitReaderError::Unspecified)
			},
		}
	}

	/// Reads a u8 from 4 bits.
	/// Returns a BitReaderError if the stream ends prematurely.
	pub fn read_u8_from_nibble(&mut self) -> Result<u8, BitReaderError> {
		let mut buf = &mut [0u8];

		// println!("bit pos = {:?}", self.global_bit_pos);

		match (self.bit_pos, self.current_byte) {
			(0, None) => match self.read_exact(buf) {
				Ok(()) => {
					self.global_bit_pos += 4;
					self.bit_pos = 4;
					self.current_byte = Some(buf[0]);
					Ok(buf[0] & 0x0f)
				},
				Err(e) => if e.description() == "EOF" {
					Err(BitReaderError::EOF)
				} else {
					Err(BitReaderError::Unspecified)
				},
			},
			(0...3, Some(byte)) => {
				self.global_bit_pos += 4;
				self.bit_pos += 4;
				Ok((byte >> (self.bit_pos - 4)) & 0x0f)
			},
			(4, Some(byte)) => {
				self.global_bit_pos += 4;
				self.bit_pos = 0;
				self.current_byte = None;
				Ok((byte >> 4) & 0x0f)
			},
			(bit_pos @ 5...7, Some(byte)) => {
				match self.read_exact(buf) {
					Ok(()) => {
						self.global_bit_pos += 4;
						self.bit_pos = self.bit_pos - 4;
						self.current_byte = Some(buf[0]);
						Ok(((byte >> (bit_pos)) | (buf[0] << (8 - bit_pos))) & 0x0f)
					},
					Err(e) => if e.description() == "EOF" {
						Err(BitReaderError::EOF)
					} else {
						Err(BitReaderError::Unspecified)
					},
				}
			},
			_ => unreachable!(),
		}
	}

	/// Reads a u16 from two bytes.
	/// Only supports little endian, i.e. the least significant byte comes first in the stream.
	/// Returns a BitReaderError if the stream ends prematurely.
	pub fn read_u16(&mut self) -> Result<u16, BitReaderError> {
		let mut buf = &mut [0u8; 2];

		// println!("bit pos = {:?}", self.global_bit_pos);

		match (self.current_byte, self.read_exact(buf)) {
			(Some(byte), Ok(())) => {
				self.current_byte = Some(buf[1]);
				self.global_bit_pos += 16;
				Ok(((byte as u16) >> self.bit_pos) | ((buf[0] as u16) << (8 - self.bit_pos)) | ((buf[1] as u16) << (16 - self.bit_pos)))
			},
			(None, Ok(())) => {
				self.global_bit_pos += 16;
				Ok(((buf[1] as u16) << 8) | (buf[0] as u16))
			},
			(_, Err(_)) => Err(BitReaderError::Unspecified),
		}
	}

	/// Reads a u32 from four bytes.
	/// Only supports little endian, i.e. the least significant byte comes first in the stream.
	/// Returns a BitReaderError if the stream ends prematurely.
	pub fn read_u32(&mut self) -> Result<u32, BitReaderError> {
		match (self.read_u16(), self.read_u16()) {
			(Ok(my_u16_0), Ok(my_u16_1)) => Ok((my_u16_1 as u32) << 16 | (my_u16_0 as u32)),
			(_, _) => Err(BitReaderError::Unspecified),
		}
	}

	/// Reads a u32 from n bits.
	/// Only supports little endian, i.e. the least significant bit comes first in the stream.
	/// Returns a BitReaderError if the stream ends prematurely, or if n exceeds the number of possible bits.
	pub fn read_u32_from_n_bits(&mut self, n: usize) -> Result<u32, BitReaderError> {
		if n > 32 {
			return Err(BitReaderError::TooManyBitsForU32);
		}

		let mut my_u32 = 0;

		// println!("bit pos = {:?}", self.global_bit_pos);

		for i in 0..n {
			match self.read_bit() {
				Ok(true) => my_u32 = my_u32 | (1 << i),
				Ok(false) => {},
				Err(_) => return Err(BitReaderError::Unspecified),
			}
		}

		Ok(my_u32)
	}

	/// Reads a u32 from n nibbles (4 bits).
	/// Only supports little endian, i.e. the least significant nibble comes first in the stream.
	/// Returns a BitReaderError if the stream ends prematurely, or if n exceeds the number of possible nibbles.
	pub fn read_u32_from_n_nibbles(&mut self, n: usize) -> Result<u32, BitReaderError> {
		let mut my_u32 = 0;

		for i in 0..n {
			match self.read_u8_from_nibble() {
				Ok(my_u8) => my_u32 = my_u32 | ((my_u8 as u32) << (4 * i)),
				Err(e) => return Err(e),
			}
		}

		Ok(my_u32)
	}

	/// Reads one bit from the stream, returns a bool result.
	/// Returns a BitReaderError if the stream ends prematurely.
	pub fn read_bit(&mut self) -> Result<bool, BitReaderError> {
		// println!("bit pos = {:?}", self.global_bit_pos);

		match (self.current_byte, self.bit_pos) {
			(Some(byte), bit_pos) => {
				self.bit_pos = (self.bit_pos + 1) % 8;
				self.global_bit_pos += 1;
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
						self.global_bit_pos += 1;
						Ok(buf[0] & 1 == 1)
					},
					Err(_) => Err(BitReaderError::Unspecified),
				}
			}
		}
	}

	/// Reads n bits from the stream, returns a Vec<bool> result.
	/// Returns a BitReaderError if the stream ends prematurely.
	pub fn read_n_bits(&mut self, n: usize) -> Result<Vec<bool>, BitReaderError> {
		let mut v = Vec::with_capacity(n);

		for _ in 0..n {
			match self.read_bit() {
				Ok(bit) => v.push(bit),
				Err(_) => return Err(BitReaderError::Unspecified),
			}
		}

		Ok(v)
	}

	/// Reads a u8 from n bits from the stream.
	/// Returns a BitReaderError if the stream ends prematurely, or if n exceeds the
	/// possible number of bits.
	pub fn read_u8_from_n_bits(&mut self, n: usize) -> Result<u8, BitReaderError> {
		if n > 8 {
			return Err(BitReaderError::TooManyBitsForU8);
		}

		let mut my_u8 = 0;

		for i in 0..n {
			match self.read_bit() {
				Ok(true) => my_u8 = my_u8 | (1 << i),
				Ok(false) => {},
				Err(_) => return Err(BitReaderError::Unspecified),
			}
		}

		Ok(my_u8)
	}

	/// Reads a u8 from n bits from the stream.
	/// As opposed to read_u8_from_n_bits(), this method interprets the bits in reverse order,
	/// i.e. the most significant bit comes first in the stream.
	/// Returns a BitReaderError if the stream ends prematurely, or if n exceeds the possible
	/// number of bits.
	pub fn read_u8_from_n_bits_reverse(&mut self, n: usize) -> Result<u8, BitReaderError> {
		if n > 8 {
			return Err(BitReaderError::TooManyBitsForU8);
		}

		let mut my_u8 = 0;

		for _ in 0..n {
			match self.read_bit() {
				Ok(true) => my_u8 = (my_u8 << 1) | 1,
				Ok(false) => my_u8 = my_u8 << 1,
				Err(_) => return Err(BitReaderError::Unspecified),
			}
		}

		Ok(my_u8)
	}

	/// Reads u8 from bits up to the next byte boundary.
	/// Returns a BitReaderError if the stream ends prematurely.
	pub fn read_u8_from_byte_tail(&mut self) -> Result<u8, BitReaderError> {
		let bit_pos = self.bit_pos.clone();

		if bit_pos == 0 {

			Ok(0)
		} else {

			self.read_u8_from_n_bits(8 - bit_pos as usize)
		}
	}

	/// Reads a u16 from n bits from the stream.
	/// Returns a BitReaderError if the stream ends prematurely, or if n exceeds the
	/// possible number of bits.
	pub fn read_u16_from_n_bits(&mut self, n: usize) -> Result<u16, BitReaderError> {
		if n > 16 {
			return Err(BitReaderError::TooManyBitsForU16);
		}

		let mut my_u16 = 0;

		for i in 0..n {
			match self.read_bit() {
				Ok(true) => my_u16 = my_u16 | (1 << i),
				Ok(false) => {},
				Err(_) => return Err(BitReaderError::Unspecified),
			}
		}

		Ok(my_u16)
	}

	/// Reads a vector of u8 until a value of 0 is encountered.
	/// Returns a Vec<u8> result, where the terminating 0 value is not included.
	/// Returns a BitReaderError if the stream ends prematurely.
	pub fn read_zero_terminated_string(&mut self) -> Result<Vec<u8>, BitReaderError> {
		let mut my_string = Vec::with_capacity(16);

		loop {
			match self.read_u8() {
				Ok(0) => break,
				Ok(byte) => my_string.push(byte),
				Err(_) => return Err(BitReaderError::Unspecified),
			}
		}

		Ok(my_string)
	}

	/// Reads a vector of u8 of a given length.
	/// Returns a BitReaderError if the stream ends prematurely.
	pub fn read_fixed_length_string(&mut self, len: usize) -> Result<Vec<u8>, BitReaderError> {
		let mut my_string = Vec::with_capacity(len);

		for _ in 0..len {
			match self.read_u8() {
				Ok(byte) => my_string.push(byte),
				Err(_) => return Err(BitReaderError::Unspecified),
			}
		}

		Ok(my_string)
	}
}

/// Error types that can be returned by the decompressor
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum BitReaderError {
	/// Unspecified error
	Unspecified,
	/// Tried to read u8 from more than 8 bits.
	TooManyBitsForU8,
	/// Tried to read u16 from more than 16 bits.
	TooManyBitsForU16,
	/// Tried to read u32 from more than 32 bits.
	TooManyBitsForU32,
	/// Unexpected end of file
	EOF,
}

impl Display for BitReaderError {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
		fmt.write_str(self.description())
	}
}

impl Error for BitReaderError {
	fn description(&self) -> &str {
		match *self {
			BitReaderError::TooManyBitsForU8 => "Tried reading u8 from more than 8 bits",
			BitReaderError::TooManyBitsForU16 => "Tried reading u16 from more than 16 bits",
			BitReaderError::TooManyBitsForU32 => "Tried reading u32 from more than 32 bits",
			BitReaderError::EOF => "EOF",
			_ => "Generic error",
		}
	}
}

mod tests {
	#[test]
	fn should_read_one_u8() {
		use super::*;
		use std::io::Cursor;

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
		use std::io::Cursor;

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
		use std::io::Cursor;

		let expected = 0x8b1f;
		let mut br = BitReader::new(Cursor::new(vec![(expected & 0x00ff) as u8, (expected >> 8) as u8]));

		match br.read_u16() {
			Ok(my_u16) => assert_eq!(expected, my_u16),
			_ => panic!("Should have read one u16"),
		}
	}

	#[test]
	fn should_read_one_set_bit() {
		use super::*;
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![3]));

		match br.read_bit() {
			Ok(my_bit) => assert!(my_bit),
			_ => panic!("Should have read one set bit"),
		}
	}

	#[test]
	fn should_read_some_bits() {
		use super::*;
		use std::io::Cursor;

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
		use std::io::Cursor;

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
		use std::io::Cursor;

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
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![0x8e, 0x30, 0x04, 0x56]));

		match br.read_u32() {
			Ok(my_u32) => {
				assert_eq!(0x5604308e, my_u32);
			},
			_ => panic!("Should have read one u32"),
		}
	}

	#[test]
	fn should_read_zero_terminated_string() {
		use super::*;
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![
			0x78, 0x78, 0x78, 0x78, 0x78,
			0x79, 0x79, 0x79, 0x79, 0x79,
			0x2e, 0x74, 0x78, 0x74, 0x00,
			0x78, 0x78, 0x78, 0x78, 0x78,
			0x79, 0x79, 0x79, 0x79, 0x79,
			0x2e, 0x74, 0x78, 0x74, 0x00,
		]));

		match br.read_zero_terminated_string() {
			Ok(my_vec) => assert_eq!("xxxxxyyyyy.txt", &(String::from_utf8(my_vec).unwrap())),
			_ => panic!("Should have read zero-terminated string"),
		}
	}

	#[test]
	fn should_read_fixed_length_string() {
		use super::*;
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![
			0x78, 0x78, 0x78, 0x78, 0x78,
			0x79, 0x79, 0x79, 0x79, 0x79,
			0x2e, 0x74, 0x78, 0x74, 0x00,
			0x78, 0x78, 0x78, 0x78, 0x78,
			0x79, 0x79, 0x79, 0x79, 0x79,
			0x2e, 0x74, 0x78, 0x74, 0x00,
		]));

		match br.read_fixed_length_string(14) {
			Ok(my_vec) => assert_eq!("xxxxxyyyyy.txt", &(String::from_utf8(my_vec).unwrap())),
			_ => panic!("Should have read fixed-length string"),
		}
	}

	#[test]
	fn should_read_eight_bits() {
		use super::*;
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![0x08]));

		match br.read_n_bits(8) {
			Ok(bits) => assert_eq!(vec![false, false, false, true, false, false, false, false], bits),
			_ => panic!("Should have read 8 bits"),
		}
	}

	#[test]
	fn should_read_11_bits() {
		use super::*;
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![0b11001000, 0b11111110]));

		match br.read_n_bits(11) {
			Ok(bits) => assert_eq!(vec![false, false, false, true, false, false, true, true, false, true, true], bits),
			_ => panic!("Should have read 11 bits"),
		}
	}

	#[test]
	fn should_read_29u8_from_5_bits() {
		use super::*;
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![157]));

		match br.read_u8_from_n_bits(5) {
			Ok(my_u8) => assert_eq!(29, my_u8),
			_ => panic!("Should have read 29u8"),
		}
	}

	#[test]
	fn should_read_9u8_from_5_bits_reverse() {
		use super::*;
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![0b00110010]));

		match br.read_u8_from_n_bits_reverse(5) {
			Ok(my_u8) => assert_eq!(9, my_u8),
			_ => panic!("Should have read 9u8"),
		}
	}


	#[test]
	fn should_read_3784u16_from_11_bits() {
		use super::*;
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![0b11001000, 0b11111110]));

		match br.read_u16_from_n_bits(11) {
			Ok(my_u16) => assert_eq!(1736, my_u16),
			_ => panic!("Should have read 3784u16"),
		}
	}

	#[test]
	fn should_read_19u8_from_byte_tail() {
		use super::*;
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![0b10011101]));
		let _ = br.read_bit();
		let _ = br.read_bit();
		let _ = br.read_bit();

		match br.read_u8_from_byte_tail() {
			Ok(my_u8) => assert_eq!(19, my_u8),
			_ => panic!("Should have read 19u8"),
		}
	}

	#[test]
	fn should_read_10u8_from_nibble() {
		use super::*;
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![0b11010101]));
		let _ = br.read_bit();
		let _ = br.read_bit();
		let _ = br.read_bit();

		match br.read_u8_from_nibble() {
			Ok(my_u8) => assert_eq!(10, my_u8),
			_ => panic!("Should have read 10u8"),
		}
	}

	#[test]
	fn should_read_10u8_nibble_twice() {
		use super::*;
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![0b10101010]));

		match br.read_u8_from_nibble() {
			Ok(my_u8) => assert_eq!(10, my_u8),
			_ => panic!("Should have read 10u8"),
		}

		match br.read_u8_from_nibble() {
			Ok(my_u8) => assert_eq!(10, my_u8),
			_ => panic!("Should have read 10u8"),
		}
	}

	#[test]
	fn should_read_7u8_nibble_four_times() {
		use super::*;
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![0b11101111, 0b11101110, 0b11101110]));
		let _ = br.read_bit();

		match br.read_u8_from_nibble() {
			Ok(my_u8) => assert_eq!(7, my_u8),
			_ => panic!("Should have read 7u8"),
		}

		match br.read_u8_from_nibble() {
			Ok(my_u8) => assert_eq!(7, my_u8),
			_ => panic!("Should have read 7u8"),
		}

		match br.read_u8_from_nibble() {
			Ok(my_u8) => assert_eq!(7, my_u8),
			_ => panic!("Should have read 7u8"),
		}

		match br.read_u8_from_nibble() {
			Ok(my_u8) => assert_eq!(7, my_u8),
			_ => panic!("Should have read 7u8"),
		}
	}

	#[test]
	fn should_read_524527u32_from_5_nibbles() {
		use super::*;
		use std::io::Cursor;

		let mut br = BitReader::new(Cursor::new(vec![0b1110_1111, 0, 0b1110_1000]));

		match br.read_u32_from_n_nibbles(5) {
			Ok(my_u32) => assert_eq!(524527, my_u32),
			_ => panic!("Should have read 524527u32"),
		}
	}
}
