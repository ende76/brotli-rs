use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::fmt::{ Display, Formatter };
use std::io;
use std::io::Read;
use std::result;
use std::string::String;
use std::vec::Vec;

const BYTE_IDENTIFIER1_GZIP: u8 = 0x1f;
const BYTE_IDENTIFIER2_GZIP: u8 = 0x8b;
const BYTE_COMPRESSION_METHOD_DEFLATE: u8 = 0x08;


/// Wraps an input stream and provides methods for decompressing.
///
/// # Examples
///
/// extern crate compression;
///
/// use compression::gzip::Decompressor;
/// use std::fs::File;
///
/// let mut f = try!(File::open("compressed.txt.gz"));
/// let mut gzip = Decompressor::new(f);
pub struct Decompressor<R> {
	buf: VecDeque<u8>,
	comment: Option<String>,
	crc16: Option<u16>,
	extra_flags: Option<ExtraFlags>,
	filename: Option<String>,
	flags: Option<Flags>,
	in_buf: Vec<u8>,
	in_stream: R,
	mtime: Option<u32>,
	os: Option<u8>,
	state: State,
}

#[derive(Debug, Clone, PartialEq)]
enum DecompressorError {
	InvalidOS,
	NeedMoreBytes,
	NonZeroReservedFlag,
	NonZeroReservedExtraFlag,
	UnknownIdentifier,
	UnknownCompressionMethod,
}

impl Display for DecompressorError {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
		fmt.write_str(self.description())
	}
}

impl Error for DecompressorError {
	fn description(&self) -> &str {
		match self {
			&DecompressorError::InvalidOS => "Invalid OS identifier in gzip header",
			&DecompressorError::NeedMoreBytes => "Need more bytes",
			&DecompressorError::NonZeroReservedFlag => "Non-zero reserved flag in gzip header",
			&DecompressorError::NonZeroReservedExtraFlag => "Non-zero reserved extra-flag in gzip header",
			&DecompressorError::UnknownCompressionMethod => "Unknown compression method in gzip header",
			&DecompressorError::UnknownIdentifier => "Unknown identifier in gzip header (not a gzip file)",
		}
	}
}


#[derive(Debug, Clone, PartialEq)]
enum State {
	Error(DecompressorError),
	Initialized,
	Identification(Identification1, Identification2),
	CompressionMethod(CompressionMethod),
	Flags{
		ftext: bool,
		fhcrc: bool,
		fextra: bool,
		fname: bool,
		fcomment: bool,
	},
	MTime(u32),
	ExtraFlags{
		maximum_compression: bool,
		fastest_algorithm: bool,
	},
	OS(u8),
	FlagFExtra,
	XLen(u16),
	ExtraField(usize),
	FlagFName(bool),
	Filename(String),
	FlagFComment(bool),
	Comment(String),
	FlagFHCRC(bool),
	CRC16(u16),
}


#[derive(Debug, Clone, PartialEq)]
enum Identification1 {
	Gzip
}

#[derive(Debug, Clone, PartialEq)]
enum Identification2 {
	Gzip
}

#[derive(Debug, Clone, PartialEq)]
enum CompressionMethod {
	Deflate
}

struct Flags{
	ftext: bool,
	fhcrc: bool,
	fextra: bool,
	fname: bool,
	fcomment: bool,
}

struct ExtraFlags{
	maximum_compression: bool,
	fastest_algorithm: bool,
}

impl<R: Read> Decompressor<R> {
	pub fn new(in_stream: R) -> Decompressor<R> {
		Decompressor{
			buf: VecDeque::new(),
			comment: None,
			crc16: None,
			extra_flags: None,
			filename: None,
			flags: None,
			in_buf: Vec::new(),
			in_stream: in_stream,
			mtime: None,
			os: None,
			state: State::Initialized,
		}
	}

	fn get_state(&self) -> State {
		return self.state.clone();
	}

	fn read_identification(&mut self) {
		let mut buf = &mut [0u8, 0u8];

		match (self.in_stream.read(buf), (buf[0], buf[1])) {
			(Ok(2), (BYTE_IDENTIFIER1_GZIP, BYTE_IDENTIFIER2_GZIP)) => self.state = State::Identification(Identification1::Gzip, Identification2::Gzip),
			(Ok(2), (_, _)) => self.state = State::Error(DecompressorError::UnknownIdentifier),
			(Ok(_), (_, _)) => self.state = State::Error(DecompressorError::NeedMoreBytes),
			(Err(e), (_, _)) => panic!(e),
		};
	}

	fn read_compression_method(&mut self) {
		let mut buf = &mut[0u8];

		match (self.in_stream.read(buf), buf[0]) {
			(Ok(1), BYTE_COMPRESSION_METHOD_DEFLATE) => self.state = State::CompressionMethod(CompressionMethod::Deflate),
			(Ok(1), _) => self.state = State::Error(DecompressorError::UnknownCompressionMethod),
			(Ok(_), _) => self.state = State::Error(DecompressorError::NeedMoreBytes),
			(Err(e), _) => panic!(e),
		};
	}

	fn read_flags(&mut self) {
		let mut buf = &mut[0u8];

		match (self.in_stream.read(buf), buf[0]) {
			(Ok(1), flags) => {
				if flags & 0b11100000 > 0 {

					self.state = State::Error(DecompressorError::NonZeroReservedFlag);
				} else {
					self.state = State::Flags{
						ftext:    flags &  1 > 0,
						fhcrc:    flags &  2 > 0,
						fextra:   flags &  4 > 0,
						fname:    flags &  8 > 0,
						fcomment: flags & 16 > 0,
					};
				}
			},
			(Ok(_), _) => self.state = State::Error(DecompressorError::NeedMoreBytes),
			(Err(e), _) => panic!(e),
		}
	}

	fn read_mtime(&mut self) {
		let mut buf = &mut[0u8, 0u8, 0u8, 0u8];

		match (self.in_stream.read(buf), (buf[0], buf[1], buf[2], buf[3])) {
			(Ok(4), (mtime24, mtime16, mtime8, mtime0)) => {
				self.state = State::MTime(
					mtime0 as u32 | ((mtime8 as u32) << 8) | ((mtime16 as u32) << 16) | ((mtime24 as u32) << 24)
				);
			},
			(Ok(_), _) => self.state = State::Error(DecompressorError::NeedMoreBytes),
			(Err(e), _) => panic!(e),
		}
	}

	fn read_extra_flags(&mut self) {
		let mut buf = &mut[0u8];

		match (self.in_stream.read(buf), buf[0]) {
			(Ok(1), flags) => {
				if flags & 0b11111001 > 0 {

					self.state = State::Error(DecompressorError::NonZeroReservedExtraFlag);
				} else {
					self.state = State::ExtraFlags{
						maximum_compression: flags &  2 > 0,
						fastest_algorithm:   flags &  4 > 0,
					};
				}
			},
			(Ok(_), _) => self.state = State::Error(DecompressorError::NeedMoreBytes),
			(Err(e), _) => panic!(e),
		}
	}

	fn read_os(&mut self) {
		let mut buf = &mut[0u8];

		match (self.in_stream.read(buf), buf[0]) {
			(Ok(1), os) => {
				if os > 13 && os < 255 {
					self.state = State::Error(DecompressorError::InvalidOS,);
				} else {
					self.state = State::OS(os);
				}
			},
			(Ok(_), _) => self.state = State::Error(DecompressorError::NeedMoreBytes),
			(Err(e), _) => panic!(e),
		}
	}

	fn read_xlen(&mut self) {
		let mut buf = &mut[0u8, 0u8];

		match (self.in_stream.read(buf), (buf[0], buf[1])) {
			(Ok(2), (xlen8, xlen0)) => {
				self.state = State::XLen(xlen0 as u16 | ((xlen8 as u16) << 8));
			},
			(Ok(_), _) => self.state = State::Error(DecompressorError::NeedMoreBytes),
			(Err(e), _) => panic!(e),
		}
	}

	fn read_extra_field(&mut self, len: u16) {
		let mut buf = &mut vec![0u8; len as usize];

		match self.in_stream.read(buf) {
			Ok(bytes) if bytes == len as usize => {
				self.state = State::ExtraField(bytes);
			},
			Ok(_) => self.state = State::Error(DecompressorError::NeedMoreBytes),
			Err(e) => panic!(e),
		}
	}

	fn read_filename(&mut self) {
		let mut buf  = &mut vec![0u8];
		let mut v = Vec::new();

		loop {
			match (self.in_stream.read(buf), buf[0]) {
				(Ok(1), 0) => break,
				(Ok(1), byte) => {
					v.push(byte);
				},
				(Ok(_), _) => self.state = State::Error(DecompressorError::NeedMoreBytes),
				(Err(e), _) => panic!(e),
			}
		}

		self.state = State::Filename(String::from_utf8(v).unwrap());
	}

	fn read_comment(&mut self) {
		let mut buf  = &mut vec![0u8];
		let mut v = Vec::new();

		loop {
			match (self.in_stream.read(buf), buf[0]) {
				(Ok(1), 0) => break,
				(Ok(1), byte) => {
					v.push(byte);
				},
				(Ok(_), _) => self.state = State::Error(DecompressorError::NeedMoreBytes),
				(Err(e), _) => panic!(e),
			}
		}

		self.state = State::Comment(String::from_utf8(v).unwrap());
	}

	fn read_crc16(&mut self) {
		let mut buf = &mut[0u8, 0u8];

		match (self.in_stream.read(buf), (buf[0], buf[1])) {
			(Ok(2), (crc168, crc160)) => {
				self.state = State::CRC16(crc160 as u16 | ((crc168 as u16) << 8));
			},
			(Ok(_), _) => self.state = State::Error(DecompressorError::NeedMoreBytes),
			(Err(e), _) => panic!(e),
		}
	}

	fn process_compressed_blocks(&mut self) {

		panic!("I guess I actually have to decompress now…");
	}

	fn decompress(&mut self) -> result::Result<usize, DecompressorError> {
		loop {
			match self.get_state() {
				State::Initialized => self.read_identification(),
				State::Identification(id1, id2) => {
					assert_eq!((Identification1::Gzip, Identification2::Gzip), (id1, id2));
					self.read_compression_method();
				},
				State::CompressionMethod(cm) => {
					assert_eq!(CompressionMethod::Deflate, cm);
					self.read_flags();
				},
				State::Flags{ftext, fhcrc, fextra, fname, fcomment} => {
					self.flags = Some(Flags{
						ftext: ftext,
						fhcrc: fhcrc,
						fextra: fextra,
						fname: fname,
						fcomment: fcomment,
					});
					self.read_mtime();
				},
				State::MTime(mtime) => {
					self.mtime = Some(mtime);
					self.read_extra_flags();
				},
				State::ExtraFlags{ maximum_compression, fastest_algorithm } => {
					self.extra_flags = Some(ExtraFlags{
						maximum_compression: maximum_compression,
						fastest_algorithm: fastest_algorithm,
					});
					self.read_os();
				},
				State::OS(os) => {
					self.os = Some(os);
					self.state = State::FlagFExtra;
				},
				State::FlagFExtra => {
					if self.flags.as_ref().unwrap().fextra {

						self.read_xlen();
					} else {

						self.state = State::FlagFName(self.flags.as_ref().unwrap().fname);
					}
				},
				State::XLen(xlen) => {

					self.read_extra_field(xlen);
				},
				State::ExtraField(xlen) => {

					self.state = State::FlagFName(self.flags.as_ref().unwrap().fname);
				},
				State::FlagFName(true) => {

					self.read_filename();
				},
				State::Filename(filename) => {
					self.filename = Some(filename);
					self.state = State::FlagFComment(self.flags.as_ref().unwrap().fcomment);
				},
				State::FlagFName(false) => {

					self.state = State::FlagFComment(self.flags.as_ref().unwrap().fcomment);
				},
				State::FlagFComment(true) => {

					self.read_comment();
				},
				State::Comment(comment) => {
					self.comment = Some(comment);
					self.state = State::FlagFHCRC(self.flags.as_ref().unwrap().fcomment);
				}
				State::FlagFComment(false) => {

					self.state = State::FlagFHCRC(self.flags.as_ref().unwrap().fhcrc);
				},
				State::FlagFHCRC(true) => {

					self.read_crc16()
				},
				State::CRC16(crc16) => {
					self.crc16 = Some(crc16);
					self.process_compressed_blocks();
				},
				State::FlagFHCRC(false) => {

					self.process_compressed_blocks();
				},
				State::Error(ref e) => {

					panic!(e.clone())
				},
			};
		}

		Ok(0)
	}
}

impl<R: Read> Read for Decompressor<R> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		if self.buf.len() == 0 {
			match self.decompress() {
				Err(e) => panic!(e),
				Ok(_) => {},
			}
		}

		let l = if self.buf.len() < buf.len() {

			self.buf.len()
		} else {

			buf.len()
		};

		for i in 0..l {

			buf[i] = self.buf.pop_front().unwrap();
		}

		Ok(l)
	}
}