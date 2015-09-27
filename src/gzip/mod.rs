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
	bfinal: Option<BFinal>,
	btype: Option<BType>,
	buf: Vec<u8>,
	codes: Option<Vec<usize>>,
	crc16: Option<CRC16>,
	extra_flags: Option<ExtraFlags>,
	file_comment: Option<FileComment>,
	file_name: Option<FileName>,
	flags: Option<Flags>,
	identification: Option<Identification>,
	in_buf: VecDeque<u8>,
	in_stream: R,
	mtime: Option<MTime>,
	next_bit: u8,
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
type BFinal = bool;

#[derive(Debug, Clone, PartialEq)]
enum BType {
	NoCompression,
	CompressedWithFixedHuffmanCodes,
	CompressedWithDynamicHuffmanCodes,
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
	ParsingBFinal,
	BFinal(BFinal),
	ParsingBType,
	BType(BType),
	NoCompressionParsingLen,
	ParsingCodeTrees,
	CreatingFixedHuffmanCodes,
	ParsingCode,
}

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
	BFinal(BFinal),
	BType(BType),
}

#[derive(Debug, Clone, PartialEq)]
enum DecompressorError {
	NeedMoreBytes,
	NonZeroReservedFlag,
	NonZeroReservedExtraFlag,
	ReservedBType,
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
			&DecompressorError::ReservedBType => "Reserved BType in deflate block header",
			&DecompressorError::UnknownCompressionMethod => "Unknown compression method in gzip header",
			&DecompressorError::UnknownIdentification => "Unknown identification in gzip header (not a gzip file)",
			&DecompressorError::UnknownOS => "Unknown OS identification in gzip header",
		}
	}
}


impl<R: Read> Decompressor<R> {
	pub fn new(in_stream: R) -> Decompressor<R> {
		Decompressor{
			bfinal: None,
			btype: None,
			buf: Vec::new(),
			codes: None,
			crc16: None,
			extra_flags: None,
			file_comment: None,
			file_name: None,
			flags: None,
			identification: None,
			in_buf: VecDeque::new(),
			in_stream: in_stream,
			next_bit: 0,
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

	fn parse_bfinal(ref mut buf: &mut VecDeque<u8>, mut next_bit: &mut u8) -> result::Result<DecompressorSuccess, DecompressorError> {
		if buf.len() < 1 {

			Err(DecompressorError::NeedMoreBytes)
		} else {
			let b = if *next_bit == 7 {
				buf.pop_front().unwrap()
			} else {
				buf[0]
			};
			let bit_mask = 1u8 << *next_bit;

			*next_bit += 1;

			Ok(DecompressorSuccess::BFinal(b & bit_mask > 0))
		}
	}

	fn parse_btype(ref mut buf: &mut VecDeque<u8>, mut next_bit: &mut u8) -> result::Result<DecompressorSuccess, DecompressorError> {
		if buf.len() < 1 || (buf.len() < 2 && *next_bit == 7) {

			Err(DecompressorError::NeedMoreBytes)
		} else {
			let b0 = if *next_bit == 7 {
				buf.pop_front().unwrap()
			} else {
				buf[0]
			};
			let bit_mask0 = 1u8 << *next_bit;
			*next_bit += 1;

			let b1 = if *next_bit == 7 {
				buf.pop_front().unwrap()
			} else {
				buf[0]
			};
			let bit_mask1 = 1u8 << *next_bit;
			*next_bit += 1;

			match (b1 & bit_mask1, b0 & bit_mask0) {
				(0, 0) => Ok(DecompressorSuccess::BType(BType::NoCompression)),
				(0, _) => Ok(DecompressorSuccess::BType(BType::CompressedWithFixedHuffmanCodes)),
				(_, 0) => Ok(DecompressorSuccess::BType(BType::CompressedWithDynamicHuffmanCodes)),
				(_, _) => Err(DecompressorError::ReservedBType),
			}
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

					self.state = State::ParsingBFinal;
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
					self.state = State::ParsingBFinal;
				},
				State::ParsingBFinal => {
					match Self::parse_bfinal(&mut self.in_buf, &mut self.next_bit) {
						Ok(DecompressorSuccess::BFinal(bfinal)) => self.state = State::BFinal(bfinal),
						Err(DecompressorError::NeedMoreBytes) => self.state = State::Error(DecompressorError::NeedMoreBytes, Box::new(self.state.clone())),
						_ => unreachable!(),
					}
				},
				State::BFinal(bfinal) => {
					self.bfinal = Some(bfinal);
					self.state = State::ParsingBType;
				},
				State::ParsingBType => {
					match Self::parse_btype(&mut self.in_buf, &mut self.next_bit) {
						Ok(DecompressorSuccess::BType(btype)) => self.state = State::BType(btype),
						Err(DecompressorError::ReservedBType) => self.state = State::Error(DecompressorError::ReservedBType, Box::new(self.state.clone())),
						Err(DecompressorError::NeedMoreBytes) => self.state = State::Error(DecompressorError::NeedMoreBytes, Box::new(self.state.clone())),
						_ => unreachable!(),
					}
				},
				State::BType(btype) => {
					self.btype = Some(btype);

					match self.btype {
						Some(BType::NoCompression) => self.state = State::NoCompressionParsingLen,
						Some(BType::CompressedWithDynamicHuffmanCodes) => self.state = State::ParsingCodeTrees,
						Some(BType::CompressedWithFixedHuffmanCodes) => self.state = State::CreatingFixedHuffmanCodes,
						_ => unreachable!(),
					}
				},
				State::NoCompressionParsingLen => {
					unimplemented!();
				},
				State::ParsingCodeTrees => {
					unimplemented!();
				},
				State::CreatingFixedHuffmanCodes => {
					unimplemented!();
				},
				state => {
					assert_eq!(State::Initialized, state);
					panic!(state);
				}
			};
		}
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

			buf[i] = self.buf.pop().unwrap();
		}

		Ok(l)
	}
}
