use crate::index::Index;
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
    pub fn to_bitmap<'i>(&self, index: &'i Index, grapheme: &str) -> Option<Cow<'i, RoaringBitmap>> {
        match self {
            GraphemeRule::NotInWord => Some(Cow::Owned({
                let mut result = index.full_bitmap();
                if let Some(has_grapheme) = index.get_by_grapheme_by_count_from(grapheme, 0) {
                    result -= has_grapheme;
                }
                result
            })),
            GraphemeRule::ExactPlace { grapheme_pos } => index.get_by_grapheme_and_pos(grapheme, *grapheme_pos).map(Cow::Borrowed),
            GraphemeRule::ExactCount { count } => index.get_by_grapheme_by_count(grapheme, *count).map(Cow::Borrowed),
            GraphemeRule::CountFrom { count_from } => index.get_by_grapheme_by_count_from(grapheme, *count_from).map(Cow::Borrowed),
        }
    }
}
