use crate::index::Index;
use roaring::RoaringBitmap;
use std::borrow::Cow;
use std::ops::BitXorAssign;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum GraphemeRule {
    NotInWord,
    ExactPlace { grapheme_pos: usize },
    ExactCount { count: usize },
    CountFrom { count_from: usize },
}

impl GraphemeRule {
    pub fn to_bitmap<'i>(&self, index: &'i Index, grapheme: &str) -> Cow<'i, RoaringBitmap> {
        match self {
            GraphemeRule::NotInWord => Cow::Owned({
                let mut result = index.full_bitmap();
                if let Some(has_grapheme) = index.get_by_grapheme_by_count_from(grapheme, 0) {
                    result -= has_grapheme;
                }
                result
            }),
            GraphemeRule::ExactPlace { grapheme_pos } => {}
            GraphemeRule::ExactCount { count } => {}
            GraphemeRule::CountFrom { count_from } => {}
        }
    }
}
