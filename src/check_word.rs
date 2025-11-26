use crate::check_result::CheckResult;
use crate::hash_map_counter_inc;
use std::collections::{HashMap, hash_map};
use std::iter;

const WORD_LEN_ENUMERATION_THRESHOLD: usize = 20;
const EXPECTED_ALPHABET_SIZE: usize = 64;

pub fn check_word(hidden: &[&str], suggested: &[&str]) -> Vec<CheckResult> {
    if suggested.len() <= WORD_LEN_ENUMERATION_THRESHOLD {
        check_word_enumerating(hidden, suggested)
    } else {
        check_word_hashing(hidden, suggested)
    }
}

fn check_word_enumerating(hidden: &[&str], suggested: &[&str]) -> Vec<CheckResult> {
    let mut not_found_hidden = Vec::with_capacity(suggested.len());
    let mut result = vec![CheckResult::NotInWord; suggested.len()];
    for ((&grapheme_s, &grapheme_h), r) in iter::zip(suggested, hidden).zip(result.iter_mut()).rev()
    {
        if grapheme_s == grapheme_h {
            *r = CheckResult::ExactPlace;
        } else {
            not_found_hidden.push(grapheme_h);
        }
    }
    for (&grapheme_s, r) in iter::zip(suggested, result.iter_mut()) {
        if r == &CheckResult::NotInWord
            && let Some(i) = not_found_hidden
                .iter()
                .rposition(|&grapheme_h| grapheme_s == grapheme_h)
        {
            not_found_hidden.swap_remove(i);
            *r = CheckResult::InWord;
        }
    }
    result
}

fn check_word_hashing(hidden: &[&str], suggested: &[&str]) -> Vec<CheckResult> {
    let mut not_found_hidden =
        HashMap::<&str, usize>::with_capacity(suggested.len().min(EXPECTED_ALPHABET_SIZE));
    let mut result = vec![CheckResult::NotInWord; suggested.len()];
    for ((&grapheme_s, &grapheme_h), r) in iter::zip(suggested, hidden).zip(result.iter_mut()).rev()
    {
        if grapheme_s == grapheme_h {
            *r = CheckResult::ExactPlace;
        } else {
            hash_map_counter_inc!(not_found_hidden, grapheme_h);
        }
    }
    for (&grapheme_s, r) in iter::zip(suggested, result.iter_mut()) {
        if r == &CheckResult::NotInWord
            && let hash_map::Entry::Occupied(entry) = not_found_hidden.entry(grapheme_s)
        {
            match entry.get().checked_sub(1) {
                Some(v) => *entry.into_mut() = v,
                None => {
                    entry.remove();
                }
            }
            *r = CheckResult::InWord;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dict::Dict;
    use unicode_segmentation::UnicodeSegmentation;

    fn assert_check_word(dict: Dict, hidden: usize, suggested: usize, expected: &[CheckResult]) {
        let suggested = &dict.words()[suggested];
        let hidden = &dict.words()[hidden];

        assert_eq!(check_word(hidden, suggested), expected);
        assert_eq!(check_word_enumerating(hidden, suggested), expected);
        assert_eq!(check_word_hashing(hidden, suggested), expected);
    }

    #[test]
    fn test_check_word_exact() {
        let dict = Dict::try_from_iter(
            ["hello", "world"].map(|w| Vec::from_iter(w.graphemes(true)).into_boxed_slice()),
        )
        .unwrap();
        let expected = vec![CheckResult::ExactPlace; dict.word_len()];
        assert_check_word(dict, 0, 0, &expected);
    }

    #[test]
    fn test_check_word_partial0() {
        let dict = Dict::try_from_iter(
            ["hello", "world"].map(|w| Vec::from_iter(w.graphemes(true)).into_boxed_slice()),
        )
        .unwrap();
        assert_check_word(
            dict,
            1,
            0,
            &[
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::ExactPlace,
                CheckResult::InWord,
            ],
        );
    }

    #[test]
    fn test_check_word_partial1() {
        let dict = Dict::try_from_iter(
            ["hello", "world"].map(|w| Vec::from_iter(w.graphemes(true)).into_boxed_slice()),
        )
        .unwrap();
        assert_check_word(
            dict,
            0,
            1,
            &[
                CheckResult::NotInWord,
                CheckResult::InWord,
                CheckResult::NotInWord,
                CheckResult::ExactPlace,
                CheckResult::NotInWord,
            ],
        );
    }

    #[test]
    fn test_check_word_long_partial0() {
        let dict = Dict::try_from_iter(
            ["abcdefghjjk", "jjjnopqrstk"]
                .map(|w| Vec::from_iter(w.graphemes(true)).into_boxed_slice()),
        )
        .unwrap();
        assert_check_word(
            dict,
            0,
            1,
            &[
                CheckResult::InWord,
                CheckResult::InWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::ExactPlace,
            ],
        );
    }

    #[test]
    fn test_check_word_long_partial1() {
        let dict = Dict::try_from_iter(
            ["abcdefghjjk", "jjjnopqrstk"]
                .map(|w| Vec::from_iter(w.graphemes(true)).into_boxed_slice()),
        )
        .unwrap();
        assert_check_word(
            dict,
            1,
            0,
            &[
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::InWord,
                CheckResult::InWord,
                CheckResult::ExactPlace,
            ],
        );
    }
}
