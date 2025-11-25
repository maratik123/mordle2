use crate::dict::Dict;
use crate::hash_map_counter_inc;
use logging_timer::time;
use roaring::RoaringBitmap;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::num::TryFromIntError;

#[derive(Debug, PartialEq)]
pub struct Index<'d, 's: 'd> {
    bitmaps: HashMap<&'s str, IndexEntry<'d, 's>>,
    dict: &'d Dict<'s>,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum IndexError {
    #[error("Dict too large: {0} words")]
    DictTooLarge(usize, #[source] TryFromIntError),
}

impl<'d, 's: 'd> Index<'d, 's> {
    #[time("info", "Index::{}")]
    pub fn from_dict(dict: &'d Dict<'s>) -> Result<Self, IndexError> {
        if let Err(err) = u32::try_from(dict.len()) {
            return Err(IndexError::DictTooLarge(dict.len(), err));
        }

        let bitmaps = Self::create_bitmaps(dict);
        Ok(Self::finalize_entries(dict, bitmaps))
    }

    #[time("debug", "Index::{}")]
    fn create_bitmaps(dict: &'d Dict<'s>) -> HashMap<&'s str, IndexEntry<'d, 's>> {
        let mut bitmaps: HashMap<&'s str, IndexEntry<'d, 's>> = dict
            .words()
            .iter()
            .enumerate()
            .map(|(word_pos, word)| (word_pos as u32, word))
            .flat_map(|(word_pos, word)| {
                word.iter()
                    .enumerate()
                    .map(move |(grapheme_pos, grapheme)| (word_pos, grapheme_pos, grapheme))
            })
            .fold(
                Default::default(),
                |mut acc, (word_pos, grapheme_pos, grapheme)| {
                    acc.entry(grapheme)
                        .or_insert_with(|| IndexEntry::new(dict.word_len()))
                        .by_pos[grapheme_pos]
                        .insert(word_pos);
                    acc
                },
            );
        bitmaps.shrink_to_fit();
        bitmaps
    }

    #[time("debug", "Index::{}")]
    fn finalize_entries(
        dict: &'d Dict<'s>,
        mut bitmaps: HashMap<&'s str, IndexEntry<'d, 's>>,
    ) -> Self {
        for index_entry in bitmaps.values_mut() {
            index_entry.fill_counts(dict);
            index_entry.fill_counts_from_and_finalize();
        }

        Self { dict, bitmaps }
    }
}

impl<'d, 's> Index<'d, 's> {
    pub fn full_bitmap(&self) -> RoaringBitmap {
        let mut result = RoaringBitmap::new();
        result.insert_range(..self.dict.len() as u32);
        result
    }

    pub fn empty_bitmap() -> RoaringBitmap {
        RoaringBitmap::new()
    }

    pub fn get_by_grapheme_by_count(&self, grapheme: &str, count: usize) -> Option<&RoaringBitmap> {
        self.get_entry_by_grapheme(grapheme)
            .and_then(|entry| entry.counts(count))
    }

    pub fn get_by_grapheme_by_count_from(
        &self,
        grapheme: &str,
        count_from: usize,
    ) -> Option<&RoaringBitmap> {
        self.get_entry_by_grapheme(grapheme)
            .and_then(|entry| entry.counts_from(count_from))
    }

    pub fn get_by_grapheme_and_pos(
        &self,
        grapheme: &str,
        grapheme_pos: usize,
    ) -> Option<&RoaringBitmap> {
        self.get_entry_by_grapheme(grapheme)
            .and_then(|entry| entry.get_by_pos(grapheme_pos))
    }

    pub fn get_entry_by_grapheme(&self, grapheme: &str) -> Option<&IndexEntry<'d, 's>> {
        self.bitmaps.get(grapheme)
    }

    pub fn dict(&self) -> &'d Dict<'s> {
        self.dict
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct IndexEntry<'d, 's> {
    counts: Vec<RoaringBitmap>,
    counts_from: Vec<RoaringBitmap>,
    by_pos: Vec<RoaringBitmap>,
    phantom: PhantomData<&'d Dict<'s>>,
}

impl<'d, 's> IndexEntry<'d, 's> {
    pub fn counts(&self, count: usize) -> Option<&RoaringBitmap> {
        self.counts.get(count)
    }

    pub fn counts_from(&self, count_from: usize) -> Option<&RoaringBitmap> {
        self.counts_from.get(count_from)
    }

    pub fn get_by_pos(&self, grapheme_pos: usize) -> Option<&RoaringBitmap> {
        self.by_pos.get(grapheme_pos)
    }

    fn new(word_len: usize) -> Self {
        Self {
            counts: vec![RoaringBitmap::new(); word_len],
            counts_from: vec![RoaringBitmap::new(); word_len],
            by_pos: vec![RoaringBitmap::new(); word_len],
            phantom: Default::default(),
        }
    }

    #[time("debug", "IndexEntry::{}")]
    fn fill_counts(&mut self, dict: &Dict) {
        let mut counts_by_word_pos: HashMap<u32, usize> = HashMap::with_capacity(dict.len());
        for bitmap in self.by_pos.iter_mut() {
            bitmap.optimize();
            for word_pos in bitmap.iter() {
                hash_map_counter_inc!(counts_by_word_pos, word_pos);
            }
        }
        for (word_pos, count) in counts_by_word_pos.into_iter() {
            self.counts[count].insert(word_pos);
        }
    }

    #[time("debug", "IndexEntry::{}")]
    fn fill_counts_from_and_finalize(&mut self) {
        for (grapheme_pos, counts) in self.counts.iter_mut().enumerate().rev() {
            counts.optimize();
            match self.counts_from.split_at_mut(grapheme_pos + 1) {
                ([.., to], []) => *to = counts.clone(),
                ([.., to], [from, ..]) => {
                    *to = &*counts | &*from;
                    to.optimize();
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn test_empty_dict() {
        let dict = Dict::try_from_iter([]).unwrap();
        let index = Index::from_dict(&dict);
        assert_eq!(
            index,
            Ok(Index {
                bitmaps: Default::default(),
                dict: &dict
            })
        );
    }

    #[test]
    fn test_index_from_dict() {
        let dict =
            Dict::try_from_iter(["hello", "world"].map(|w| Vec::from_iter(w.graphemes(true))))
                .unwrap();
        let index = Index::from_dict(&dict);
        assert_eq!(
            index,
            Ok(Index {
                dict: &dict,
                bitmaps: HashMap::from([
                    (
                        "h",
                        IndexEntry {
                            counts: vec![
                                RoaringBitmap::from([0]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            counts_from: vec![
                                RoaringBitmap::from([0]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            by_pos: vec![
                                RoaringBitmap::from([0]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            phantom: Default::default(),
                        }
                    ),
                    (
                        "e",
                        IndexEntry {
                            counts: vec![
                                RoaringBitmap::from([0]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            counts_from: vec![
                                RoaringBitmap::from([0]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            by_pos: vec![
                                RoaringBitmap::new(),
                                RoaringBitmap::from([0]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            phantom: Default::default(),
                        }
                    ),
                    (
                        "l",
                        IndexEntry {
                            counts: vec![
                                RoaringBitmap::from([1]),
                                RoaringBitmap::from([0]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            counts_from: vec![
                                RoaringBitmap::from([0, 1]),
                                RoaringBitmap::from([0]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            by_pos: vec![
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::from([0]),
                                RoaringBitmap::from([0, 1]),
                                RoaringBitmap::new(),
                            ],
                            phantom: Default::default(),
                        }
                    ),
                    (
                        "o",
                        IndexEntry {
                            counts: vec![
                                RoaringBitmap::from([0, 1]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            counts_from: vec![
                                RoaringBitmap::from([0, 1]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            by_pos: vec![
                                RoaringBitmap::new(),
                                RoaringBitmap::from([1]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::from([0]),
                            ],
                            phantom: Default::default(),
                        }
                    ),
                    (
                        "w",
                        IndexEntry {
                            counts: vec![
                                RoaringBitmap::from([1]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            counts_from: vec![
                                RoaringBitmap::from([1]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            by_pos: vec![
                                RoaringBitmap::from([1]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            phantom: Default::default(),
                        }
                    ),
                    (
                        "r",
                        IndexEntry {
                            counts: vec![
                                RoaringBitmap::from([1]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            counts_from: vec![
                                RoaringBitmap::from([1]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            by_pos: vec![
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::from([1]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            phantom: Default::default(),
                        }
                    ),
                    (
                        "d",
                        IndexEntry {
                            counts: vec![
                                RoaringBitmap::from([1]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            counts_from: vec![
                                RoaringBitmap::from([1]),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                            ],
                            by_pos: vec![
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::new(),
                                RoaringBitmap::from([1]),
                            ],
                            phantom: Default::default(),
                        }
                    ),
                ])
            })
        );
    }
}
