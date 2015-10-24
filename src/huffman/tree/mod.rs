use ::bitreader::BitReader;
use std::io::Read;

// For Huffman codes used in the Deflate spec, is seems that the length of a
// code is at most 10 bits (max alphabet size is 704).
// For this simple use case, we don't need/want to deal with type parameters.
pub type Symbol = u16;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
	left: Option<Box<Tree>>,
	right: Option<Box<Tree>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tree {
	Leaf(Symbol),
	Inner(Node),
}

const EMPTY_NODE: Tree = Tree::Inner(Node{
	left: None,
	right: None,
});


impl Tree {
	pub fn new() -> Tree {
		EMPTY_NODE
	}

	pub fn insert(&mut self, code: Vec<bool>, symbol: Symbol) {
		if code.len() == 1 {
			if code[0] {
				*self = Tree::Inner(Node{
					left: match *self {
						Tree::Inner(Node{
							ref left,
							right: _,
						}) => left.clone(),
						Tree::Leaf(_) => unreachable!(),
					},
					right: Some(Box::new(Tree::Leaf(symbol))),
				});
			} else {
				*self = Tree::Inner(Node{
					left: Some(Box::new(Tree::Leaf(symbol))),
					right: match *self {
						Tree::Inner(Node{
							left: _,
							ref right,
						}) => right.clone(),
						Tree::Leaf(_) => unreachable!(),
					},
				});
			}
		} else {
			if code[0] {
				match *self {
					Tree::Inner(Node{
						left: _,
						ref mut right,
					}) => {
						match *right {
							None => *right = Some(Box::new(Tree::new())),
							Some(_) => {
								// Nothing to do
							},
						};
						// Now we can be sure that right is a subtree. So we can delegate to it.

						match *right {
							Some(ref mut boxed_tree) => (*boxed_tree).insert(code[1..].to_vec(), symbol),
							_ => unreachable!(),
						};
					},
					Tree::Leaf(_) => unreachable!(),
				}
			} else {
				match *self {
					Tree::Inner(Node{
						ref mut left,
						right: _,
					}) => {
						match *left {
							None => *left = Some(Box::new(Tree::new())),
							Some(_) => {
								// Nothing to do
							},
						};
						// Now we can be sure that left is a subtree. So we can delegate to it.

						match *left {
							Some(ref mut boxed_tree) => (*boxed_tree).insert(code[1..].to_vec(), symbol),
							_ => unreachable!(),
						};
					},
					Tree::Leaf(_) => unreachable!(),
				}
			}
		}
	}

	pub fn lookup(&self, c: bool) -> Option<Tree> {
		match *self {
			Tree::Leaf(_) => None,
			Tree::Inner(Node{
				ref left,
				ref right
			}) =>
				if c {
					match *right {
						Some(ref boxed_tree) => Some((**boxed_tree).clone()),
						None => None,
					}
				} else {
					match *left {
						Some(ref boxed_tree) => Some((**boxed_tree).clone()),
						None => None,
					}
				}
		}
	}

	pub fn lookup_symbol<R: Read>(&self, r: &mut BitReader<R>) -> Option<Symbol> {
		loop {
			match r.read_bit() {
				Ok(bit) =>
					match self.lookup(bit) {
						Some(Tree::Leaf(symbol)) => return Some(symbol),
						Some(inner) => return inner.lookup_symbol(r),
						None => unreachable!(),
					},
				Err(_) => return None,
			}
		}
	}
}


mod tests {
	#[test]
	fn should_create_empty_tree() {
		use super::Tree;
		assert_eq!(Tree::new(), super::EMPTY_NODE);
	}

	#[test]
	fn should_create_different_instances() {
		use super::Tree;
		let mut tree_0 = Tree::new();
		let tree_1 = Tree::new();

		tree_0.insert(vec![false], 666);
		assert!(tree_0 != super::EMPTY_NODE);
		assert!(tree_1 == super::EMPTY_NODE);
	}

	#[test]
	fn should_insert_and_lookup_first_level_leaf_on_left() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![0]));
		let mut tree = Tree::new();
		tree.insert(vec![false], 666);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(666));
	}

	#[test]
	fn should_insert_and_lookup_first_level_leaf_on_right() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![1]));
		let mut tree = Tree::new();
		tree.insert(vec![true], 666);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(666));
	}

	#[test]
	fn should_insert_first_level_leaf_on_left_then_on_right() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![2]));
		let mut tree = Tree::new();
		tree.insert(vec![false], 667);
		tree.insert(vec![true], 666);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(667));
		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(666));
	}

	#[test]
	fn should_insert_first_level_leaf_on_right_then_on_left() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![1]));
		let mut tree = Tree::new();
		tree.insert(vec![true], 666);
		tree.insert(vec![false], 667);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(666));
		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(667));
	}

	#[test]
	fn should_insert_second_level_leaf_left_right() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![2]));
		let mut tree = Tree::new();
		tree.insert(vec![false, true], 6666);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(6666));
	}

	#[test]
	fn should_insert_second_level_leaf_right_left() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![1]));
		let mut tree = Tree::new();
		tree.insert(vec![true, false], 6666);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(6666));
	}

	#[test]
	fn should_lookup_first_level_leaf_left() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![0b11001]));
		let mut tree = Tree::new();
		tree.insert(vec![true, false], 6666);
		tree.insert(vec![false], 666);
		tree.insert(vec![true, true], 6667);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(6666));
		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(666));
		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(6667));
	}

	#[test]
	fn should_lookup_first_level_leaf_right() {
		use ::bitreader::BitReader;
		use super::Tree;
		use std::io::Cursor;

		let mut lookup_stream = BitReader::new(Cursor::new(vec![0b10100]));
		let mut tree = Tree::new();
		tree.insert(vec![false, false], 6666);
		tree.insert(vec![true], 666);
		tree.insert(vec![false, true], 6667);

		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(6666));
		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(666));
		assert_eq!(tree.lookup_symbol(&mut lookup_stream), Some(6667));
	}

	#[test]
	fn should_result_in_none() {
		use super::Tree;

		let mut tree = Tree::new();
		tree.insert(vec![true], 666);

		assert_eq!(None, tree.lookup(false));
	}
}
