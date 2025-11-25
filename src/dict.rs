use crate::best_choice::best_choice;
use crate::bitmap_iter::{BitmapIter, BitmapIterError};
use logging_timer::time;
use rand::prelude::ThreadRng;
use roaring::RoaringBitmap;
use std::fmt::Debug;
use std::iter;

#[derive(Default, Debug, Eq, PartialEq)]
pub struct Dict<'s> {
    word_len: usize,
    words: Vec<Vec<&'s str>>,
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum DictError {
    #[error("Word length mismatch: expected {expected_word_len}, got {actual_word_len} for {word}")]
    WordLenMismatch {
        expected_word_len: usize,
        actual_word_len: usize,
        word: String,
    },
}

impl<'s> Dict<'s> {
    #[time("debug", "Dict::{}")]
    pub fn try_from_iter(iter: impl IntoIterator<Item = Vec<&'s str>>) -> Result<Self, DictError> {
        let mut iter = iter.into_iter();

        Ok(if let Some(first) = iter.next() {
            let word_len = first.len();
            let mut words: Vec<_> =
                Result::from_iter(iter::once(first).map(Ok).chain(iter.map(|s_graphemes| {
                    if s_graphemes.len() == word_len {
                        Ok(s_graphemes)
                    } else {
                        Err(DictError::WordLenMismatch {
                            expected_word_len: word_len,
                            actual_word_len: s_graphemes.len(),
                            word: s_graphemes.concat(),
                        })
                    }
                })))?;

            words.sort_unstable();
            words.dedup();
            words.shrink_to_fit();

            Self { word_len, words }
        } else {
            Default::default()
        })
    }

    pub fn word_len(&self) -> usize {
        self.word_len
    }

    pub fn words(&self) -> &Vec<Vec<&'s str>> {
        &self.words
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn word_index(&self, word: &Vec<&str>) -> Option<usize> {
        self.words.binary_search(word).ok()
    }

    pub fn try_bitmap_iter<'b, 'd>(
        &'d self,
        bitmap: &'b RoaringBitmap,
    ) -> Result<BitmapIter<'s, 'd, 'b>, BitmapIterError> {
        BitmapIter::try_from_dict(self, bitmap)
    }

    pub fn best_choice<'d>(
        &'d self,
        rng: &mut ThreadRng,
        mask: &RoaringBitmap,
    ) -> Result<Option<&'d Vec<&'s str>>, BitmapIterError> {
        best_choice(rng, self, mask)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn test_collect_from_iter() {
        let dict = Dict::try_from_iter(
            ["world", "hello", "hello", "world"].map(|w| Vec::from_iter(w.graphemes(true))),
        )
        .unwrap();
        assert_eq!(
            dict,
            Dict {
                word_len: 5,
                words: vec![vec!["h", "e", "l", "l", "o"], vec!["w", "o", "r", "l", "d"],]
            }
        );
    }

    #[test]
    fn test_collect_from_iter_zalgo() {
        let dict = Dict::try_from_iter(
            ["h̛̞́͜e͉͛l̐ͅlȯ͉͔͖͝͞", "w̲̃o͖̜̅̍r̗̹̐̔l̝͊ď̘", "щ̲͋ё̢̭̍̅͜͝т̧̻̟͂̅̄к̣̘͉̀̇͝а̡̧̗̂̒̂", "щ̢̡̪̲͗͒̓̒ѐ̧̱̩̪̄̾̂̅͟т̰̇к̰̘̊̒а̱̰̟͔̊̆̕̚"].map(|w| Vec::from_iter(w.graphemes(true))),
        )
        .unwrap();
        assert_eq!(dict.word_len(), 5);
        assert_eq!(dict.len(), 4);
    }

    #[test]
    fn empty_dict() {
        let dict = Dict::try_from_iter(iter::empty()).unwrap();
        assert_eq!(dict, Default::default());
    }

    #[test]
    fn test_word_len_mismatch() {
        assert_eq!(
            Dict::try_from_iter(["hello", "word"].map(|w| Vec::from_iter(w.graphemes(true)))),
            Err(DictError::WordLenMismatch {
                expected_word_len: 5,
                actual_word_len: 4,
                word: "word".to_string(),
            })
        );
    }
}
