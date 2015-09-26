use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::fmt::{ Display, Formatter };
use std::io;
use std::io::Read;
use std::result;
use std::string::String;
use std::vec::Vec;

const U16_IDENTIFICATION_GZIP: u16 = 0x1f8b;
const BYTE_COMPRESSION_METHOD_DEFLATE: u8 = 0x08;

const FLAGS_MASK: u8           = 0b11100000;
const FLAG_FTEXT: u8          =  1;
const FLAG_FHCRC: u8          =  2;
const FLAG_FEXTRA: u8         =  4;
const FLAG_FNAME: u8          =  8;
const FLAG_FCOMMENT: u8       = 16;

const EXTRA_FLAGS_MASK: u8     = 0b11111001;
const EXTRA_FLAG_MAXIMUM_COMPRESSION: u8 = 2;
const EXTRA_FLAG_FASTEST_ALGORITHM: u8   = 4;

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
	crc16: Option<CRC16>,
	extra_flags: Option<ExtraFlags>,
	file_comment: Option<FileComment>,
	file_name: Option<FileName>,
	flags: Option<Flags>,
	identification: Option<Identification>,
	in_buf: VecDeque<u8>,
	in_stream: R,
	mtime: Option<MTime>,
	os: Option<OS>,
	state: State,
	xlen: Option<XLen>,
}

#[derive(Debug, Clone, PartialEq)]
enum Identification {
	Gzip
}

#[derive(Debug, Clone, PartialEq)]
enum CompressionMethod {
	Deflate
}

#[derive(Debug, Clone, PartialEq)]
struct Flags{
	ftext: bool,
	fhcrc: bool,
	fextra: bool,
	fname: bool,
	fcomment: bool,
}

type MTime = u32;

#[derive(Debug, Clone, PartialEq)]
struct ExtraFlags {
	maximum_compression: bool,
	fastest_algorithm: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum OS {
	FATFilesystem,
	Amiga,
	VMS,
	Unix,
	VMCMS,
	AtariTOS,
	HPFSFilesystem,
	Macintosh,
	ZSystem,
	CPM,
	TOPS20,
	NTFSFilesystem,
	QDOS,
	AcornRISCOS,
	Unknown,
}

type XLen = u16;
type FileName = String;
type FileComment = String;
type CRC16 = u16;

enum DecompressorSuccess {
	Identification(Identification),
	CompressionMethod(CompressionMethod),
	Flags(Flags),
	MTime(MTime),
	ExtraFlags(ExtraFlags),
	OS(OS),
	XLen(XLen),
	ExtraField,
	FileName(FileName),
	FileComment(FileComment),
	CRC16(CRC16),
}

#[derive(Debug, Clone, PartialEq)]
enum DecompressorError {
	NeedMoreBytes,
	NonZeroReservedFlag,
	NonZeroReservedExtraFlag,
	UnknownCompressionMethod,
	UnknownIdentification,
	UnknownOS,
}

impl Display for DecompressorError {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
		fmt.write_str(self.description())
	}
}

impl Error for DecompressorError {
	fn description(&self) -> &str {
		match self {
			&DecompressorError::NeedMoreBytes => "Need more bytes",
			&DecompressorError::NonZeroReservedFlag => "Non-zero reserved flag in gzip header",
			&DecompressorError::NonZeroReservedExtraFlag => "Non-zero reserved extra-flag in gzip header",
			&DecompressorError::UnknownCompressionMethod => "Unknown compression method in gzip header",
			&DecompressorError::UnknownIdentification => "Unknown identification in gzip header (not a gzip file)",
			&DecompressorError::UnknownOS => "Unknown OS identification in gzip header",
		}
	}
}


#[derive(Debug, Clone, PartialEq)]
enum State {
	Error(DecompressorError, Box<State>),
	Initialized,
	ParsingIdentification,
	Identification(Identification),
	ParsingCompressionMethod,
	CompressionMethod(CompressionMethod),
	ParsingFlags,
	Flags(Flags),
	ParsingMTime,
	MTime(MTime),
	ParsingExtraFlags,
	ExtraFlags(ExtraFlags),
	ParsingOS,
	OS(OS),
	ParsingXLen(bool),
	XLen(XLen),
	ParsingExtraField,
	ExtraField,
	ParsingFileName(bool),
	FileName(FileName),
	ParsingFileComment(bool),
	FileComment(FileComment),
	ParsingCRC16(bool),
	CRC16(CRC16),
	ParsingCompressedBlock,
}

impl<R: Read> Decompressor<R> {
	pub fn new(in_stream: R) -> Decompressor<R> {
		Decompressor{
			buf: VecDeque::new(),
			crc16: None,
			extra_flags: None,
			file_comment: None,
			file_name: None,
			flags: None,
			identification: None,
			in_buf: VecDeque::new(),
			in_stream: in_stream,
			mtime: None,
			os: None,
			state: State::Initialized,
			xlen: None,
		}
	}

	fn read_more_bytes(&mut self) {
		let mut buf = &mut [0u8, 0u8];

		match self.in_stream.read(buf) {
			Ok(l) => {
				for i in 0..l {
					self.in_buf.push_back(buf[i]);
				};
			},
			Err(e) => panic!(e),
		}
	}

	fn parse_identification(ref mut buf: &mut VecDeque<u8>) -> result::Result<DecompressorSuccess, DecompressorError> {
		if buf.len() < 2 {

			Err(DecompressorError::NeedMoreBytes)
		} else {
			let b8 = buf.pop_front().unwrap();
			let b0 = buf.pop_front().unwrap();

			match ((b8 as u16) << 8) | b0 as u16 {
				U16_IDENTIFICATION_GZIP => Ok(DecompressorSuccess::Identification(Identification::Gzip)),
				_ => Err(DecompressorError::UnknownIdentification),
			}
		}
	}

	fn parse_compression_method(ref mut buf: &mut VecDeque<u8>) -> result::Result<DecompressorSuccess, DecompressorError> {
		if buf.len() < 1 {

			Err(DecompressorError::NeedMoreBytes)
		} else {
			let b = buf.pop_front().unwrap();

			match b {
				BYTE_COMPRESSION_METHOD_DEFLATE => Ok(DecompressorSuccess::CompressionMethod(CompressionMethod::Deflate)),
				_ => Err(DecompressorError::UnknownCompressionMethod),
			}
		}
	}

	fn parse_flags(ref mut buf: &mut VecDeque<u8>) -> result::Result<DecompressorSuccess, DecompressorError> {
		if buf.len() < 1 {

			Err(DecompressorError::NeedMoreBytes)
		} else {
			let b = buf.pop_front().unwrap();

			match b {
				flags => {
					if flags & FLAGS_MASK > 0 {

						Err(DecompressorError::NonZeroReservedFlag)
					} else {
						Ok(DecompressorSuccess::Flags(Flags{
							ftext:    flags & FLAG_FTEXT > 0,
							fhcrc:    flags & FLAG_FHCRC > 0,
							fextra:   flags & FLAG_FEXTRA > 0,
							fname:    flags & FLAG_FNAME > 0,
							fcomment: flags & FLAG_FCOMMENT > 0,
						}))
					}
				}
			}
		}
	}

	fn parse_mtime(ref mut buf: &mut VecDeque<u8>) -> result::Result<DecompressorSuccess, DecompressorError> {
		if buf.len() < 4 {

			Err(DecompressorError::NeedMoreBytes)
		} else {
			let b0  = buf.pop_front().unwrap();
			let b8  = buf.pop_front().unwrap();
			let b16 = buf.pop_front().unwrap();
			let b24 = buf.pop_front().unwrap();

			Ok(DecompressorSuccess::MTime(b0 as MTime | ((b8 as MTime) << 8) | ((b16 as MTime) << 16) | ((b24 as MTime) << 24)))
		}
	}


	fn parse_extra_flags(ref mut buf: &mut VecDeque<u8>) -> result::Result<DecompressorSuccess, DecompressorError> {
		if buf.len() < 1 {

			Err(DecompressorError::NeedMoreBytes)
		} else {
			let b = buf.pop_front().unwrap();

			match b {
				flags => {
					if flags & EXTRA_FLAGS_MASK > 0 {

						Err(DecompressorError::NonZeroReservedExtraFlag)
					} else {
						Ok(DecompressorSuccess::ExtraFlags(ExtraFlags{
							maximum_compression: flags & EXTRA_FLAG_MAXIMUM_COMPRESSION > 0,
							fastest_algorithm: flags & EXTRA_FLAG_FASTEST_ALGORITHM > 0,
						}))
					}
				}
			}
		}
	}


	fn parse_os(ref mut buf: &mut VecDeque<u8>) -> result::Result<DecompressorSuccess, DecompressorError> {
		if buf.len() < 1 {

			Err(DecompressorError::NeedMoreBytes)
		} else {
			let b = buf.pop_front().unwrap();

			match b {
				0 => Ok(DecompressorSuccess::OS(OS::FATFilesystem)),
				1 => Ok(DecompressorSuccess::OS(OS::Amiga)),
				2 => Ok(DecompressorSuccess::OS(OS::VMS)),
				3 => Ok(DecompressorSuccess::OS(OS::Unix)),
				4 => Ok(DecompressorSuccess::OS(OS::VMCMS)),
				5 => Ok(DecompressorSuccess::OS(OS::AtariTOS)),
				6 => Ok(DecompressorSuccess::OS(OS::HPFSFilesystem)),
				7 => Ok(DecompressorSuccess::OS(OS::Macintosh)),
				8 => Ok(DecompressorSuccess::OS(OS::ZSystem)),
				9 => Ok(DecompressorSuccess::OS(OS::CPM)),
				10 => Ok(DecompressorSuccess::OS(OS::TOPS20)),
				11 => Ok(DecompressorSuccess::OS(OS::NTFSFilesystem)),
				12 => Ok(DecompressorSuccess::OS(OS::QDOS)),
				13 => Ok(DecompressorSuccess::OS(OS::AcornRISCOS)),
				255 => Ok(DecompressorSuccess::OS(OS::Unknown)),
				_ => Err(DecompressorError::UnknownOS),
			}
		}
	}

	fn parse_xlen(ref mut buf: &mut VecDeque<u8>) -> result::Result<DecompressorSuccess, DecompressorError> {
		if buf.len() < 2 {

			Err(DecompressorError::NeedMoreBytes)
		} else {
			let b0 = buf.pop_front().unwrap();
			let b8 = buf.pop_front().unwrap();

			Ok(DecompressorSuccess::XLen(((b8 as XLen) << 8) | b0 as XLen))
		}
	}

	fn parse_extra_field(ref mut buf: &mut VecDeque<u8>, xlen: XLen) -> result::Result<DecompressorSuccess, DecompressorError> {
		if buf.len() < xlen as usize {

			Err(DecompressorError::NeedMoreBytes)
		} else {

			Ok(DecompressorSuccess::ExtraField)
		}
	}

	fn parse_file_name(ref mut buf: &mut VecDeque<u8>) -> result::Result<DecompressorSuccess, DecompressorError> {
		if !buf.iter().any(|&b| b == 0) {

			Err(DecompressorError::NeedMoreBytes)
		} else {
			let mut file_name: Vec<u8> = Vec::new();
			let mut b;

			loop {
				b = buf.pop_front().unwrap();

				if b == 0 {
					break;
				}

				file_name.push(b);
			}

			Ok(DecompressorSuccess::FileName(String::from_utf8(file_name).unwrap()))
		}
	}

	fn parse_file_comment(ref mut buf: &mut VecDeque<u8>) -> result::Result<DecompressorSuccess, DecompressorError> {
		if !buf.iter().any(|&b| b == 0) {

			Err(DecompressorError::NeedMoreBytes)
		} else {
			let mut file_comment: Vec<u8> = Vec::new();
			let mut b;

			loop {
				b = buf.pop_front().unwrap();

				if b == 0 {
					break;
				}

				file_comment.push(b);
			}

			Ok(DecompressorSuccess::FileComment(String::from_utf8(file_comment).unwrap()))
		}
	}

	fn parse_crc16(ref mut buf: &mut VecDeque<u8>) -> result::Result<DecompressorSuccess, DecompressorError> {
		if buf.len() < 2 {

			Err(DecompressorError::NeedMoreBytes)
		} else {
			let b0 = buf.pop_front().unwrap();
			let b8 = buf.pop_front().unwrap();

			Ok(DecompressorSuccess::CRC16(((b8 as CRC16) << 8) | b0 as CRC16))
		}
	}

	fn decompress(&mut self) -> result::Result<usize, DecompressorError> {
		loop {
			match self.state.clone() {
				State::Error(DecompressorError::NeedMoreBytes, return_state) => {
					self.read_more_bytes();
					self.state = *return_state;
				},
				State::Initialized => {
					self.state = State::ParsingIdentification;
				},
				State::ParsingIdentification => {
					match Self::parse_identification(&mut self.in_buf) {
						Ok(DecompressorSuccess::Identification(id)) => self.state = State::Identification(id),
						Err(DecompressorError::NeedMoreBytes) => self.state = State::Error(DecompressorError::NeedMoreBytes, Box::new(self.state.clone())),
						Err(DecompressorError::UnknownIdentification) => panic!("DecompressorError::UnknownIdentification"),
						_ => unreachable!(),
					}
				},
				State::Identification(id) => {
					assert_eq!(Identification::Gzip, id);
					self.identification = Some(id);
					self.state = State::ParsingCompressionMethod;
				},
				State::ParsingCompressionMethod => {
					match Self::parse_compression_method(&mut self.in_buf) {
						Ok(DecompressorSuccess::CompressionMethod(cm)) => self.state = State::CompressionMethod(cm),
						Err(DecompressorError::NeedMoreBytes) => self.state = State::Error(DecompressorError::NeedMoreBytes, Box::new(self.state.clone())),
						Err(DecompressorError::UnknownCompressionMethod) => panic!("DecompressorError::UnknownCompressionMethod"),
						_ => unreachable!(),
					}
				},
				State::CompressionMethod(cm) => {
					assert_eq!(CompressionMethod::Deflate, cm);
					self.state = State::ParsingFlags;
				},
				State::ParsingFlags => {
					match Self::parse_flags(&mut self.in_buf) {
						Ok(DecompressorSuccess::Flags(flags)) => self.state = State::Flags(flags),
						Err(DecompressorError::NeedMoreBytes) => self.state = State::Error(DecompressorError::NeedMoreBytes, Box::new(self.state.clone())),
						Err(DecompressorError::NonZeroReservedFlag) => panic!("DecompressorError::NonZeroReservedFlag"),
						_ => unreachable!(),
					}
				}
				State::Flags(flags) => {
					self.flags = Some(flags);
					self.state = State::ParsingMTime;
				},
				State::ParsingMTime => {
					match Self::parse_mtime(&mut self.in_buf) {
						Ok(DecompressorSuccess::MTime(mtime)) => self.state = State::MTime(mtime),
						Err(DecompressorError::NeedMoreBytes) => self.state = State::Error(DecompressorError::NeedMoreBytes, Box::new(self.state.clone())),
						_ => unreachable!(),
					}
				},
				State::MTime(mtime) => {
					self.mtime = Some(mtime);
					self.state = State::ParsingExtraFlags;
				},
				State::ParsingExtraFlags => {
					match Self::parse_extra_flags(&mut self.in_buf) {
						Ok(DecompressorSuccess::ExtraFlags(extra_flags)) => self.state = State::ExtraFlags(extra_flags),
						Err(DecompressorError::NeedMoreBytes) => self.state = State::Error(DecompressorError::NeedMoreBytes, Box::new(self.state.clone())),
						Err(DecompressorError::NonZeroReservedExtraFlag) => panic!("DecompressorError::NonZeroReservedExtraFlag"),
						_ => unreachable!(),
					}
				},
				State::ExtraFlags(extra_flags) => {
					self.extra_flags = Some(extra_flags);
					self.state = State::ParsingOS;
				},
				State::ParsingOS => {
					match Self::parse_os(&mut self.in_buf) {
						Ok(DecompressorSuccess::OS(os)) => self.state = State::OS(os),
						Err(DecompressorError::NeedMoreBytes) => self.state = State::Error(DecompressorError::NeedMoreBytes, Box::new(self.state.clone())),
						Err(DecompressorError::UnknownOS) => panic!("DecompressorError::UnknownOS"),
						_ => unreachable!(),
					}
				},
				State::OS(os) => {
					self.os = Some(os);
					self.state = State::ParsingXLen(self.flags.as_ref().unwrap().fextra);
				},
				State::ParsingXLen(true) => {
					match Self::parse_xlen(&mut self.in_buf) {
						Ok(DecompressorSuccess::XLen(xlen)) => self.state = State::XLen(xlen),
						Err(DecompressorError::NeedMoreBytes) => self.state = State::Error(DecompressorError::NeedMoreBytes, Box::new(self.state.clone())),
						_ => unreachable!(),
					}
				},
				State::ParsingXLen(false) => {

					self.state = State::ParsingFileName(self.flags.as_ref().unwrap().fname);
				},
				State::XLen(xlen) => {
					self.xlen = Some(xlen);
					self.state = State::ParsingExtraField;
				},
				State::ParsingExtraField => {
					// We say we're parsing, but really we're just discarding self.xlen # of bytes from in_stream
					match Self::parse_extra_field(&mut self.in_buf, self.xlen.unwrap()) {
						Ok(DecompressorSuccess::ExtraField) => self.state = State::ExtraField,
						Err(DecompressorError::NeedMoreBytes) => self.state = State::Error(DecompressorError::NeedMoreBytes, Box::new(self.state.clone())),
						_ => unreachable!(),
					}
				}
				State::ExtraField => {

					self.state = State::ParsingFileName(self.flags.as_ref().unwrap().fname);
				},
				State::ParsingFileName(true) => {
					match Self::parse_file_name(&mut self.in_buf) {
						Ok(DecompressorSuccess::FileName(file_name)) => self.state = State::FileName(file_name),
						Err(DecompressorError::NeedMoreBytes) => self.state = State::Error(DecompressorError::NeedMoreBytes, Box::new(self.state.clone())),
						_ => unreachable!(),
					}
				},
				State::ParsingFileName(false) => {

					self.state = State::ParsingFileComment(self.flags.as_ref().unwrap().fcomment);
				},
				State::FileName(file_name) => {
					self.file_name = Some(file_name);
					self.state = State::ParsingFileComment(self.flags.as_ref().unwrap().fcomment);
				},
				State::ParsingFileComment(true) => {
					match Self::parse_file_comment(&mut self.in_buf) {
						Ok(DecompressorSuccess::FileComment(file_comment)) => self.state = State::FileComment(file_comment),
						Err(DecompressorError::NeedMoreBytes) => self.state = State::Error(DecompressorError::NeedMoreBytes, Box::new(self.state.clone())),
						_ => unreachable!(),
					}
				},
				State::ParsingFileComment(false) => {

					self.state = State::ParsingCRC16(self.flags.as_ref().unwrap().fhcrc);
				},
				State::FileComment(file_comment) => {
					self.file_comment = Some(file_comment);
					self.state = State::ParsingCRC16(self.flags.as_ref().unwrap().fhcrc);
				},
				State::ParsingCRC16(false) => {

					self.state = State::ParsingCompressedBlock;
				},
				State::ParsingCRC16(true) => {
					match Self::parse_crc16(&mut self.in_buf) {
						Ok(DecompressorSuccess::CRC16(crc16)) => self.state = State::CRC16(crc16),
						Err(DecompressorError::NeedMoreBytes) => self.state = State::Error(DecompressorError::NeedMoreBytes, Box::new(self.state.clone())),
						_ => unreachable!(),
					}
				},
				State::CRC16(crc16) => {
					self.crc16 = Some(crc16);
					self.state = State::ParsingCompressedBlock;
				},
				State::ParsingCompressedBlock => {
					unimplemented!();
				},
				state => {
					assert_eq!(State::Initialized, state);
					panic!("uncaught state");
				}
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