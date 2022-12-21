use thiserror;

use rea_rs::midi::Notation as MNotation;
use rea_rs::NotationMessage;

use self::chord_notations::ChordNotations;
use self::note_notations::NoteNotations;

pub mod chord_notations;
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
}
pub type NotationResult<T> = Result<T, NotationError>;

// base notation strings should look like:
// "NOTE 0 60 Voice 1 text ReaScore|voice:2|dynamics:\mf"
// which should be parsed as
// `Vec[
//    NotationType::Note(1, 60, NoteNotation::Voice(2)),
//    NotationType::Chord(1, 60, ChordNotation::Dynamics("\mf")),
//]`
const SECTION: &'static str = "ReaScore";
const NOTATION_DELIMITER: &'static str = "|";
const TOKENS_DELIMITER: &'static str = ":";

/// Get reascore notations, if any.
pub fn notations_from_midi(msg: NotationMessage) -> Option<Vec<NotationType>> {
    match msg.notation() {
        MNotation::Note {
            channel: _,
            note: _,
            tokens,
        } => {
            let tokens = reascore_notation_string(tokens)?;
            let notes = tokens
                .iter()
                .filter_map(|tk| Some(NotationType::Note(tk.parse().ok()?)));
            let chords = tokens
                .iter()
                .filter_map(|tk| Some(NotationType::Chord(tk.parse().ok()?)));
            Some(notes.chain(chords).collect())
        }
        MNotation::Track(_) => todo!(),
        MNotation::Unknown(_) => todo!(),
    }
}

/// Get reascore tokens, if any.
fn reascore_notation_string(tokens: Vec<String>) -> Option<Vec<String>> {
    let v: Vec<String> = tokens
        .into_iter()
        .filter(|tk| tk.starts_with(SECTION))
        .collect();
    match v.len() {
        0 => None,
        1 => Some(
            v[0].split(NOTATION_DELIMITER)
                .map(|s| s.to_string())
                .collect(),
        ),
        _ => {
            eprintln!("More than one ReaScore token: {:?}", v);
            None
        }
    }
}

fn reascore_tokens(
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

/// Try to get token from vec, and return [NotationError] at fail.
fn get_token<'a>(
    v: &'a Vec<&str>,
    idx: usize,
) -> Result<&'a str, NotationError> {
    let s = v.get(idx).ok_or(NotationError::NotEnoughTokens(2, 1))?;
    Ok(*s)
}

/// Handles parsing and representation of raw notation
#[derive(Debug, PartialEq, Clone)]
pub enum NotationType {
    /// channel, note
    Note(NoteNotations),
    /// channel, note: note still presents, as chord
    /// events will be deduplicated by mapping to events.
    Chord(ChordNotations),
    Event,
}

#[cfg(test)]
#[test]
fn test_reascore_tokens() {
    assert_eq!(
        reascore_tokens("a:b:c:d", None).ok(),
        Some(vec!["a", "b", "c", "d"])
    );
    assert_eq!(
        reascore_tokens("a:b:c:d", 4).ok(),
        Some(vec!["a", "b", "c", "d"])
    );
    assert_eq!(
        reascore_tokens("a:b:c:d", 3).unwrap_err().to_string(),
        NotationError::TooManyTokens(3).to_string()
    );
    assert_eq!(
        reascore_tokens("a:b:c:d", 5).unwrap_err().to_string(),
        NotationError::NotEnoughTokens(5, 4).to_string()
    );
    assert_eq!(
        reascore_tokens("", 1).unwrap_err().to_string(),
        NotationError::NoTokens("".to_string()).to_string()
    );
    assert_eq!(reascore_tokens("a", None).ok(), Some(vec!["a"]));
}

#[cfg(test)]
#[test]
fn test_parsing() {
    let msg = NotationMessage::from(MNotation::Note {
        channel: 1,
        note: 60,
        tokens: vec![
            "text".to_string(),
            "ReaScore|note-head:cross|dyn:\\mf".to_string(),
        ],
    });
    println!("{}", msg);
    assert_eq!(
        notations_from_midi(msg).unwrap(),
        vec![
            NotationType::Note(NoteNotations::NoteHead(
                note_notations::NoteHead::Cross
            )),
            NotationType::Chord(ChordNotations::Dynamics("\\mf".to_string()))
        ]
    );
}
