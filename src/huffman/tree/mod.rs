use ::bitreader::BitReader;
use std::io::Read;

/// For Huffman codes used in the Deflate spec, is seems that the length of a code is at most 9 bits.
/// For this simple use case, we don't need/want to deal with type parameters.
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

	pub fn is_empty(&self) -> bool {
		*self == EMPTY_NODE
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


// These are not very good tests, because they rely
// on implementation details, for the tree data structure
mod tests {
	#[test]
	fn should_create_empty_tree() {
		use super::Tree;
		assert!(Tree::new().is_empty());
	}

	#[test]
	fn should_create_different_instances() {
		use super::Tree;
		let mut tree_0 = Tree::new();
		let tree_1 = Tree::new();

		tree_0.insert(vec![false], 666);
		assert!(!tree_0.is_empty());
		assert!(tree_1.is_empty());
	}

	#[test]
	fn should_insert_first_level_leaf_on_left() {
		use super::{Tree, Node};
		use super::Tree::{ Inner, Leaf };

		let mut tree = Tree::new();
		tree.insert(vec![false], 666);

		assert_eq!(Inner(Node{
			left: Some(Box::new(Leaf(666))),
			right: None,
		}), tree);
	}

	#[test]
	fn should_insert_first_level_leaf_on_right() {
		use super::{Tree, Node};
		use super::Tree::{ Inner, Leaf };

		let mut tree = Tree::new();
		tree.insert(vec![true], 666);

		assert_eq!(Inner(Node{
			left: None,
			right: Some(Box::new(Leaf(666))),
		}), tree);
	}

	#[test]
	fn should_insert_first_level_leaf_on_left_then_on_right() {
		use super::{Tree, Node};
		use super::Tree::{ Inner, Leaf };

		let mut tree = Tree::new();
		tree.insert(vec![false], 667);
		tree.insert(vec![true], 666);

		assert_eq!(Inner(Node{
			left: Some(Box::new(Leaf(667))),
			right: Some(Box::new(Leaf(666))),
		}), tree);
	}

	#[test]
	fn should_insert_first_level_leaf_on_right_then_on_left() {
		use super::{Tree, Node};
		use super::Tree::{ Inner, Leaf };

		let mut tree = Tree::new();
		tree.insert(vec![true], 666);
		tree.insert(vec![false], 667);

		assert_eq!(Inner(Node{
			left: Some(Box::new(Leaf(667))),
			right: Some(Box::new(Leaf(666))),
		}), tree);
	}

	#[test]
	fn should_insert_second_level_leaf_left_right() {
		use super::{Tree, Node};
		use super::Tree::{ Inner, Leaf };

		let mut tree = Tree::new();
		tree.insert(vec![false, true], 6666);

		assert_eq!(Inner(Node{
			left: Some(Box::new(Inner(Node{
				left: None,
				right: Some(Box::new(Leaf(6666))),
			}))),
			right: None
		}), tree);
	}

	#[test]
	fn should_insert_second_level_leaf_right_left() {
		use super::{Tree, Node};
		use super::Tree::{ Inner, Leaf };

		let mut tree = Tree::new();
		tree.insert(vec![true, false], 6666);

		assert_eq!(Inner(Node{
			left: None,
			right: Some(Box::new(Inner(Node{
				left: Some(Box::new(Leaf(6666))),
				right: None,
			}))),
		}), tree);
	}

	#[test]
	fn should_lookup_first_level_leaf_left() {
		use super::Tree;
		use super::Tree::Leaf;

		let mut tree = Tree::new();
		tree.insert(vec![true, false], 6666);
		tree.insert(vec![false], 666);
		tree.insert(vec![true, true], 6667);

		assert_eq!(Some(Leaf(666)), tree.lookup(false));
	}

	#[test]
	fn should_lookup_first_level_leaf_right() {
		use super::Tree;
		use super::Tree::Leaf;

		let mut tree = Tree::new();
		tree.insert(vec![false, false], 6666);
		tree.insert(vec![true], 666);
		tree.insert(vec![false, true], 6667);

		assert_eq!(Some(Leaf(666)), tree.lookup(true));
	}

	#[test]
	fn should_lookup_first_level_node_left() {
		use super::{ Tree, Node };
		use super::Tree::{ Inner, Leaf };

		let mut tree = Tree::new();
		tree.insert(vec![false, false], 6666);
		tree.insert(vec![true], 666);
		tree.insert(vec![false, true], 6667);

		assert_eq!(Some(Inner(Node{
			left: Some(Box::new(Leaf(6666))),
			right: Some(Box::new(Leaf(6667))),
		})), tree.lookup(false));
	}

	#[test]
	fn should_result_in_none() {
		use super::Tree;

		let mut tree = Tree::new();
		tree.insert(vec![true], 666);

		assert_eq!(None, tree.lookup(false));
	}
}
