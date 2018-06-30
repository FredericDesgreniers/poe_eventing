pub mod manager;

use self::manager::EventManager;
use failure::Error;
use io_watch::poll::Poller;
use regex::{Captures, Regex};
use std::sync::mpsc::Sender;
use std::sync::Arc;

lazy_static! {
    static ref REGEX_LINE_INFO: Regex = {
        let regex = Regex::new(r"^(?P<year>\d{4})/(?P<month>\d{2})/(?P<day>\d{2}) (?P<hour>\d{2}):(?P<minute>\d{2}):(?P<second>\d{2}) (?P<time>\d+).*]");
        regex.unwrap()
    };
}

/// Generic info about a poe log file line
#[derive(Default, Clone, Debug)]
pub struct PoeLogLineInfo {
    time: u64,
}

/// A poe event
#[derive(Debug)]
pub enum PoeEvent {
    ConnectingToInstance(String),
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
            event_manager: EventManager::new(string_poll),
            sender: Arc::new(sender),
        }
    }

    /// Register all poe specific events
    pub fn register_poe_events(&mut self) -> Result<(), Error> {
        let manager = &mut self.event_manager;

        manager.register_filter(|line: String, info: &mut PoeLogLineInfo| {
            if let Some(captures) = REGEX_LINE_INFO.captures(&line) {
                info.time = captures["time"].parse::<u64>().expect("time");
                line[captures[0].len()..].trim().to_string()
            } else {
                line
            }
        });
        {
            let sender = self.sender.clone();
            manager.register_event(
                "^: You have entered (?P<location>.*)\\.$",
                move |captures: Captures, info: PoeLogLineInfo| {
                    let location = &captures["location"];
                    let event = PoeEvent::JoinedArea(location.to_string());

                    //TODO: This  shouldn't be unwrap
                    sender.send((event, info)).unwrap();
                },
            )?;
        };
        {
            let sender = self.sender.clone();
            manager.register_event(
                "^Connecting to instance server at (?P<ip>.*)$",
                move |captures: Captures, info: PoeLogLineInfo| {
                    let ip = &captures["ip"];

                    sender
                        .send((PoeEvent::ConnectingToInstance(ip.to_string()), info))
                        .unwrap();
                },
            )?;
        };

        Ok(())
    }

    /// Run infinitely, sending back events to the receiver
    pub fn run(&mut self) -> Result<!, Error> {
        self.event_manager.run()
    }
}
