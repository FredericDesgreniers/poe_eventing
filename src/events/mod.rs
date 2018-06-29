pub mod manager;

use self::manager::EventManager;
use failure::Error;
use io_watch::poll::Poller;
use regex::Captures;
use std::sync::mpsc::{Sender};
use std::sync::Arc;

/// Generic info about a poe log file line
#[derive(Default, Clone, Debug)]
pub struct PoeLogLineInfo {
    time: String,
    date: String,
}

/// A poe event
#[derive(Debug)]
pub enum PoeEvent {
    JoinedArea(String),
}

/// Wrapper around `EventManager` that gives back poe specific events
pub struct PoeEvents<T: Poller<Output = String>> {
    event_manager: EventManager<T, PoeLogLineInfo>,
    sender: Arc<Sender<(PoeEvent, PoeLogLineInfo)>>,
}

impl<T: Poller<Output = String>> PoeEvents<T> {
    /// Create a new `PoeEvents` struct.
    /// This should be created in the same thread that it will run on.
    ///
    /// `string_poll` A generic Poller that outputs strings
    /// `sender` A send channel. The receiver end will get back the events
    pub fn new(string_poll: T, sender: Sender<(PoeEvent, PoeLogLineInfo)>) -> Self {
        Self {
            event_manager :EventManager::new(string_poll),
            sender: Arc::new(sender),
        }
    }

    /// Register all poe specific events
    pub fn register_poe_events(&mut self) -> Result<(), Error> {
        let manager = &mut self.event_manager;
        let sender = self.sender.clone();

        manager.register_filter(|line: String, _info: &mut PoeLogLineInfo| {
            //TODO: This should be an initial regex pass that extracts data and populates `_info`
            if let Some(index) = line.find("]") {
                line[index + 1..].trim().to_string()
            } else {
                line
            }
        });

        manager.register_event(
            "^: You have entered (?P<location>.*)\\.$",
            move |captures: Captures, info: PoeLogLineInfo| {
                let location = &captures["location"];
                let event = PoeEvent::JoinedArea(location.to_string());

                //TODO: This  shouldn't be unwrap
                sender.send((event, info)).unwrap();
            },
        )?;

        Ok(())
    }

    /// Run infinitely, sending back events to the receiver
    pub fn run(&mut self) -> Result<!, Error> {
        self.event_manager.run()
    }
}
