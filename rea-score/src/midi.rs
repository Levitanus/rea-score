use rea_rs::{MidiMessage, NotationMessage, Reaper};

pub fn print_midi() {
    let pr = Reaper::get().current_project();
    let it = pr.get_selected_item(0).unwrap();
    let take = it.active_take();
    let _iter = take
        .iter_midi(None)
        .unwrap()
        .map(|event| {
            println!("event: {}", event);
            match NotationMessage::from_raw(event.message().get_raw()) {
                None => (),
                Some(msg) => {
                    println!("raw_msg: {}", msg);
                    let string =
                        String::from_utf8(msg.get_raw()[2..].to_vec());
                    match string {
                        Ok(string) => println!("String: {}", string),
                        Err(_) => println!("can not convert to string"),
                    }
                }
            }
        })
        .count();
}
