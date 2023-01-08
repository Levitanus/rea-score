use thiserror;

use self::chord_notations::ChordNotations;
use self::note_notations::NoteNotations;

pub mod chord_notations;
pub mod message;
pub mod note_notations;

#[derive(Debug, thiserror::Error)]
pub enum NotationError {
    #[error("No tokens found! Original string: `{0}`")]
    NoTokens(String),
    #[error("Not enough tokens found! Expected: {0}, found: {1}")]
    NotEnoughTokens(u16, u16),
    #[error("Too many tokens! Expected: {0}")]
    TooManyTokens(u16),
    #[error("Unexpected Token: {0}")]
    UnexpectedToken(String),
    #[error(
        "Unexpected Notation: Can not apply notation \
        to object: {notation}, {object}"
    )]
    UnexpectedNotation { notation: String, object: String },
}
pub type NotationResult<T> = Result<T, NotationError>;

// base notation strings should look like:
// "NOTE 0 60 Voice 1 text ReaScore|voice:2|dynamics:\mf"
// which should be parsed as
// `Vec[
//    NotationType::Note(1, 60, NoteNotation::Voice(2)),
//    NotationType::Chord(1, 60, ChordNotation::Dynamics("\mf")),
// ]`
const SECTION: &'static str = "ReaScore";
const NOTATION_DELIMITER: &'static str = "|";
const TOKENS_DELIMITER: &'static str = ":";

/// Try to get token from vec, and return [NotationError] at fail.
fn get_token<'a>(
    v: &'a Vec<&str>,
    idx: usize,
) -> Result<&'a str, NotationError> {
    let s = v.get(idx).ok_or(NotationError::NotEnoughTokens(2, 1))?;
    Ok(*s)
}

pub trait NotationRender {
    fn render(&self, pitch_string: impl Into<String>) -> String;
}
/// Way to decide which note should carry the notation on note-split.
///
/// if corresponding function returns true → it will be kept at the head, at
/// tail or in both. By default, only head specified, then tail is inverted.
/// But both functions can be implemented.
pub trait NotationSplitPosition {
    fn is_head(&self) -> bool;
    fn is_tail(&self) -> bool {
        !self.is_head()
    }
}

/// Handles parsing and representation of raw notation
#[derive(Debug, PartialEq, Clone)]
pub enum NotationType {
    /// channel, note
    Note(NoteNotations),
    /// channel, note: note still presents, as chord
    /// events will be de-duplicated by mapping to events.
    Chord(ChordNotations),
    Event,
}
impl ToString for NotationType {
    fn to_string(&self) -> String {
        match self {
            Self::Note(n) => n.to_string(),
            Self::Chord(c) => c.to_string(),
            Self::Event => unimplemented!(),
        }
    }
}

#[cfg(test)]
#[test]
fn test_notation_type() {
    use self::note_notations::NoteHead;

    let a = NotationType::Note(NoteNotations::NoteHead(NoteHead::Cross));
    let b =
        NotationType::Note(NoteNotations::NoteHead(NoteHead::HarmonicMixed));
    let c = NotationType::Chord(ChordNotations::Dynamics("\\mf".into()));
    let d = NotationType::Chord(ChordNotations::Dynamics("\\f".into()));
    assert_eq!(a, a);
    assert_eq!(b, b);
    assert_eq!(c, c);
    assert_eq!(d, d);

    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(c, d);
}

/// parse reascore single notation string
///
/// If exact amount of tokens is specifeid — NotationError will be returned on
/// wrond tokens amount
pub fn reascore_tokens(
    reascore_notation_string: &str,
    expected_amount: impl Into<Option<u16>>,
) -> NotationResult<Vec<&str>> {
    let mut split = reascore_notation_string.split(TOKENS_DELIMITER);
    if reascore_notation_string.is_empty() {
        return Err(NotationError::NoTokens(
            reascore_notation_string.to_string(),
        )
        .into());
    }
    match expected_amount.into() {
        None => Ok(split.collect()),
        Some(am) => {
            let mut v = Vec::new();
            for idx in 0..am {
                v.push(
                    split
                        .next()
                        .ok_or(NotationError::NotEnoughTokens(am, idx))?,
                )
            }
            split
                .next()
                .is_none()
                .then(|| v)
                .ok_or(NotationError::TooManyTokens(am).into())
        }
    }
}
