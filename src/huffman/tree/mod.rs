use ::bitreader::BitReader;
use std::io::Read;

mod pseudocode;

// For Huffman codes used in the Brotli spec, is seems that the length of a
// code is at most 10 bits (max alphabet size is 704).
// For this simple use case, we don't need/want to deal with type parameters.
pub type Symbol = u16;

#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
	buf: Vec<Option<Symbol>>,
	len: usize,
	last_symbol: Option<Symbol>,
}

// Index structure in self.buf[]:
//
//
//            0                ^
//           / \               |
// left <-- /   \ --> right    |
//         /     \             |
//        /       \
//       /         \       Max Code Length == Max Number of Edges in a Path == Max Tree Depth
//      1           2      (in this example it's 3)
//     / \         / \
//    /   \       /   \        |
//   3     4     5     6       |
//  / \   / \   / \   / \      |
// 7   8 9  10 11 12 13 14     v
// …
//
// Length of self.buf[] = 2^(codelength + 1) - 1
//

impl Tree {
	pub fn with_max_depth(max_depth: usize) -> Tree {
		Tree {
			buf: vec![None; (1 << (max_depth + 1)) - 1],
			len: 0,
			last_symbol: None,
		}
	}

	pub fn from_raw_data(buf: Vec<Option<Symbol>>, len: usize, last_symbol: Option<Symbol>) -> Tree {
		Tree {
			buf: buf,
			len: len,
			last_symbol: last_symbol,
		}
	}

	pub fn insert(&mut self, code: &[bool], symbol: Symbol) {
		self.len += 1;
		self.last_symbol = Some(symbol);

		let insert_at_index = (1 << code.len()) - 1 + code.iter().fold(0, |acc, &bit| (acc << 1) + if bit { 1 } else { 0 });

		if insert_at_index > self.buf.len() - 1 {
			panic!("Index {:?} exceeds MAX_INDEX at insert (code = {:?})", insert_at_index, code);
		}

		self.buf[insert_at_index] = Some(symbol)
	}

	fn lookup<R: Read>(&self, r: &mut BitReader<R>) -> Result<Option<Symbol>, ::bitreader::BitReaderError> {
		let mut pseudo_code = 1;
		loop {
			pseudo_code = (pseudo_code << 1) + match r.read_bit_as_usize() {
				Ok(b) => b,
				Err(e) => return Err(e),
			};

			let lookup_index = pseudocode::LUT_PSEUDO_CODE[pseudo_code];

			if lookup_index > self.buf.len() - 1 {
				return Ok(None);
			}

			match self.buf[lookup_index] {
				Some(symbol) => return Ok(Some(symbol)),
				None => {},
			}
		}
	}

	pub fn lookup_symbol<R: Read>(&self, mut r: &mut BitReader<R>) -> Result<Option<Symbol>, ::bitreader::BitReaderError, >  {
		// println!("self.len = {:?}", self.len);

		match self.len {
			0 => Ok(None),
			1 => Ok(self.last_symbol),
			_ => self.lookup(&mut r),
		}
	}
}


mod tests {
	#[test]
	fn should_insert_and_lookup_first_level_leaf_on_left() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![0]));
		let mut tree = Tree::with_max_depth(1);
		tree.insert(&vec![false], 666);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(666)));
	}

	#[test]
	fn should_insert_and_lookup_first_level_leaf_on_right() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![1]));
		let mut tree = Tree::with_max_depth(1);
		tree.insert(&vec![true], 666);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(666)));
	}

	#[test]
	fn should_insert_first_level_leaf_on_left_then_on_right() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![2]));
		let mut tree = Tree::with_max_depth(1);
		tree.insert(&vec![false], 667);
		tree.insert(&vec![true], 666);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(667)));
		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(666)));
	}

	#[test]
	fn should_insert_first_level_leaf_on_right_then_on_left() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![1]));
		let mut tree = Tree::with_max_depth(1);
		tree.insert(&vec![true], 666);
		tree.insert(&vec![false], 667);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(666)));
		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(667)));
	}

	#[test]
	fn should_insert_second_level_leaf_left_right() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![2]));
		let mut tree = Tree::with_max_depth(2);
		tree.insert(&vec![false, true], 6666);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(6666)));
	}

	#[test]
	fn should_insert_second_level_leaf_right_left() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![1]));
		let mut tree = Tree::with_max_depth(2);
		tree.insert(&vec![true, false], 6666);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(6666)));
	}

	#[test]
	fn should_lookup_first_level_leaf_left() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![0b11001]));
		let mut tree = Tree::with_max_depth(2);
		tree.insert(&vec![true, false], 6666);
		tree.insert(&vec![false], 666);
		tree.insert(&vec![true, true], 6667);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(6666)));
		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(666)));
		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(6667)));
	}

	#[test]
	fn should_lookup_first_level_leaf_right() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![0b10100]));
		let mut tree = Tree::with_max_depth(2);
		tree.insert(&vec![false, false], 6666);
		tree.insert(&vec![true], 666);
		tree.insert(&vec![false, true], 6667);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(6666)));
		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(666)));
		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Ok(Some(6667)));
	}
}
