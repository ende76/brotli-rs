
fn uppercase_all(base_word: &[u8]) -> Vec<u8> {
	let l = base_word.len();
	let mut v = Vec::with_capacity(l);
	let mut i = 0;

	while i < l {
		match base_word[i] {
			1...96|123...191 => {
				v.push(base_word[i]);
				i += 1;
			},
			97...122 => {
				v.push(base_word[i] ^ 32);
				i += 1;
			},
			192...223 => {
				v.push(base_word[i]);
				if i + 1 < l {
					v.push(base_word[i + 1] ^ 32);
				}
				i += 2;
			},
			224...255 => {
				v.push(base_word[i]);
				if i + 1 < l {
					v.push(base_word[i + 1]);
				}
				if i + 2 < l {
					v.push(base_word[i + 2] ^ 5);
				}
				i = i + 3;
			},
			_ => unreachable!(),
		}
	}

	v
}

fn uppercase_first(base_word: &[u8]) -> Vec<u8> {
	let l = base_word.len();
	let mut v = Vec::with_capacity(l);
	let i;

	match base_word[0] {
		1...96|123...191 => {
			v.push(base_word[0]);
			i = 1;
		},
		97...122 => {
			v.push(base_word[0] ^ 32);
			i = 1;
		},
		192...223 => {
			v.push(base_word[0]);
			if 1 < l {
				v.push(base_word[1] ^ 32);
			}
			i = 2;
		},
		224...255 => {
			v.push(base_word[0]);
			if 1 < l {
				v.push(base_word[1]);
			}
			if 2 < l {
				v.push(base_word[2] ^ 5);
			}
			i = 3;
		},
		_ => unreachable!(),
	}

	[v, Vec::from(&base_word[i..])].concat()
}


pub fn transformation(id: usize, base_word: &[u8]) -> Vec<u8> {
	match id {
		  0 => Vec::from(base_word),
		  1 => [Vec::from(base_word), vec![0x20]].concat(),
		  2 => [vec![0x20], Vec::from(base_word), vec![0x20]].concat(),
		  3 => Vec::from(&base_word[1..]),
		  4 => [uppercase_first(base_word), vec![0x20]].concat(),
		  5 => [Vec::from(base_word), vec![0x20, 0x74, 0x68, 0x65, 0x20]].concat(),
		  6 => [vec![0x20], Vec::from(base_word)].concat(),
		  7 => [vec![0x73, 0x20], Vec::from(base_word), vec![0x20]].concat(),
		  8 => [Vec::from(base_word), vec![0x20, 0x6f, 0x66, 0x20]].concat(),
		  9 => uppercase_first(base_word),
		 10 => [Vec::from(base_word), vec![0x20, 0x61, 0x6e, 0x64, 0x20]].concat(),
		 11 => Vec::from(&base_word[2..]),
		 12 => Vec::from(&base_word[0..base_word.len()-1]),
		 13 => [vec![0x2c, 0x20], Vec::from(base_word), vec![0x20]].concat(),
		 14 => [Vec::from(base_word), vec![0x2c, 0x20]].concat(),
		 15 => [vec![0x20], uppercase_first(base_word), vec![0x20]].concat(),
		 16 => [Vec::from(base_word), vec![0x20, 0x69, 0x6e, 0x20]].concat(),
		 17 => [Vec::from(base_word), vec![0x20, 0x74, 0x6f, 0x20]].concat(),
		 18 => [vec![0x65, 0x20], Vec::from(base_word), vec![0x20]].concat(),
		 19 => [Vec::from(base_word), vec![0x22]].concat(),
		 20 => [Vec::from(base_word), vec![0x2e]].concat(),
		 21 => [Vec::from(base_word), vec![0x22, 0x3e]].concat(),
		 22 => [Vec::from(base_word), vec![0x0a]].concat(),
		 23 => Vec::from(&base_word[0..base_word.len() - 3]),
		 24 => [Vec::from(base_word), vec![0x5d]].concat(),
		 25 => [Vec::from(base_word), vec![0x20, 0x66, 0x6f, 0x72, 0x20]].concat(),
		 26 => Vec::from(&base_word[3..]),
		 27 => Vec::from(&base_word[0..base_word.len() - 2]),
		 28 => [Vec::from(base_word), vec![0x20, 0x61, 0x20]].concat(),
		 29 => [Vec::from(base_word), vec![0x20, 0x74, 0x68, 0x61, 0x74, 0x20]].concat(),
		 30 => [vec![0x20], uppercase_first(base_word)].concat(),
		 31 => [Vec::from(base_word), vec![0x2e, 0x20]].concat(),
		 32 => [vec![0x2e], Vec::from(base_word)].concat(),
		 33 => [vec![0x20], Vec::from(base_word), vec![0x2c, 0x20]].concat(),
		 34 => Vec::from(&base_word[4..]),
		 35 => [Vec::from(base_word), vec![0x20, 0x77, 0x69, 0x74, 0x68, 0x20]].concat(),
		 36 => [Vec::from(base_word), vec![0x27]].concat(),
		 37 => [Vec::from(base_word), vec![0x20, 0x66, 0x72, 0x6f, 0x6d, 0x20]].concat(),
		 38 => [Vec::from(base_word), vec![0x20, 0x62, 0x79, 0x20]].concat(),
		 39 => Vec::from(&base_word[5..]),
		 40 => Vec::from(&base_word[6..]),
		 41 => [vec![0x20, 0x74, 0x68, 0x65, 0x20], Vec::from(base_word)].concat(),
		 42 => Vec::from(&base_word[0..base_word.len() - 4]),
		 43 => [Vec::from(base_word), vec![0x2e, 0x20, 0x54, 0x68, 0x65, 0x20]].concat(),
		 44 => uppercase_all(base_word),
		 45 => [Vec::from(base_word), vec![0x20, 0x6f, 0x6e, 0x20]].concat(),
		 46 => [Vec::from(base_word), vec![0x20, 0x61, 0x73, 0x20]].concat(),
		 47 => [Vec::from(base_word), vec![0x20, 0x69, 0x73, 0x20]].concat(),
		 48 => Vec::from(&base_word[0..base_word.len() - 7]),
		 49 => [Vec::from(&base_word[0..base_word.len() - 1]), vec![0x69, 0x6e, 0x67, 0x20]].concat(),
		 50 => [Vec::from(base_word), vec![0x0a, 0x09]].concat(),
		 51 => [Vec::from(base_word), vec![0x3a]].concat(),
		 52 => [vec![0x20], Vec::from(base_word), vec![0x2e, 0x20]].concat(),
		 53 => [Vec::from(base_word), vec![0x65, 0x64, 0x20]].concat(),
		 54 => Vec::from(&base_word[9..]),
		 55 => Vec::from(&base_word[7..]),
		 56 => Vec::from(&base_word[..base_word.len() - 6]),
		 57 => [Vec::from(base_word), vec![0x28]].concat(),
		 58 => [uppercase_first(base_word), vec![0x2c, 0x20]].concat(),
		 59 => Vec::from(&base_word[..base_word.len() - 8]),
		 60 => [Vec::from(base_word), vec![0x20, 0x61, 0x74, 0x20]].concat(),
		 61 => [Vec::from(base_word), vec![0x6c, 0x79, 0x20]].concat(),
		 62 => [vec![0x20, 0x74, 0x68, 0x65, 0x20], Vec::from(base_word), vec![0x20, 0x6f, 0x66, 0x20]].concat(),
		 63 => Vec::from(&base_word[..base_word.len() - 5]),
		 64 => Vec::from(&base_word[..base_word.len() - 9]),
		 65 => [vec![0x20], uppercase_first(base_word), vec![0x2c, 0x20]].concat(),
		 66 => [uppercase_first(base_word), vec![0x22]].concat(),
		 67 => [vec![0x2e], Vec::from(base_word), vec![0x28]].concat(),
		 68 => [uppercase_all(base_word), vec![0x20]].concat(),
		 69 => [uppercase_first(base_word), vec![0x22, 0x3e]].concat(),
		 70 => [Vec::from(base_word), vec![0x3d, 0x22]].concat(),
		 71 => [vec![0x20], Vec::from(base_word), vec![0x2e]].concat(),
		 72 => [vec![0x2e, 0x63, 0x6f, 0x6d, 0x2f], Vec::from(base_word)].concat(),
		 73 => [vec![0x20, 0x74, 0x68, 0x65, 0x20], Vec::from(base_word), vec![0x20, 0x6f, 0x66, 0x20, 0x74, 0x68, 0x65, 0x20]].concat(),
		 74 => [uppercase_first(base_word), vec![0x27]].concat(),
		 75 => [Vec::from(base_word), vec![0x2e, 0x20, 0x54, 0x68, 0x69, 0x73, 0x20]].concat(),
		 76 => [Vec::from(base_word), vec![0x2c]].concat(),
		 77 => [vec![0x2e], Vec::from(base_word), vec![0x20]].concat(),
		 78 => [uppercase_first(base_word), vec![0x28]].concat(),
		 79 => [uppercase_first(base_word), vec![0x2e]].concat(),
		 80 => [Vec::from(base_word), vec![0x20, 0x6e, 0x6f, 0x74, 0x20]].concat(),
		 81 => [vec![0x20], Vec::from(base_word), vec![0x3d, 0x22]].concat(),
		 82 => [Vec::from(base_word), vec![0x65, 0x72, 0x20]].concat(),
		 83 => [vec![0x20], uppercase_all(base_word), vec![0x20]].concat(),
		 84 => [Vec::from(base_word), vec![0x61, 0x6c, 0x20]].concat(),
		 85 => [vec![0x20], uppercase_all(base_word)].concat(),
		 86 => [Vec::from(base_word), vec![0x3d, 0x27]].concat(),
		 87 => [uppercase_all(base_word), vec![0x22]].concat(),
		 88 => [uppercase_first(base_word), vec![0x2e, 0x20]].concat(),
		 89 => [vec![0x20], Vec::from(base_word), vec![0x28]].concat(),
		 90 => [Vec::from(base_word), vec![0x66, 0x75, 0x6c, 0x20]].concat(),
		 91 => [vec![0x20], uppercase_first(base_word), vec![0x2e, 0x20]].concat(),
		 92 => [Vec::from(base_word), vec![0x69, 0x76, 0x65, 0x20]].concat(),
		 93 => [Vec::from(base_word), vec![0x6c, 0x65, 0x73, 0x73, 0x20]].concat(),
		 94 => [uppercase_all(base_word), vec![0x27]].concat(),
		 95 => [Vec::from(base_word), vec![0x65, 0x73, 0x74, 0x20]].concat(),
		 96 => [vec![0x20], uppercase_first(base_word), vec![0x2e]].concat(),
		 97 => [uppercase_all(base_word), vec![0x22, 0x3e]].concat(),
		 98 => [vec![0x20], Vec::from(base_word), vec![0x3d, 0x27]].concat(),
		 99 => [uppercase_first(base_word), vec![0x2c]].concat(),
		100 => [Vec::from(base_word), vec![0x69, 0x7a, 0x65, 0x20]].concat(),
		101 => [uppercase_all(base_word), vec![0x2e]].concat(),
		102 => [vec![0xc2, 0xa0], Vec::from(base_word)].concat(),
		103 => [vec![0x20], Vec::from(base_word), vec![0x2c]].concat(),
		104 => [uppercase_first(base_word), vec![0x3d, 0x22]].concat(),
		105 => [uppercase_all(base_word), vec![0x3d, 0x22]].concat(),
		106 => [Vec::from(base_word), vec![0x6f, 0x75, 0x73, 0x20]].concat(),
		107 => [uppercase_all(base_word), vec![0x2c, 0x20]].concat(),
		108 => [uppercase_first(base_word), vec![0x3d, 0x27]].concat(),
		109 => [vec![0x20], uppercase_first(base_word), vec![0x2c]].concat(),
		110 => [vec![0x20], uppercase_all(base_word), vec![0x3d, 0x22]].concat(),
		111 => [vec![0x20], uppercase_all(base_word), vec![0x2c, 0x20]].concat(),
		112 => [uppercase_all(base_word), vec![0x2c]].concat(),
		113 => [uppercase_all(base_word), vec![0x28]].concat(),
		114 => [uppercase_all(base_word), vec![0x2e, 0x20]].concat(),
		115 => [vec![0x20], uppercase_all(base_word), vec![0x2e]].concat(),
		116 => [uppercase_all(base_word), vec![0x3d, 0x27]].concat(),
		117 => [vec![0x20], uppercase_all(base_word), vec![0x2e, 0x20]].concat(),
		118 => [vec![0x20], uppercase_first(base_word), vec![0x3d, 0x22]].concat(),
		119 => [vec![0x20], uppercase_all(base_word), vec![0x3d, 0x27]].concat(),
		120 => [vec![0x20], uppercase_first(base_word), vec![0x3d, 0x27]].concat(),
		_ => unreachable!(),
	}
}

#[cfg(test)]
mod tests {
    use super::transformation;

	#[test]
	fn should_transform_0 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd";

		assert_eq!(String::from_utf8(transformation(0, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_1 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd ";

		assert_eq!(String::from_utf8(transformation(1, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_2 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " ä bäse wörd ";

		assert_eq!(String::from_utf8(transformation(2, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_3 () {
		let base_word = String::from("a bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " bäse wörd";

		assert_eq!(String::from_utf8(transformation(3, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_4 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä bäse wörd ";

		assert_eq!(String::from_utf8(transformation(4, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_5 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd the ";

		assert_eq!(String::from_utf8(transformation(5, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_6 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " ä bäse wörd";

		assert_eq!(String::from_utf8(transformation(6, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_7 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "s ä bäse wörd ";

		assert_eq!(String::from_utf8(transformation(7, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_8 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd of ";

		assert_eq!(String::from_utf8(transformation(8, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_9 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä bäse wörd";

		assert_eq!(String::from_utf8(transformation(9, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_10 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd and ";

		assert_eq!(String::from_utf8(transformation(10, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_11 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " bäse wörd";

		assert_eq!(String::from_utf8(transformation(11, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_12 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wör";

		assert_eq!(String::from_utf8(transformation(12, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_13 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = ", ä bäse wörd ";

		assert_eq!(String::from_utf8(transformation(13, &base_word)).unwrap(), String::from(expected));
	}
	#[test]
	fn should_transform_14 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd, ";

		assert_eq!(String::from_utf8(transformation(14, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_15 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä bäse wörd ";

		assert_eq!(String::from_utf8(transformation(15, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_16 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd in ";

		assert_eq!(String::from_utf8(transformation(16, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_17 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd to ";

		assert_eq!(String::from_utf8(transformation(17, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_18 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "e ä bäse wörd ";

		assert_eq!(String::from_utf8(transformation(18, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_19 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd\"";

		assert_eq!(String::from_utf8(transformation(19, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_20 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd.";

		assert_eq!(String::from_utf8(transformation(20, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_21 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd\">";

		assert_eq!(String::from_utf8(transformation(21, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_22 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd\n";

		assert_eq!(String::from_utf8(transformation(22, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_23 () {
		let base_word = String::from("ä bäse word").bytes().collect::<Vec<_>>();

		let expected = "ä bäse w";

		assert_eq!(String::from_utf8(transformation(23, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_24 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd]";

		assert_eq!(String::from_utf8(transformation(24, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_25 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd for ";

		assert_eq!(String::from_utf8(transformation(25, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_26 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "bäse wörd";

		assert_eq!(String::from_utf8(transformation(26, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_27 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wö";

		assert_eq!(String::from_utf8(transformation(27, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_28 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd a ";

		assert_eq!(String::from_utf8(transformation(28, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_29 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd that ";

		assert_eq!(String::from_utf8(transformation(29, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_30 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä bäse wörd";

		assert_eq!(String::from_utf8(transformation(30, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_31 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd. ";

		assert_eq!(String::from_utf8(transformation(31, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_32 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = ".ä bäse wörd";

		assert_eq!(String::from_utf8(transformation(32, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_33 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " ä bäse wörd, ";

		assert_eq!(String::from_utf8(transformation(33, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_34 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "äse wörd";

		assert_eq!(String::from_utf8(transformation(34, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_35 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd with ";

		assert_eq!(String::from_utf8(transformation(35, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_36 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd'";

		assert_eq!(String::from_utf8(transformation(36, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_37 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd from ";

		assert_eq!(String::from_utf8(transformation(37, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_38 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd by ";

		assert_eq!(String::from_utf8(transformation(38, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_39 () {
		let base_word = String::from("ä base wörd").bytes().collect::<Vec<_>>();

		let expected = "se wörd";

		assert_eq!(String::from_utf8(transformation(39, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_40 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "se wörd";

		assert_eq!(String::from_utf8(transformation(40, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_41 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " the ä bäse wörd";

		assert_eq!(String::from_utf8(transformation(41, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_42 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse w";

		assert_eq!(String::from_utf8(transformation(42, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_43 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd. The ";

		assert_eq!(String::from_utf8(transformation(43, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_44 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä BÄSE WÖRD";

		assert_eq!(String::from_utf8(transformation(44, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_45 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd on ";

		assert_eq!(String::from_utf8(transformation(45, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_46 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd as ";

		assert_eq!(String::from_utf8(transformation(46, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_47 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd is ";

		assert_eq!(String::from_utf8(transformation(47, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_48 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäs";

		assert_eq!(String::from_utf8(transformation(48, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_49 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wöring ";

		assert_eq!(String::from_utf8(transformation(49, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_50 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd\n\t";

		assert_eq!(String::from_utf8(transformation(50, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_51 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd:";

		assert_eq!(String::from_utf8(transformation(51, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_52 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " ä bäse wörd. ";

		assert_eq!(String::from_utf8(transformation(52, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_53 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörded ";

		assert_eq!(String::from_utf8(transformation(53, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_54 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "wörd";

		assert_eq!(String::from_utf8(transformation(54, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_55 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "e wörd";

		assert_eq!(String::from_utf8(transformation(55, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_56 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse";

		assert_eq!(String::from_utf8(transformation(56, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_57 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd(";

		assert_eq!(String::from_utf8(transformation(57, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_58 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä bäse wörd, ";

		assert_eq!(String::from_utf8(transformation(58, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_59 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bä";

		assert_eq!(String::from_utf8(transformation(59, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_60 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd at ";

		assert_eq!(String::from_utf8(transformation(60, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_61 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wördly ";

		assert_eq!(String::from_utf8(transformation(61, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_62 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " the ä bäse wörd of ";

		assert_eq!(String::from_utf8(transformation(62, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_63 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse ";

		assert_eq!(String::from_utf8(transformation(63, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_64 () {
		let base_word = String::from("ä base wörd").bytes().collect::<Vec<_>>();

		let expected = "ä b";

		assert_eq!(String::from_utf8(transformation(64, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_65 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä bäse wörd, ";

		assert_eq!(String::from_utf8(transformation(65, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_66 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä bäse wörd\"";

		assert_eq!(String::from_utf8(transformation(66, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_67 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = ".ä bäse wörd(";

		assert_eq!(String::from_utf8(transformation(67, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_68 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä BÄSE WÖRD ";

		assert_eq!(String::from_utf8(transformation(68, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_69 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä bäse wörd\">";

		assert_eq!(String::from_utf8(transformation(69, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_70 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd=\"";

		assert_eq!(String::from_utf8(transformation(70, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_71 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " ä bäse wörd.";

		assert_eq!(String::from_utf8(transformation(71, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_72 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = ".com/ä bäse wörd";

		assert_eq!(String::from_utf8(transformation(72, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_73 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " the ä bäse wörd of the ";

		assert_eq!(String::from_utf8(transformation(73, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_74 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä bäse wörd'";

		assert_eq!(String::from_utf8(transformation(74, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_75 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd. This ";

		assert_eq!(String::from_utf8(transformation(75, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_76 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd,";

		assert_eq!(String::from_utf8(transformation(76, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_77 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = ".ä bäse wörd ";

		assert_eq!(String::from_utf8(transformation(77, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_78 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä bäse wörd(";

		assert_eq!(String::from_utf8(transformation(78, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_79 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä bäse wörd.";

		assert_eq!(String::from_utf8(transformation(79, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_80 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd not ";

		assert_eq!(String::from_utf8(transformation(80, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_81 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " ä bäse wörd=\"";

		assert_eq!(String::from_utf8(transformation(81, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_82 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörder ";

		assert_eq!(String::from_utf8(transformation(82, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_83 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä BÄSE WÖRD ";

		assert_eq!(String::from_utf8(transformation(83, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_84 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wördal ";

		assert_eq!(String::from_utf8(transformation(84, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_85 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä BÄSE WÖRD";

		assert_eq!(String::from_utf8(transformation(85, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_86 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wörd='";

		assert_eq!(String::from_utf8(transformation(86, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_87 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä BÄSE WÖRD\"";

		assert_eq!(String::from_utf8(transformation(87, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_88 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä bäse wörd. ";

		assert_eq!(String::from_utf8(transformation(88, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_89 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " ä bäse wörd(";

		assert_eq!(String::from_utf8(transformation(89, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_90 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wördful ";

		assert_eq!(String::from_utf8(transformation(90, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_91 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä bäse wörd. ";

		assert_eq!(String::from_utf8(transformation(91, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_92 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wördive ";

		assert_eq!(String::from_utf8(transformation(92, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_93 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wördless ";

		assert_eq!(String::from_utf8(transformation(93, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_94 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä BÄSE WÖRD'";

		assert_eq!(String::from_utf8(transformation(94, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_95 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wördest ";

		assert_eq!(String::from_utf8(transformation(95, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_96 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä bäse wörd.";

		assert_eq!(String::from_utf8(transformation(96, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_97 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä BÄSE WÖRD\">";

		assert_eq!(String::from_utf8(transformation(97, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_98 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " ä bäse wörd='";

		assert_eq!(String::from_utf8(transformation(98, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_99 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä bäse wörd,";

		assert_eq!(String::from_utf8(transformation(99, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_100 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wördize ";

		assert_eq!(String::from_utf8(transformation(100, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_101 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä BÄSE WÖRD.";

		assert_eq!(String::from_utf8(transformation(101, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_102 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = [vec![0xc2, 0xa0], base_word.clone()].concat();

		assert_eq!(transformation(102, &base_word), expected);
	}

	#[test]
	fn should_transform_103 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " ä bäse wörd,";

		assert_eq!(String::from_utf8(transformation(103, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_104 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä bäse wörd=\"";

		assert_eq!(String::from_utf8(transformation(104, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_105 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä BÄSE WÖRD=\"";

		assert_eq!(String::from_utf8(transformation(105, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_106 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "ä bäse wördous ";

		assert_eq!(String::from_utf8(transformation(106, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_107 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä BÄSE WÖRD, ";

		assert_eq!(String::from_utf8(transformation(107, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_108 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä bäse wörd='";

		assert_eq!(String::from_utf8(transformation(108, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_109 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä bäse wörd,";

		assert_eq!(String::from_utf8(transformation(109, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_110 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä BÄSE WÖRD=\"";

		assert_eq!(String::from_utf8(transformation(110, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_111 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä BÄSE WÖRD, ";

		assert_eq!(String::from_utf8(transformation(111, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_112 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä BÄSE WÖRD,";

		assert_eq!(String::from_utf8(transformation(112, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_113 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä BÄSE WÖRD(";

		assert_eq!(String::from_utf8(transformation(113, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_114 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä BÄSE WÖRD. ";

		assert_eq!(String::from_utf8(transformation(114, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_115 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä BÄSE WÖRD.";

		assert_eq!(String::from_utf8(transformation(115, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_116 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = "Ä BÄSE WÖRD='";

		assert_eq!(String::from_utf8(transformation(116, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_117 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä BÄSE WÖRD. ";

		assert_eq!(String::from_utf8(transformation(117, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_118 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä bäse wörd=\"";

		assert_eq!(String::from_utf8(transformation(118, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_119 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä BÄSE WÖRD='";

		assert_eq!(String::from_utf8(transformation(119, &base_word)).unwrap(), String::from(expected));
	}

	#[test]
	fn should_transform_120 () {
		let base_word = String::from("ä bäse wörd").bytes().collect::<Vec<_>>();

		let expected = " Ä bäse wörd='";

		assert_eq!(String::from_utf8(transformation(120, &base_word)).unwrap(), String::from(expected));
	}
}