use ::bitreader::BitReader;
use ::deflate;

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::fmt::{ Display, Formatter };
use std::io;
use std::io::Read;
use std::result;
use std::string::String;


const U8_IDENTIFICATION1_GZIP: u8 = 0x1f;
const U8_IDENTIFICATION2_GZIP: u8 = 0x8b;
const U8_COMPRESSION_METHOD_DEFLATE: u8 = 0x08;

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
pub struct Decompressor<R: Read> {
	in_stream: BitReader<R>,

	header: Header,

	buf: VecDeque<u8>,

	state: State,

	// note: this could probably be a more generic Decompressor trait
	//       does not have much impact in this concrete case, because
	//       gzip only seems to use deflate for the actual compression
	sub_decompressor: Option<deflate::Decompressor>,
}

#[derive(Debug, Clone, PartialEq)]
struct Header {
	id1: Option<Identification>,
	id2: Option<Identification>,
	cm: Option<CompressionMethod>,
	flags: Option<Flags>,
	mtime: Option<MTime>,
	xfl: Option<ExtraFlags>,
	os: Option<OS>,
	xlen: Option<XLen>,
	extra_field: Option<ExtraField>,
	file_name: Option<FileName>,
	file_comment: Option<FileComment>,
	crc16: Option<CRC16>,
}

impl Header {
	fn new() -> Header {
		Header{
			id1: None,
			id2: None,
			cm: None,
			flags: None,
			mtime: None,
			xfl: None,
			os: None,
			xlen: None,
			extra_field: None,
			file_name: None,
			file_comment: None,
			crc16: None,
		}
	}
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
type ExtraField = Vec<u8>;

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

type Symbol = u16;
// type HuffmanCodes = huffman::tree::Tree;

#[derive(Debug, Clone, PartialEq)]
enum State {
	HeaderBegin,
	Identification1(Identification),
	Identification2(Identification),
	CompressionMethod(CompressionMethod),
	Flags(Flags),
	MTime(MTime),
	ExtraFlags(ExtraFlags),
	OS(OS),
	ParsingXLen(bool),
	XLen(XLen),
	ExtraField(ExtraField),
	ParsingFileName(bool),
	FileName(FileName),
	ParsingFileComment(bool),
	FileComment(FileComment),
	ParsingCRC16(bool),
	CRC16(CRC16),
	HeaderEnd,
	InSubDecompressor,
}

#[derive(Debug, Clone, PartialEq)]
enum DecompressorError {
	UnexpectedEOF,
	NonZeroReservedFlag,
	NonZeroReservedExtraFlag,
	InvalidCompressionMethod,
	InvalidIdentification,
	InvalidOS,
}

impl Display for DecompressorError {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
		fmt.write_str(self.description())
	}
}

impl Error for DecompressorError {
	fn description(&self) -> &str {
		match self {
			&DecompressorError::UnexpectedEOF => "Encountered unexpected EOF",
			&DecompressorError::NonZeroReservedFlag => "Non-zero reserved flag in gzip header",
			&DecompressorError::NonZeroReservedExtraFlag => "Non-zero reserved extra-flag in gzip header",
			&DecompressorError::InvalidCompressionMethod => "Invalid compression method in gzip header",
			&DecompressorError::InvalidIdentification => "Invalid identification in gzip header (not a gzip file)",
			&DecompressorError::InvalidOS => "Invalid OS identification in gzip header",
		}
	}
}

impl<R: Read> Decompressor<R> {
	pub fn new(in_stream: BitReader<R>) -> Decompressor<R> {
		Decompressor{
			in_stream: in_stream,

			header: Header::new(),

			buf: VecDeque::new(),

			state: State::HeaderBegin,

			sub_decompressor: None,
		}
	}

	fn parse_identification1(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_u8() {
			Ok(U8_IDENTIFICATION1_GZIP) => Ok(State::Identification1(Identification::Gzip)),
			Ok(_) => Err(DecompressorError::InvalidIdentification),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_identification2(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_u8() {
			Ok(U8_IDENTIFICATION2_GZIP) => Ok(State::Identification2(Identification::Gzip)),
			Ok(_) => Err(DecompressorError::InvalidIdentification),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_compression_method(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_u8() {
			Ok(U8_COMPRESSION_METHOD_DEFLATE) => Ok(State::CompressionMethod(CompressionMethod::Deflate)),
			Ok(_) => Err(DecompressorError::InvalidCompressionMethod),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_flags(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_n_bits(8) {
			Ok(flags) => {
				if flags[5] || flags[6] || flags[7] {

					Err(DecompressorError::NonZeroReservedFlag)
				} else {
					Ok(State::Flags(Flags{
						ftext:    flags[0],
						fhcrc:    flags[1],
						fextra:   flags[2],
						fname:    flags[3],
						fcomment: flags[4],
					}))
				}
			},
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_mtime(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_u32() {
			Ok(mtime) => Ok(State::MTime(mtime)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_xfl(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_n_bits(8) {
			Ok(flags) => {
				if flags[0] || flags[3] || flags[4] || flags[5] || flags[6] || flags[7] {

					Err(DecompressorError::NonZeroReservedExtraFlag)
				} else {
					Ok(State::ExtraFlags(ExtraFlags{
						maximum_compression: flags[1],
						fastest_algorithm:   flags[2],
					}))
				}
			},
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_os(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_u8() {
			Ok(0) => Ok(State::OS(OS::FATFilesystem)),
			Ok(1) => Ok(State::OS(OS::Amiga)),
			Ok(2) => Ok(State::OS(OS::VMS)),
			Ok(3) => Ok(State::OS(OS::Unix)),
			Ok(4) => Ok(State::OS(OS::VMCMS)),
			Ok(5) => Ok(State::OS(OS::AtariTOS)),
			Ok(6) => Ok(State::OS(OS::HPFSFilesystem)),
			Ok(7) => Ok(State::OS(OS::Macintosh)),
			Ok(8) => Ok(State::OS(OS::ZSystem)),
			Ok(9) => Ok(State::OS(OS::CPM)),
			Ok(10) => Ok(State::OS(OS::TOPS20)),
			Ok(11) => Ok(State::OS(OS::NTFSFilesystem)),
			Ok(12) => Ok(State::OS(OS::QDOS)),
			Ok(13) => Ok(State::OS(OS::AcornRISCOS)),
			Ok(255) => Ok(State::OS(OS::Unknown)),
			Ok(_) => Err(DecompressorError::InvalidOS),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_xlen(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_u16() {
			Ok(xlen) => Ok(State::XLen(xlen)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_extra_field(&mut self, xlen: XLen) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_fixed_length_string(xlen as usize) {
			Ok(extra_field) => Ok(State::ExtraField(extra_field)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_file_name(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_zero_terminated_string() {
			Ok(file_name) => Ok(State::FileName(String::from_utf8(file_name).unwrap())),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_file_comment(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_zero_terminated_string() {
			Ok(file_comment) => Ok(State::FileComment(String::from_utf8(file_comment).unwrap())),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn parse_crc16(&mut self) -> result::Result<State, DecompressorError> {
		match self.in_stream.read_u16() {
			Ok(crc16) => Ok(State::CRC16(crc16)),
			Err(_) => Err(DecompressorError::UnexpectedEOF),
		}
	}

	fn decompress(&mut self) -> result::Result<usize, DecompressorError> {
		loop {
			match self.state.clone() {
				State::HeaderBegin => {
					self.state = match self.parse_identification1() {
						Ok(state) => state,
						Err(e) => panic!(e),
					}
				},
				State::Identification1(id) => {
					assert_eq!(Identification::Gzip, id);
					self.header.id1 = Some(id);
					self.state = match self.parse_identification2() {
						Ok(state) => state,
						Err(e) => panic!(e),
					}
				},
				State::Identification2(id) => {
					assert_eq!(Identification::Gzip, id);
					self.header.id2 = Some(id);
					self.state = match self.parse_compression_method() {
						Ok(state) => state,
						Err(e) => panic!(e),
					}
				},
				State::CompressionMethod(cm) => {
					assert_eq!(CompressionMethod::Deflate, cm);
					self.header.cm = Some(cm);
					self.state = match self.parse_flags() {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::Flags(flags) => {
					self.header.flags = Some(flags);
					self.state = match self.parse_mtime() {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::MTime(mtime) => {
					self.header.mtime = Some(mtime);
					self.state = match self.parse_xfl() {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::ExtraFlags(xfl) => {
					self.header.xfl = Some(xfl);
					self.state = match self.parse_os() {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::OS(os) => {
					self.header.os = Some(os);
					self.state = State::ParsingXLen(self.header.flags.as_ref().unwrap().fextra);
				},
				State::ParsingXLen(true) => {
					self.state = match self.parse_xlen() {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::ParsingXLen(false) => {

					self.state = State::ParsingFileName(self.header.flags.as_ref().unwrap().fname);
				},
				State::XLen(xlen) => {
					self.header.xlen = Some(xlen);
					self.state = match self.parse_extra_field(xlen) {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::ExtraField(extra_field) => {
					self.header.extra_field = Some(extra_field);
					self.state = State::ParsingFileName(self.header.flags.as_ref().unwrap().fname);
				},
				State::ParsingFileName(true) => {
					self.state = match self.parse_file_name() {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::ParsingFileName(false) => {

					self.state = State::ParsingFileComment(self.header.flags.as_ref().unwrap().fcomment);
				},
				State::FileName(file_name) => {
					self.header.file_name = Some(file_name);
					self.state = State::ParsingFileComment(self.header.flags.as_ref().unwrap().fcomment);
				},
				State::ParsingFileComment(true) => {
					self.state = match self.parse_file_comment() {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::ParsingFileComment(false) => {

					self.state = State::ParsingCRC16(self.header.flags.as_ref().unwrap().fhcrc);
				},
				State::FileComment(file_comment) => {
					self.header.file_comment = Some(file_comment);
					self.state = State::ParsingCRC16(self.header.flags.as_ref().unwrap().fhcrc);
				},
				State::ParsingCRC16(false) => {

					self.state = State::HeaderEnd;
				},
				State::ParsingCRC16(true) => {
					self.state = match self.parse_crc16() {
						Ok(state) => state,
						Err(e) => panic!(e),
					};
				},
				State::CRC16(crc16) => {
					self.header.crc16 = Some(crc16);
					self.state = State::HeaderEnd;
				},
				State::HeaderEnd => {
					match (&self.sub_decompressor, &self.header.cm) {
						(&None, &Some(CompressionMethod::Deflate)) => self.sub_decompressor = Some(deflate::Decompressor::new()),
						(&Some(_), &Some(CompressionMethod::Deflate)) => {},
						(_, &None) => unreachable!(),
					};
					self.state = State::InSubDecompressor;
				},
				State::InSubDecompressor => {
					return match self.sub_decompressor.as_mut().unwrap().decompress(&mut self.in_stream) {
						ref v => if v.len() > 0 {
							for &b in v {
								self.buf.push_front(b);
							}
							Ok(v.len())
						} else {
							Ok(0)
						}
					}
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

			buf[i] = self.buf.pop_back().unwrap();
		}

		Ok(l)
	}
}
