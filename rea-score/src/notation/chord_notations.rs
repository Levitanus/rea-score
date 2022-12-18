use std::{error::Error, str::FromStr};

#[derive(Debug)]
pub enum ChordNotations {
    Dynamics(String),
}
impl ToString for ChordNotations {
    fn to_string(&self) -> String {
        match self {
            Self::Dynamics(idx) => format!("dyn:{}", idx),
        }
    }
}
impl FromStr for ChordNotations {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("dyn") {
            let d = s.split(":").collect::<Vec<&str>>()[1].to_string();
            Ok(Self::Dynamics(d))
        } else {
            Err(format!("Can not parse {}", s).into())
        }
    }
}
