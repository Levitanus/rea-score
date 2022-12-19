use std::{error::Error, str::FromStr};

use super::{get_token, reascore_tokens, NotationError};

#[derive(Debug, PartialEq)]
pub enum NoteNotations {
    Voice(u8),
}
impl ToString for NoteNotations {
    fn to_string(&self) -> String {
        match self {
            Self::Voice(idx) => format!("voice:{}", idx),
        }
    }
}
impl FromStr for NoteNotations {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = reascore_tokens(s, None)?;
        match tokens[0] {
            "voice" => {
                let idx = get_token(&tokens, 1)?;
                Ok(Self::Voice(idx.parse()?))
            }
            x => Err(NotationError::UnexpectedToken(x.to_string()).into()),
        }
    }
}
