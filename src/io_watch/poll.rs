use failure::Error;
use std::io;
use std::io::Read;
use std::thread::sleep;
use std::time::Duration;

/// A general trait to enable polling and getting an output Vector
pub trait Poller {
    type Output;

    /// Block until new output is found and return that output
    fn wait_and_read(&mut self) -> Result<Vec<Self::Output>, Error>;
}

/// Polls a Read implementation for new bytes that are appended.
/// Note that this does not work for anything other than appending
pub struct BytePoll<T: Read> {
    read_from: T,
    wait_delay: u64,
    buffer: Vec<u8>,
}

impl<T: Read> BytePoll<T> {
    /// Create a new BytePoll using
    /// `read_from` A read implementation, for example a file
    /// `wait_delay` How long to wait between polls
    /// `buffer_len` Poll buffer length
    pub fn new(read_from: T, wait_delay: u64, buffer_len: usize) -> Result<Self, io::Error> {
        Ok(Self {
            read_from,
            wait_delay,
            buffer: vec![0u8; buffer_len],
        })
    }
}

impl<T: Read> Poller for BytePoll<T> {
    type Output = u8;

    /// Blocks until new bytes are appended to the file, then returns the new bytes
    fn wait_and_read(&mut self) -> Result<Vec<<Self as Poller>::Output>, Error> {
        let buffer = &mut self.buffer;
        let read_from = &mut self.read_from;

        loop {
            let bytes_read = read_from.read(&mut buffer[..])?;

            if bytes_read > 0 {
                return Ok(buffer[..bytes_read].to_vec());
            }

            sleep(Duration::from_millis(self.wait_delay));
        }
    }
}

/// Poll a Read implementation for new chars
pub struct CharPoll<T>
where
    T: Poller<Output = u8>,
{
    byte_poll: T,
}

impl<T: Read> CharPoll<BytePoll<T>> {
    /// Create a new CharPoll using
    /// `read_from` A read implementation, for example a file
    /// `wait_delay` How long to wait between polls
    /// `buffer_len` Poll buffer length
    pub fn new(read_from: T, wait_delay: u64, buffer_len: usize) -> Result<Self, io::Error> {
        Ok(Self {
            byte_poll: BytePoll::new(read_from, wait_delay, buffer_len)?,
        })
    }
}

impl<T: Poller<Output = u8>> From<T> for CharPoll<T> {
    fn from(byte_poll: T) -> Self {
        Self { byte_poll }
    }
}

impl<T: Poller<Output = u8>> Poller for CharPoll<T> {
    type Output = char;

    /// Blocks until new bytes are appended to file and returns them as utf8 characters
    fn wait_and_read(&mut self) -> Result<Vec<<Self as Poller>::Output>, Error> {
        let new_bytes = self.byte_poll.wait_and_read()?;
        let new_chars = String::from_utf8(new_bytes)?.chars().collect();

        Ok(new_chars)
    }
}

/// Poll for new appended lines to a Read implementation
pub struct StringLinePoll<T: Poller<Output = char>> {
    char_poll: T,
    char_buffer: Vec<char>,
}

impl<T: Read> StringLinePoll<CharPoll<BytePoll<T>>> {
    /// Create a new StringLinePoll using
    /// `read_from` A read implementation, for example a file
    /// `wait_delay` How long to wait between polls
    /// `buffer_len` Poll buffer length
    pub fn new(read_from: T, wait_delay: u64, buffer_len: usize) -> Result<Self, io::Error> {
        Ok(Self {
            char_poll: CharPoll::new(read_from, wait_delay, buffer_len)?,
            char_buffer: Vec::new(),
        })
    }
}

impl<T: Poller<Output = char>> From<T> for StringLinePoll<T> {
    fn from(char_poll: T) -> Self {
        Self {
            char_poll,
            char_buffer: Vec::new(),
        }
    }
}

impl<T: Poller<Output = char>> Poller for StringLinePoll<T> {
    type Output = String;

    /// Blocks until new lines are appended to Read implementation and returns those lines
    fn wait_and_read(&mut self) -> Result<Vec<<Self as Poller>::Output>, Error> {
        let mut lines = Vec::new();

        let buffer = &mut self.char_buffer;
        let char_poll = &mut self.char_poll;

        while lines.len() == 0 {
            for chr in char_poll.wait_and_read()? {
                match chr {
                    '\n' => {
                        lines.push(buffer.iter().collect::<String>());
                        buffer.clear();
                    }
                    chr => {
                        buffer.push(chr);
                    }
                }
            }
        }

        Ok(lines)
    }
}
