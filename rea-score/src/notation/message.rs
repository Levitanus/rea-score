use super::{
    reascore_tokens, NotationError, NotationType, NOTATION_DELIMITER, SECTION,
};
use rea_rs::midi::Notation as MNotation;
use rea_rs::NotationMessage;

pub struct MidiFuncs {}
impl MidiFuncs {
    /// Get reascore notations, if any.
    pub fn parse_notations(msg: NotationMessage) -> Option<Vec<NotationType>> {
        match msg.notation() {
            MNotation::Note {
                channel: _,
                note: _,
                tokens,
            } => {
                let tokens = reascore_notation_string(&tokens)?;
                let notes = tokens.iter().filter_map(|tk| {
                    Some(NotationType::Note(tk.parse().ok()?))
                });
                let chords = tokens.iter().filter_map(|tk| {
                    Some(NotationType::Chord(tk.parse().ok()?))
                });
                Some(notes.chain(chords).collect())
            }
            MNotation::Track(_) => todo!(),
            MNotation::Unknown(_) => todo!(),
        }
    }

    /// Update message notations to hold the given.
    ///
    /// If no single reascore notation present, or even no reascore section in
    /// message tokens — they will be added. Otherwise, particular token will
    /// be replaced by actual. Nothing will be removed.
    pub fn replace_notations(
        mut msg: NotationMessage,
        notations: impl IntoIterator<Item = NotationType> + Clone + std::fmt::Debug,
    ) -> Result<NotationMessage, NotationError> {
        match msg.notation() {
            MNotation::Note {
                channel,
                note,
                tokens,
            } => {
                let mut strings = notations
                    .clone()
                    .into_iter()
                    .map(|nt| nt.to_string())
                    .collect::<Vec<_>>();
                let mut rs_tokens = match reascore_notation_string(&tokens) {
                    None => Vec::new(),
                    Some(v) => v,
                };
                let mut rs_tokens = rs_tokens
                    .iter_mut()
                    .map(|tk| -> Result<String, NotationError> {
                        let tk_start = reascore_tokens(tk, None)?[0];
                        for (idx, st) in strings.iter().enumerate() {
                            if st.starts_with(tk_start) {
                                return Ok(strings.swap_remove(idx));
                            }
                        }
                        Ok(tk.clone())
                    })
                    .collect::<Result<Vec<String>, NotationError>>()?;
                rs_tokens.extend(strings);
                let tokens = replace_reascore_notation_string(
                    tokens,
                    format!(
                        "{SECTION}{NOTATION_DELIMITER}{}",
                        rs_tokens.join(NOTATION_DELIMITER)
                    ),
                );
                msg.set_notation(MNotation::Note {
                    channel,
                    note,
                    tokens,
                });
                Ok(msg)
            }
            x => Err(NotationError::UnexpectedNotation {
                notation: format!("{:?}", notations),
                object: format!("{x}"),
            }),
        }
    }

    /// Remove notations from the message.
    ///
    /// It will remove only notations, that exact matches given objects.
    pub fn remove_notations(
        mut msg: NotationMessage,
        notations: Vec<NotationType>,
    ) -> NotationMessage {
        match msg.notation() {
            MNotation::Note {
                channel,
                note,
                tokens,
            } => {
                let tokens = tokens
                    .into_iter()
                    .map(|mut tk| {
                        notations
                            .iter()
                            .map(|nt| {
                                tk = tk.replace(nt.to_string().as_str(), "")
                            })
                            .count();
                        tk
                    })
                    .collect::<Vec<_>>();
                msg.set_notation(MNotation::Note {
                    channel,
                    note,
                    tokens,
                })
            }
            _ => unimplemented!(),
        }
        msg
    }
}
/// Get reascore tokens, if any.
fn reascore_notation_string(tokens: &Vec<String>) -> Option<Vec<String>> {
    let v: Vec<String> = tokens
        .iter()
        .filter(|tk| tk.starts_with(SECTION))
        .map(|tk| tk.clone())
        .collect();
    match v.len() {
        0 => None,
        1 => {
            let mut tk = v[0]
                .split(NOTATION_DELIMITER)
                .map(|s| s.to_string())
                .collect::<Vec<_>>();
            tk.remove(0);
            Some(tk)
        }
        _ => {
            eprintln!("More than one ReaScore token: {:?}", v);
            None
        }
    }
}

/// Replace Reascore token string by given, or push one if not exists.
fn replace_reascore_notation_string(
    mut tokens: Vec<String>,
    reascore_string: String,
) -> Vec<String> {
    let mut replaced = false;
    for tk in tokens.iter_mut() {
        if tk.starts_with(SECTION) {
            *tk = reascore_string.clone();
            replaced = true;
            break;
        }
    }
    match replaced {
        true => (),
        false => tokens.push(format!("text {reascore_string}")),
    }
    tokens
}

#[cfg(test)]
mod tests {
    use rea_rs::{midi::Notation as MNotation, NotationMessage};

    use crate::notation::{
        chord_notations::ChordNotations,
        message::MidiFuncs,
        note_notations::{self, NoteNotations},
        reascore_tokens, NotationError, NotationType,
    };

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
        assert_eq!(
            MidiFuncs::parse_notations(msg.clone()).unwrap(),
            vec![
                NotationType::Note(NoteNotations::NoteHead(
                    note_notations::NoteHead::Cross
                )),
                NotationType::Chord(ChordNotations::Dynamics(
                    "\\mf".to_string()
                ))
            ]
        );
        let msg1 = MidiFuncs::replace_notations(
            msg.clone(),
            vec![NotationType::Note(NoteNotations::NoteHead(
                note_notations::NoteHead::Baroque,
            ))],
        )
        .expect("Can not replace notations");
        assert_eq!(
            MidiFuncs::parse_notations(msg1).unwrap(),
            vec![
                NotationType::Note(NoteNotations::NoteHead(
                    note_notations::NoteHead::Baroque
                )),
                NotationType::Chord(ChordNotations::Dynamics(
                    "\\mf".to_string()
                ))
            ]
        );
        let msg1 = MidiFuncs::remove_notations(
            msg,
            vec![NotationType::Note(NoteNotations::NoteHead(
                note_notations::NoteHead::Cross,
            ))],
        );
        assert_eq!(
            MidiFuncs::parse_notations(msg1).unwrap(),
            vec![NotationType::Chord(ChordNotations::Dynamics(
                "\\mf".to_string()
            ))]
        );
        let msg = NotationMessage::from(MNotation::Note {
            channel: 1,
            note: 60,
            tokens: Vec::new(),
        });
        let msg1 = MidiFuncs::replace_notations(
            msg.clone(),
            vec![NotationType::Note(NoteNotations::NoteHead(
                note_notations::NoteHead::Baroque,
            ))],
        )
        .expect("Can not replace notations");
        assert_eq!(
            MidiFuncs::parse_notations(msg1).unwrap(),
            vec![NotationType::Note(NoteNotations::NoteHead(
                note_notations::NoteHead::Baroque
            )),]
        );
    }
}
