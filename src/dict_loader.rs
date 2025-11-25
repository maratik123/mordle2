use crate::dict::{Dict, DictError};
use crate::{EMBEDDED_DICT, normalize};
use logging_timer::time;
use std::borrow::Cow;
use std::path::PathBuf;
use std::{fs, io};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, thiserror::Error)]
pub enum DictLoaderError {
    #[error(transparent)]
    DictError(#[from] DictError),
    #[error("Failed to read file {file:?}")]
    IoError {
        file: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[time("info")]
pub fn load(content: &'_ str) -> Result<Dict<'_>, DictLoaderError> {
    Ok(Dict::try_from_iter(content.unicode_words().map(|word| {
        let mut word = Vec::from_iter(word.graphemes(true));
        word.shrink_to_fit();
        word
    }))?)
}

#[time("info")]
pub fn load_dictionary_content(
    dict_path: Option<PathBuf>,
) -> Result<Cow<'static, str>, DictLoaderError> {
    Ok(match dict_path {
        Some(dict_path) => Cow::Owned(normalize(fs::read_to_string(&dict_path).map_err(|e| {
            DictLoaderError::IoError {
                file: dict_path,
                source: e,
            }
        })?)),
        None => Cow::Borrowed(EMBEDDED_DICT),
    })
}
