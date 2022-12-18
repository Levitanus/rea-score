use std::{error::Error, str::FromStr};

#[derive(Debug)]
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
        if s.starts_with("voice") {
            let idx: u8 = s.split(":").collect::<Vec<&str>>()[1].parse()?;
            Ok(Self::Voice(idx))
        } else {
            Err(format!("Can not parse {}", s).into())
        }
    }
}
