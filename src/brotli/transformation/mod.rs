
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
		 22 => [Vec::from(base_word), vec![0x22, 0x10]].concat(),
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
		 50 => [Vec::from(base_word), vec![0x10, 0x09]].concat(),
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