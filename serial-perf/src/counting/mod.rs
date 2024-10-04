//!
//! Counting stress test that sent null-terminated numbers from 1 to type's max value.
//!

mod rx_state;
use counter::Counter;
use rx_state::RxState;
mod counter;
mod nb;
mod tx_state;
use tx_state::TxState;

// Counting test packets structure
// [0-8 bytes] - count
// [1 byte] - null \0
// [1 byte] - crc8 for count

const MAX_PACKET_SIZE: usize = 10; // 10 - 8 bytes if u64 and 1 byte for nul-terminator 1 byte for crc

use crate::statistics::Statistics;

/// Counting test is a test that sends a special increasing numbers
/// with checksum and null separator and can receive these packets
/// and calculate amount of lost packages by comparing the number in package
///
/// # Template parameters
/// - `Serial` - serial device to use for communication
/// - `Number` - a size of counter used. limited to size of usize. Can be u8, u16, u32, u64 on 64 bit platforms
///
/// # Warning
/// If `Counting` receives a packets from a different `Counting` they both must use same `Number` template argument.
pub struct Counting<Serial, Number> {
    serial: Serial,
    tx_state: TxState<Number>,
    rx_state: RxState<Number>,

    tx_stats: Statistics,
    rx_stats: Statistics,
}

impl<Serial, Number> Counting<Serial, Number>
where
    Number: Default,
{
    pub fn new(serial: Serial) -> Self {
        Self {
            serial,
            tx_state: Default::default(),
            rx_state: Default::default(),
            tx_stats: Default::default(),
            rx_stats: Default::default(),
        }
    }

    pub fn reset(&mut self) {
        self.tx_state = Default::default();
        self.rx_state = Default::default();
        self.tx_stats = Default::default();
        self.rx_stats = Default::default();
    }

    pub fn tx_stats(&self) -> &Statistics {
        &self.tx_stats
    }

    pub fn rx_stats(&self) -> &Statistics {
        &self.rx_stats
    }
}

impl<Serial, Number> Counting<Serial, Number>
where
    Number: Counter,
{
    fn on_byte_received(&mut self, byte: u8) {
        self.rx_state.on_byte_received(byte);
        self.rx_stats.add_successful(1);
    }

    fn on_byte_sent(&mut self) {
        self.tx_state.take();
        self.tx_stats.add_successful(1);
    }

    pub fn loss_stats(&self) -> &Statistics {
        self.rx_state.loss_stats()
    }
}
