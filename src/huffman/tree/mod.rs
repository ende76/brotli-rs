/// For Huffman codes used in the Deflate spec, is seems that the length of a code is at most 9 bits.
/// For this simple use case, we don't need/want to deal with type parameters.
type Symbol = u16;

#[derive(Debug, Clone, PartialEq)]
struct Node {
	left: Option<Box<Tree>>,
	right: Option<Box<Tree>>,
}

#[derive(Debug, Clone, PartialEq)]
enum Tree {
	Leaf(Symbol),
	Inner(Node),
}

impl Tree {
	fn new() -> Tree {
		Tree::Inner(Node{
			left: None,
			right: None,
		})
	}

	fn insert(&mut self, code: Vec<bool>, symbol: Symbol) {
		if code.len() == 1 {
			if code[0] {
				*self = Tree::Inner(Node{
					left: match self {
						&mut Tree::Inner(Node{
							ref left,
							right: _,
						}) => left.clone(),
						&mut Tree::Leaf(_) => unreachable!(),
					},
					right: Some(Box::new(Tree::Leaf(symbol))),
				});
			} else {
				*self = Tree::Inner(Node{
					left: Some(Box::new(Tree::Leaf(symbol))),
					right: match self {
						&mut Tree::Inner(Node{
							left: _,
							ref right,
						}) => right.clone(),
						&mut Tree::Leaf(_) => unreachable!(),
					},
				});
			}
		} else {
			if code[0] {
				match self {
					&mut Tree::Inner(Node{
						left: _,
						ref mut right,
					}) => {
						match right {
							&mut None => *right = Some(Box::new(Tree::new())),
							&mut Some(_) => {
								// Nothing to do
							},
						};
						// Now we can be sure that right is a subtree. So we can delegate to it.

						match right {
							&mut Some(ref mut boxed_tree) => (*boxed_tree).insert(code[1..].to_vec(), symbol),
							_ => unreachable!(),
						};
					},
					&mut Tree::Leaf(_) => unreachable!(),
				}
			} else {
				match self {
					&mut Tree::Inner(Node{
						ref mut left,
						right: _,
					}) => {
						match left {
							&mut None => *left = Some(Box::new(Tree::new())),
							&mut Some(_) => {
								// Nothing to do
							},
						};
						// Now we can be sure that left is a subtree. So we can delegate to it.

						match left {
							&mut Some(ref mut boxed_tree) => (*boxed_tree).insert(code[1..].to_vec(), symbol),
							_ => unreachable!(),
						};
					},
					&mut Tree::Leaf(_) => unreachable!(),
				}
			}
		}
	}
}


mod tests {
	#[test]
	fn should_create_empty_tree() {
		use super::{Tree, Node};
		use super::Tree::Inner;

		assert_eq!(Tree::new(), Inner(Node{
			left: None,
			right: None,
		}));
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
}



