use std::collections::HashMap;

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
    let mut result = prefix.to_owned();
    if !map.contains_key(prefix) {
        return result;
    }

    let mut suffix: u32 = 0;
    while map.contains_key(&format!("{}_{}", prefix, suffix)) {
        suffix += 1;
    }

    return format!("{}_{}", prefix, suffix);
}
