use crate::dict::Dict;
use roaring::RoaringBitmap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::iter::FusedIterator;

#[derive(thiserror::Error, Eq, PartialEq, Debug)]
pub enum BitmapIterError {
    #[error("Bitmap (max value = {max_bitmap_value}) is not lesser than dict length = {dict_len}")]
    DictLengthMismatch {
        dict_len: usize,
        max_bitmap_value: u32,
    },
}

#[derive(Clone)]
pub struct BitmapIter<'s: 'd, 'd, 'b> {
    dict: &'d Dict<'s>,
    bitmap_iter: roaring::bitmap::Iter<'b>,
}

impl<'s, 'd, 'b> BitmapIter<'s, 'd, 'b> {
    pub fn try_from_dict(
        dict: &'d Dict<'s>,
        bitmap: &'b RoaringBitmap,
    ) -> Result<BitmapIter<'s, 'd, 'b>, BitmapIterError> {
        if let Some(max_bitmap_value) = bitmap.max()
            && max_bitmap_value as usize >= dict.len()
        {
            Err(BitmapIterError::DictLengthMismatch {
                dict_len: dict.len(),
                max_bitmap_value,
            })
        } else {
            Ok(BitmapIter {
                dict,
                bitmap_iter: bitmap.iter(),
            })
        }
    }

    pub fn advance_to_pos(&mut self, n: usize) {
        let big = n > u32::MAX as usize;
        self.bitmap_iter
            .advance_to(if big { u32::MAX } else { n as u32 });
        if big {
            self.bitmap_iter.next();
        }
    }

    pub fn advance_back_to_pos(&mut self, n: usize) {
        self.bitmap_iter
            .advance_back_to(u32::try_from(n).unwrap_or(u32::MAX));
    }
}

impl<'s, 'd> Iterator for BitmapIter<'s, 'd, '_> {
    type Item = &'d Vec<&'s str>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(&self.dict.words()[self.bitmap_iter.next()? as usize])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.bitmap_iter.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.bitmap_iter.count()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.bitmap_iter
            .nth(n)
            .map(|word_id| &self.dict.words()[word_id as usize])
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.bitmap_iter.fold(init, |acc, word_id| {
            f(acc, &self.dict.words()[word_id as usize])
        })
    }
}

impl DoubleEndedIterator for BitmapIter<'_, '_, '_> {
    fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
        Some(&self.dict.words()[self.bitmap_iter.next_back()? as usize])
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.bitmap_iter
            .nth_back(n)
            .map(|word_id| &self.dict.words()[word_id as usize])
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.bitmap_iter.rfold(init, |acc, word_id| {
            f(acc, &self.dict.words()[word_id as usize])
        })
    }
}

#[cfg(target_pointer_width = "64")]
impl ExactSizeIterator for BitmapIter<'_, '_, '_> {}

impl FusedIterator for BitmapIter<'_, '_, '_> {}

impl Debug for BitmapIter<'_, '_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("BitmapIter")
            .field("dict", &self.dict)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::assert_equal;
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn test_bitmap_iter() {
        let dict =
            Dict::try_from_iter(["world", "hello"].map(|w| Vec::from_iter(w.graphemes(true))))
                .unwrap();
        let bitmap = RoaringBitmap::from([0, 1]);
        assert_equal(
            BitmapIter::try_from_dict(&dict, &bitmap).unwrap(),
            [
                &vec!["h", "e", "l", "l", "o"],
                &vec!["w", "o", "r", "l", "d"],
            ],
        );
    }

    #[test]
    fn test_empty_bitmap_iter() {
        let dict =
            Dict::try_from_iter(["world", "hello"].map(|w| Vec::from_iter(w.graphemes(true))))
                .unwrap();
        let bitmap = RoaringBitmap::new();
        assert_equal::<_, [&Vec<&str>; _]>(
            BitmapIter::try_from_dict(&dict, &bitmap).unwrap().next(),
            [],
        );
    }

    #[test]
    fn test_invalid_bitmap() {
        let dict =
            Dict::try_from_iter(["world", "hello"].map(|w| Vec::from_iter(w.graphemes(true))))
                .unwrap();
        let bitmap = RoaringBitmap::from([2]);
        assert_eq!(
            BitmapIter::try_from_dict(&dict, &bitmap).unwrap_err(),
            BitmapIterError::DictLengthMismatch {
                dict_len: 2,
                max_bitmap_value: 2
            }
        );
    }

    #[test]
    fn test_bitmap_on_dict_with_duplicates() {
        let dict = Dict::try_from_iter(
            ["world", "hello", "hello", "world"].map(|w| Vec::from_iter(w.graphemes(true))),
        )
        .unwrap();
        let bitmap = RoaringBitmap::from([0, 1]);
        assert_equal(
            BitmapIter::try_from_dict(&dict, &bitmap).unwrap(),
            [
                &vec!["h", "e", "l", "l", "o"],
                &vec!["w", "o", "r", "l", "d"],
            ],
        );
    }
}
