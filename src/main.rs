#![feature(nll)]
#![feature(never_type)]
extern crate failure;
extern crate regex;

mod events;
mod io_watch;

use events::EventManager;
use failure::Error;
use io_watch::poll::StringLinePoll;
use std::fs::File;

fn main() -> Result<(), Error> {
    let line_poll = StringLinePoll::new(
        File::open(
            "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Path of Exile\\logs\\Client.txt",
        )?,
        20,
        1024,
    )?;

    let mut event_manager = EventManager::new(line_poll);

    event_manager.register_filter(|line: String| {
        if let Some(index) = line.find("]") {
            line[index + 1..].trim().to_string()
        } else {
            line
        }
    });

    event_manager.register_event("^: You have entered (?P<location>.*)\\.$", |c| {
        let location = &c["location"];
        println!("Entered: {}", location);
    })?;

    event_manager.run()?;
}
