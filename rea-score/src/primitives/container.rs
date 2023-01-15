use std::collections::VecDeque;

use fraction::Fraction;

use super::{Chord, EventInfo, EventType, Length, RelativePosition};

#[derive(Debug, PartialEq, Clone)]
pub struct Container {
    events: VecDeque<EventInfo>,
    position: RelativePosition,
    length: Length,
}
impl Container {
    fn new(
        events: VecDeque<EventInfo>,
        position: RelativePosition,
        length: Length,
    ) -> Self {
        Self {
            events,
            position,
            length,
        }
    }
    pub fn empty(
        position: RelativePosition,
        length: Length,
    ) -> Self {
        let mut events = VecDeque::new();
        events.push_back(EventInfo::new(
            position.clone(),
            length.clone(),
            EventType::Rest,
        ));
        Self::new(events, position, length)
    }
    pub fn length(&self) -> &Length {
        &self.length
    }
    pub fn events(&self) -> &VecDeque<EventInfo> {
        &self.events
    }
    pub fn length_mut(&mut self) -> &mut Length {
        &mut self.length
    }
    pub fn events_mut(&mut self) -> &mut VecDeque<EventInfo> {
        &mut self.events
    }

    /// insert event to the container, resolving how to
    /// place it with other events.
    ///
    /// # Returns
    /// - None, if everything OK
    /// - Some(EventInfo), if something should be inserted to the
    ///   next measure.
    /// - Err(String), if something goes wrong.
    pub fn insert(
        &mut self,
        event: EventInfo,
    ) -> Result<Option<EventInfo>, String> {
        let mut idx = self
            .events()
            .iter()
            .position(|evt| evt.contains_pos(&event.position))
            .ok_or(format!(
                "Can not find place for event with position: {:?}",
                event.position
            ))?;
        let (event, append_to_self) =
            self.resolve_event_overlaps(event, idx)?;

        // be sure, length and position of event and current
        // are equal
        let mut current = &mut self.events_mut()[idx];
        if current.position != event.position {
            let head =
                current.cut_head_at_position(&event.position)?;
            idx += 1;
            self.events_mut().insert(idx, head);
            current = &mut self.events_mut()[idx];
        }

        // Now, when current event and event are equal at
        // position and length, and we are sure,
        // everything else is correctly splitted, we
        // can replace old event by the new one, which is
        // constructed below.
        let new_event = match &current.event {
            EventType::Rest => event.event,
            EventType::Chord(chord) => {
                EventType::Chord(chord.clone().push(event.event)?)
            }
            EventType::Note(note) => EventType::Chord(
                Chord::new()
                    .push(EventType::Note(note.clone()))?
                    .push(event.event)?,
            ),
            EventType::Tuplet(_) => todo!(),
        };
        current.set_event(new_event);

        // make sure, nothing is lost:
        match append_to_self {
            None => Ok(None),
            Some(mut head) => {
                // if head starts in the next measure,
                // return it completely.
                if head.position.position() == self.length().get() {
                    head.position.set_measure_index(
                        self.position.get_measure_index() + 1,
                    );
                    head.position.set_position(Fraction::from(0.0));
                    return Ok(Some(head));
                }
                // if head is longer then measure, insert
                // our part recursively, and
                // return head to the caller, to insert to
                // the next measure.
                if head.get_end_position().position()
                    > self.length().get()
                {
                    let mut current = head;
                    // head =
                    //     current.
                    // cut_head(Length::from(self.length.
                    // get() -
                    // current.length.get()))?;
                    let mut head = current.cut_head_at_position(
                        &RelativePosition::new(
                            self.position.get_measure_index(),
                            self.length().get(),
                        ),
                    )?;
                    head.position.set_position(Fraction::from(0.0));
                    head.position.set_measure_index(
                        self.position.get_measure_index() + 1,
                    );
                    // If still something left — things go
                    // bad,
                    // and it's time to return Err.
                    match self.insert(current)? {
                        None => Ok(Some(head)),
                        Some(unexpected) => Err(format!(
                            "unexpected head of event found: {:?}",
                            unexpected
                        )),
                    }
                } else {
                    // if head is part of our measure —
                    // recursively insert it.
                    match self.insert(head)? {
                        None => Ok(None),
                        Some(unexpected) => Err(format!(
                            "unexpected head of event found: {:?}",
                            unexpected
                        )),
                    }
                }
            }
        }
    }

    /// cuts head from inserted event, or from the current
    /// measure event, depends on their overlaps.
    ///
    /// # Returns
    /// (event, head), where:
    /// - event: EventInfo is given event, but, possibly, truncated.
    /// - head: Option<EventInfo> is cut part, from given event, or
    ///   from one,
    /// being present in measure already.
    ///
    /// # Side-effect
    /// can make event at idx shorter.
    fn resolve_event_overlaps(
        &mut self,
        mut event: EventInfo,
        idx: usize,
    ) -> Result<(EventInfo, Option<EventInfo>), String> {
        let mut append_to_self: Option<EventInfo> = None;
        let current = &mut self.events_mut()[idx];
        match event.outlasts(&current) {
            Some(len) => {
                let head = event.cut_head(len)?;
                append_to_self = Some(head);
            }
            None => {
                if let Some(len) = current.outlasts(&event) {
                    let head = current.cut_head(len)?;
                    // idx += 1;
                    // self.events.push_back(head);
                    self.events_mut().insert(idx + 1, head);
                }
            }
        }
        Ok((event, append_to_self))
    }
}
