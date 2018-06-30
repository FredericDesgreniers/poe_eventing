#![feature(nll)]
#![feature(never_type)]
extern crate core;
extern crate failure;
extern crate poe_data;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod events;
mod io_watch;

use events::PoeEvents;
use failure::Error;
use io_watch::poll::StringLinePoll;
use std::fs::File;
use std::sync::mpsc::channel;
use std::thread;

fn main() -> Result<(), Error> {
    let line_poll = StringLinePoll::new(
        File::open(
            "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Path of Exile\\logs\\Client.txt",
        )?,
        20,
        1024,
    )?;

    let (sender, receiver) = channel();

    thread::spawn(|| {
        let mut poe_events = PoeEvents::new(line_poll, sender);

        poe_events.register_poe_events().unwrap();
        poe_events.run().unwrap()
    });

    for (event, info) in receiver {
        println!("{:?}- {:?}", info, event)
    }

    Ok(())
}
