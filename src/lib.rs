pub mod best_choice;
pub mod bitmap_iter;
pub mod check_result;
pub mod check_word;
pub mod dict;
pub mod dict_loader;
pub mod grapheme_rule;
pub mod index;
pub mod rules_applier;
pub mod stats;

use unicode_normalization::{IsNormalized, UnicodeNormalization, is_nfc_quick};

pub const EMBEDDED_DICT: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/mordle-dict.txt"));

pub fn normalize(s: String) -> String {
    if is_nfc_quick(s.chars()) == IsNormalized::Yes {
        s
    } else {
        s.nfc().collect()
    }
}

#[macro_export]
macro_rules! hash_map_counter_inc {
    ($map:expr, $key:expr) => {
        use std::collections::hash_map;
        match $map.entry($key) {
            hash_map::Entry::Occupied(entry) => *entry.into_mut() += 1,
            hash_map::Entry::Vacant(entry) => {
                entry.insert(Default::default());
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_normalization::is_nfc;

    #[test]
    fn embedded_dict_should_be_normalized() {
        assert!(is_nfc(EMBEDDED_DICT));
    }
}
