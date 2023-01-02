use std::{error::Error, str::FromStr};

use super::{get_token, reascore_tokens, NotationError, NotationRender};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NoteNotations {
    NoteHead(NoteHead),
    Voice(u8),
}
impl ToString for NoteNotations {
    fn to_string(&self) -> String {
        match self {
            Self::NoteHead(head) => format!("note-head:{}", head.to_string()),
            Self::Voice(idx) => format!("voice:{}", idx.to_string()),
        }
    }
}
impl FromStr for NoteNotations {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = reascore_tokens(s, None)?;
        match tokens[0] {
            "note-head" => {
                let head = get_token(&tokens, 1)?;
                Ok(Self::NoteHead(head.parse()?))
            }
            "voice" => {
                let idx = get_token(&tokens, 1)?;
                Ok(Self::NoteHead(idx.parse()?))
            }
            x => Err(NotationError::UnexpectedToken(x.to_string()).into()),
        }
    }
}
impl NotationRender for NoteNotations {
    fn render(&self, pitch_string: impl Into<String>) -> String {
        match self {
            Self::NoteHead(head) => {
                format!(
                    "\\override NoteHead.style = #'{} {}",
                    head.to_string(),
                    pitch_string.into()
                )
            }
            Self::Voice(_) => unimplemented!("Voice can not be rendered!"),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum NoteHead {
    #[default]
    Default,
    AltDefault,
    Baroque,
    Neomensural,
    Mensural,
    Petrucci,
    Harmonic,
    HarmonicBlack,
    HarmonicMixed,
    Diamond,
    Cross,
    XCircle,
    Triangle,
    Slash,
}
impl ToString for NoteHead {
    fn to_string(&self) -> String {
        match *self {
            Self::Default => "default".to_string(),
            Self::AltDefault => "altdefault".to_string(),
            Self::Baroque => "baroque".to_string(),
            Self::Neomensural => "neomensural".to_string(),
            Self::Mensural => "mensural".to_string(),
            Self::Petrucci => "petrucci".to_string(),
            Self::Harmonic => "harmonic".to_string(),
            Self::HarmonicBlack => "harmonic-black".to_string(),
            Self::HarmonicMixed => "harmonic-mixed".to_string(),
            Self::Diamond => "diamond".to_string(),
            Self::Cross => "cross".to_string(),
            Self::XCircle => "xcircle".to_string(),
            Self::Triangle => "triangle".to_string(),
            Self::Slash => "slash".to_string(),
        }
    }
}
impl FromStr for NoteHead {
    type Err = NotationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(Self::Default),
            "altdefault" => Ok(Self::AltDefault),
            "baroque" => Ok(Self::Baroque),
            "neomensural" => Ok(Self::Neomensural),
            "mensural" => Ok(Self::Mensural),
            "petrucci" => Ok(Self::Petrucci),
            "harmonic" => Ok(Self::Harmonic),
            "harmonic-black" => Ok(Self::HarmonicBlack),
            "harmonic-mixed" => Ok(Self::HarmonicMixed),
            "diamond" => Ok(Self::Diamond),
            "cross" => Ok(Self::Cross),
            "xcircle" => Ok(Self::XCircle),
            "triangle" => Ok(Self::Triangle),
            "slash" => Ok(Self::Slash),
            x => Err(NotationError::UnexpectedToken(x.to_string())),
        }
    }
}
