pub fn codes_from_lengths(ref lengths: Vec<usize>) -> Vec<usize> {
	let max_length = lengths.iter().fold(0, |acc, &len| if len > acc { len } else { acc });
	let mut bl_count = vec![0; max_length + 1];
	for &len in lengths {
		bl_count[len] += 1;
	}

	let mut code = 0;
	let mut next_code = vec![0; max_length + 1];
	for bits in 1..max_length + 1 {
		code = (code + bl_count[bits - 1]) << 1;
		next_code[bits] = code;
	}

	let mut codes = vec![0; lengths.len()];
	for i in 0..codes.len() {
		let len = lengths[i];
		if len > 0 {
			codes[i] = next_code[len];
			next_code[len] += 1;
		}
	}

	codes
}

mod tests {

	#[test]
	fn should_encode_single_length() {
		use super::codes_from_lengths;

		let lengths = vec![1];
		let codes = codes_from_lengths(lengths);

		assert_eq!(vec![0], codes);
	}

	#[test]
	fn should_encode_lengths() {
		use super::codes_from_lengths;

		let lengths = vec![3, 3, 3, 3, 3, 2, 4, 4];
		let codes = codes_from_lengths(lengths);

		assert_eq!(vec![0b010, 0b011, 0b100, 0b101, 0b110, 0b00, 0b1110, 0b1111], codes);
	}
}