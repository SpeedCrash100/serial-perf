//!
//! Loopback is a simple utility that send back bytes it's received
//!

use crate::statistics::{CountingStatistics, Statistics};

mod nb;

enum State {
    Receiving,
    Transfer(u8),
}

/// A wrapper around serial that sends data it's received
pub struct Loopback<Serial, TxStats = CountingStatistics, RxStats = CountingStatistics> {
    serial: Serial,
    state: State,

    tx_stats: TxStats,
    rx_stats: RxStats,
}

impl<Serial, TxStats, RxStats> Loopback<Serial, TxStats, RxStats>
where
    TxStats: Statistics,
    RxStats: Statistics,
{
    /// Create a new loopback instance using provided serial and statistics.
    ///
    /// # Note
    /// The provided statistics will not reset upon creation, so you may want to call `reset` after creation if desired.
    pub fn new(serial: Serial, tx_stats: TxStats, rx_stats: RxStats) -> Self {
        Self {
            serial,
            state: State::Receiving,
            tx_stats,
            rx_stats,
        }
    }

    pub fn tx_stats(&self) -> &TxStats {
        &self.tx_stats
    }

    pub fn rx_stats(&self) -> &RxStats {
        &self.rx_stats
    }

    pub fn reset_stats(&mut self) {
        self.tx_stats.reset();
        self.rx_stats.reset();
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
