use itertools::{Either, Itertools};
use std::fmt::{Display, Formatter};
use std::{fmt, str};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum CheckResult {
    NotInWord,
    ExactPlace,
    InWord,
}

impl Display for CheckResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        char::from(*self).fmt(f)
    }
}

impl From<CheckResult> for char {
    fn from(result: CheckResult) -> Self {
        match result {
            CheckResult::NotInWord => '-',
            CheckResult::ExactPlace => '+',
            CheckResult::InWord => '?',
        }
    }
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
#[error("Invalid check result: {0}")]
pub struct CheckResultError(String);

impl TryFrom<&str> for CheckResult {
    type Error = CheckResultError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "-" => CheckResult::NotInWord,
            "+" => CheckResult::ExactPlace,
            "?" => CheckResult::InWord,
            value => return Err(CheckResultError(value.to_string())),
        })
    }
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ParseCheckResultError {
    #[error(transparent)]
    CheckResultError(#[from] CheckResultError),
    #[error("Invalid check result string length: {0}")]
    InvalidStringLength(usize),
}
pub type ParseCheckResult<'a> =
    Result<Either<Vec<CheckResult>, Vec<(CheckResult, &'a str)>>, ParseCheckResultError>;

fn parse_check_result_inner(
    graphemes: Vec<&str>,
) -> Result<Vec<CheckResult>, ParseCheckResultError> {
    Ok(graphemes
        .into_iter()
        .map(CheckResult::try_from)
        .try_collect()?)
}

fn parse_check_result_inner_with_word(
    graphemes: Vec<&'_ str>,
) -> Result<Vec<(CheckResult, &'_ str)>, ParseCheckResultError> {
    let (chunks, _) = graphemes.as_chunks::<2>();
    Ok(chunks
        .iter()
        .map(|&[grapheme, check_result]| {
            CheckResult::try_from(check_result).map(|check_result| (check_result, grapheme))
        })
        .try_collect()?)
}

pub fn parse_check_result(s: &'_ str, word_len: usize) -> ParseCheckResult<'_> {
    let graphemes = Vec::from_iter(s.graphemes(true));
    Ok(if graphemes.len() == word_len {
        Either::Left(parse_check_result_inner(graphemes)?)
    } else if graphemes.len() == word_len * 2 {
        Either::Right(parse_check_result_inner_with_word(graphemes)?)
    } else {
        return Err(ParseCheckResultError::InvalidStringLength(graphemes.len()));
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_str() {
        assert_eq!(CheckResult::try_from("-"), Ok(CheckResult::NotInWord));
        assert_eq!(CheckResult::try_from("+"), Ok(CheckResult::ExactPlace));
        assert_eq!(CheckResult::try_from("?"), Ok(CheckResult::InWord));
        assert_eq!(
            CheckResult::try_from("a"),
            Err(CheckResultError("a".to_string()))
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(CheckResult::NotInWord.to_string(), "-");
        assert_eq!(CheckResult::ExactPlace.to_string(), "+");
        assert_eq!(CheckResult::InWord.to_string(), "?");
    }

    #[test]
    fn test_parse_result() {
        assert_eq!(
            parse_check_result("--++?", 5).unwrap(),
            Either::Left(vec![
                CheckResult::NotInWord,
                CheckResult::NotInWord,
                CheckResult::ExactPlace,
                CheckResult::ExactPlace,
                CheckResult::InWord
            ])
        );
    }

    #[test]
    fn test_parse_result_with_word() {
        assert_eq!(
            parse_check_result("a-b-c+d+e?", 5).unwrap(),
            Either::Right(vec![
                (CheckResult::NotInWord, "a"),
                (CheckResult::NotInWord, "b"),
                (CheckResult::ExactPlace, "c"),
                (CheckResult::ExactPlace, "d"),
                (CheckResult::InWord, "e")
            ])
        );
    }

    #[test]
    fn test_failed_parse_result_invalid_char() {
        assert_eq!(
            parse_check_result("a-b-c+d+e.", 5).unwrap_err(),
            ParseCheckResultError::CheckResultError(CheckResultError(".".to_string()))
        );
    }

    #[test]
    fn test_failed_parse_result_invalid_len_with_word() {
        assert_eq!(
            parse_check_result("a-b-c+d+e", 5).unwrap_err(),
            ParseCheckResultError::InvalidStringLength(9)
        );
    }

    #[test]
    fn test_failed_parse_result_invalid_len() {
        assert_eq!(
            parse_check_result("--++??", 5).unwrap_err(),
            ParseCheckResultError::InvalidStringLength(6)
        );
    }

    #[test]
    fn test_failed_parse_result_invalid_str() {
        assert_eq!(
            parse_check_result("s-a+̸t+", 3).unwrap_err(),
            ParseCheckResultError::CheckResultError(CheckResultError("+̸".to_string()))
        );
    }
}
