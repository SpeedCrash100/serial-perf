//!
//! Loopback is a simple utility that send back bytes it's received
//!

use crate::statistics::Statistics;

mod nb;

enum State {
    Receiving,
    Transfer(u8),
}

/// A wrapper around serial that sends data it's received
pub struct Loopback<Serial> {
    serial: Serial,
    state: State,

    tx_stats: Statistics,
    rx_stats: Statistics,
}

impl<Serial> Loopback<Serial> {
    pub fn new(serial: Serial) -> Self {
        Self {
            serial,
            state: State::Receiving,
            tx_stats: Default::default(),
            rx_stats: Default::default(),
        }
    }

    pub fn tx_stats(&self) -> &Statistics {
        &self.tx_stats
    }

    pub fn rx_stats(&self) -> &Statistics {
        &self.rx_stats
    }

    pub fn reset_stats(&mut self) {
        self.tx_stats = Default::default();
        self.rx_stats = Default::default();
    }

    fn on_byte_received(&mut self, byte: u8) {
        match self.state {
            State::Receiving => (),
            State::Transfer(_) => {
                // We have tried to replace byte we did not sent, so we lost it -> add Tx Error
                self.tx_stats.add_failed(1);
            }
        };

        self.state = State::Transfer(byte);
        self.rx_stats.add_successful(1);
    }

    fn on_byte_sent(&mut self) {
        self.state = State::Receiving;
        self.tx_stats.add_successful(1);
    }

    fn byte_to_send(&mut self) -> Option<u8> {
        match self.state {
            State::Receiving => None,
            State::Transfer(byte) => Some(byte),
        }
    }
}
