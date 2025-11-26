use crate::bitmap_iter::BitmapIterError;
use crate::check_result::CheckResult;
use crate::check_word::check_word;
use crate::dict::Dict;
use crate::hash_map_counter_inc;
use crate::stats::variance;
use log::info;
use logging_timer::time;
use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;
use roaring::RoaringBitmap;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(PartialEq, Clone)]
struct BestChoice<'s: 'd, 'd> {
    count: usize,
    variance: Option<f64>,
    words: Vec<&'d [&'s str]>,
}

impl Debug for BestChoice<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BestChoice")
            .field("count", &self.count)
            .field("variance", &self.variance)
            .field("words", &DebugWords { words: &self.words })
            .finish()
    }
}

struct DebugWords<'a, 'd, 's> {
    words: &'a [&'d [&'s str]],
}

impl Debug for DebugWords<'_, '_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.words.iter().map(|word| word.concat()))
            .finish()
    }
}

#[time("info")]
pub fn best_choice<'s, 'd>(
    rng: &mut ThreadRng,
    dict: &'d Dict<'s>,
    mask: &RoaringBitmap,
) -> Result<Option<&'d [&'s str]>, BitmapIterError> {
    info!("Will check {len} words", len = mask.len());

    let mut best_choice: Option<BestChoice> = None;

    let bitmap_iter = dict.try_bitmap_iter(mask)?;

    for suggestion in bitmap_iter.clone() {
        let mut all_results: HashMap<Vec<CheckResult>, usize> = HashMap::new();
        for possible_hidden in bitmap_iter.clone() {
            let check_result = check_word(possible_hidden, suggestion);
            hash_map_counter_inc!(all_results, check_result);
        }
        let variance_eval = || {
            variance(&Vec::from_iter(
                all_results.values().map(|&count| count as f64),
            ))
        };

        let new_count = all_results.len();

        let best_choice_create = |variance| {
            let result = BestChoice {
                count: new_count,
                variance,
                words: vec![suggestion],
            };
            info!(
                "Word: \"{word}\", based on {result:?}",
                word = suggestion.concat()
            );
            result
        };

        best_choice = Some(match best_choice.take() {
            None => best_choice_create(variance_eval()),
            Some(mut old_best_choice) => match old_best_choice.count.cmp(&new_count) {
                Ordering::Less => best_choice_create(variance_eval()),
                Ordering::Equal => {
                    let new_variance = variance_eval();
                    match old_best_choice.variance.partial_cmp(&new_variance) {
                        Some(Ordering::Greater) => best_choice_create(new_variance),
                        Some(Ordering::Equal) => {
                            old_best_choice.words.push(suggestion);
                            info!(
                                "Word: \"{word}\", based on {old_best_choice:?}",
                                word = suggestion.concat()
                            );
                            old_best_choice
                        }
                        _ => old_best_choice,
                    }
                }
                Ordering::Greater => old_best_choice,
            },
        });
    }
    info!("Best choice: {best_choice:?}");
    Ok(best_choice.and_then(|b| b.words.choose(rng).copied()))
}
