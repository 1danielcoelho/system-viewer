use std::collections::HashMap;
use std::num::ParseIntError;

pub fn remove_numbered_suffix(s: &str) -> &str {
    let mut split_index: usize = s.len();
    for (i, c) in s.char_indices().rev() {
        if !c.is_digit(10) && c != '_' {
            split_index = i + 1;
            break;
        }
    }

    return &s[0..split_index];
}

pub fn get_unique_name<T>(prefix: &str, map: &HashMap<String, T>) -> String {
    let result = prefix.to_owned();
    if !map.contains_key(prefix) {
        return result;
    }

    let mut suffix: u32 = 0;
    while map.contains_key(&format!("{}_{}", prefix, suffix)) {
        suffix += 1;
    }

    return format!("{}_{}", prefix, suffix);
}

// Source: https://stackoverflow.com/a/52992629/2434460
pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}
