use crate::index::{Index, IndexEntry};
use roaring::RoaringBitmap;
use std::borrow::Cow;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum GraphemeRule {
    NotInWord,
    ExactPlace { grapheme_pos: usize },
    ExactCount { count: usize },
    CountFrom { count_from: usize },
}

impl GraphemeRule {
    pub fn to_bitmap<'i: 'e, 'e>(
        &self,
        index: &'i Index,
        index_entry: &'e IndexEntry,
    ) -> Cow<'e, RoaringBitmap> {
        match match self {
            GraphemeRule::NotInWord => {
                return Cow::Owned({
                    let mut result = index.full_bitmap();
                    if let Some(has_grapheme) = index_entry.counts_from(0) {
                        result -= has_grapheme;
                    }
                    result
                });
            }
            GraphemeRule::ExactPlace { grapheme_pos } => index_entry.get_by_pos(*grapheme_pos),
            GraphemeRule::ExactCount { count } => index_entry.counts(*count),
            GraphemeRule::CountFrom { count_from } => index_entry.counts_from(*count_from),
        } {
            Some(bitmap) => Cow::Borrowed(bitmap),
            None => Cow::Owned(RoaringBitmap::new()),
        }
    }
}
