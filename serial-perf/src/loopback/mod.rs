//!
//! Loopback is a simple utility that send back bytes it's received
//!

mod nb;

enum State {
    Receiving,
    Transfer(u8),
}

/// A wrapper around serial that sends data it's received
pub struct Loopback<Serial> {
    serial: Serial,
    state: State,
}

impl<Serial> Loopback<Serial> {
    pub fn new(serial: Serial) -> Self {
        Self {
            serial,
            state: State::Receiving,
        }
    }

    fn on_byte_received(&mut self, byte: u8) {
        // FIXME: handle the situation if we in State::Transfer already
        self.state = State::Transfer(byte)
    }

    fn on_byte_sent(&mut self) {
        self.state = State::Receiving
    }

    fn byte_to_send(&mut self) -> Option<u8> {
        match self.state {
            State::Receiving => None,
            State::Transfer(byte) => Some(byte),
        }
    }
}
