fn bit_string_from_code_and_length(code: usize, len: usize) -> Vec<bool> {
	let mut bits = vec![false; len];

	for i in 0..len {
		bits[len - i - 1] = (code >> i) & 1 == 1;
	}

	bits
}

#[test]
fn should_honor_leading_zeroes() {
	assert_eq!(vec![false, false], bit_string_from_code_and_length(0b00, 2));
	assert_eq!(vec![false, true, true], bit_string_from_code_and_length(0b011, 3));
}


pub fn codes_from_lengths(ref lengths: Vec<usize>) -> Vec<Vec<bool>> {
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

	let mut codes = vec![vec![]; lengths.len()];
	for i in 0..codes.len() {
		let len = lengths[i];
		if len > 0 {
			codes[i] = bit_string_from_code_and_length(next_code[len], len);
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

		assert_eq!(vec![vec![false]], codes);
	}

	#[test]
	fn should_encode_lengths() {
		use super::codes_from_lengths;

		let lengths = vec![3, 3, 3, 3, 3, 2, 4, 4];
		let codes = codes_from_lengths(lengths);

		assert_eq!(vec![vec![false, true, false], vec![false, true, true], vec![true, false, false], vec![true, false, true], vec![true, true, false], vec![false, false], vec![true, true, true, false], vec![true, true, true, true]], codes);
	}
}