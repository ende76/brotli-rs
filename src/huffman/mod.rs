pub mod tree;

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
    assert_eq!(vec![false, true, true],
               bit_string_from_code_and_length(0b011, 3));
}

pub fn codes_from_lengths_and_symbols(lengths: &[usize], symbols: &[u16]) -> tree::Tree {
    let max_length = lengths.iter().fold(0, |acc, &len| {
        if len > acc {
            len
        } else {
            acc
        }
    });
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

    let mut codes = tree::Tree::with_max_depth(max_length);
    for i in 0..lengths.len() {
        let len = lengths[i];
        if len > 0 || max_length == 0 {
            codes.insert(&bit_string_from_code_and_length(next_code[len], len),
                         symbols[i]);
            next_code[len] += 1;
        }
    }

    codes
}

pub fn codes_from_lengths(lengths: &[usize]) -> tree::Tree {
    let symbols = (0..lengths.len() as u16).collect::<Vec<_>>();

    codes_from_lengths_and_symbols(lengths, &symbols)
}
