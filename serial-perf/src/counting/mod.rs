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

use crate::statistics::{CountingStatistics, Statistics};
pub use {
    nb::ValidCountingNb, nb::ValidCountingNbError, nb::ValidCountingNbRead,
    nb::ValidCountingNbWrite,
};

pub mod prelude {
    pub use super::{
        ValidCounting, ValidCountingNb, ValidCountingNbError, ValidCountingNbRead,
        ValidCountingNbWrite,
    };
}

/// Trait for any valid counting test
///
/// The counting test is valid if supported Number type used and correct Stats set. The serial is ignored
pub trait ValidCounting {
    type Serial;
    type Number: Counter;
    type TxStats: Statistics;
    type RxStats: Statistics;
    type LossStats: Statistics;

    fn tx_stats(&self) -> &Self::TxStats;
    fn rx_stats(&self) -> &Self::RxStats;
    fn loss_stats(&self) -> &Self::LossStats;
    fn reset(&mut self);
}

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
pub struct Counting<
    Serial,
    Number,
    TxStats = CountingStatistics,
    RxStats = CountingStatistics,
    LossStats = CountingStatistics,
> {
    serial: Serial,
    tx_state: TxState<Number>,
    rx_state: RxState<Number, LossStats>,

    tx_stats: TxStats,
    rx_stats: RxStats,
}

impl<Serial, Number, TxStats, RxStats, LossStats>
    Counting<Serial, Number, TxStats, RxStats, LossStats>
where
    Number: Counter,
    TxStats: Statistics,
    RxStats: Statistics,
    LossStats: Statistics,
{
    pub fn new(
        serial: Serial,
        tx_stats: TxStats,
        rx_stats: RxStats,
        loss_stats: LossStats,
    ) -> Self {
        Self {
            serial,
            tx_state: Default::default(),
            rx_state: RxState::new(loss_stats),
            tx_stats,
            rx_stats,
        }
    }

    pub fn new_without_checksum(
        serial: Serial,
        tx_stats: TxStats,
        rx_stats: RxStats,
        loss_stats: LossStats,
    ) -> Self {
        Self {
            serial,
            tx_state: TxState::new_without_checksum(),
            rx_state: RxState::new_without_checksum(loss_stats),
            tx_stats,
            rx_stats,
        }
    }
}

impl<Serial, Number, TxStats, RxStats, LossStats> ValidCounting
    for Counting<Serial, Number, TxStats, RxStats, LossStats>
where
    Number: Counter,
    TxStats: Statistics,
    RxStats: Statistics,
    LossStats: Statistics,
{
    type Serial = Serial;
    type Number = Number;
    type TxStats = TxStats;
    type RxStats = RxStats;
    type LossStats = LossStats;

    fn tx_stats(&self) -> &TxStats {
        &self.tx_stats
    }

    fn rx_stats(&self) -> &RxStats {
        &self.rx_stats
    }

    fn loss_stats(&self) -> &LossStats {
        self.rx_state.loss_stats()
    }

    fn reset(&mut self) {
        self.tx_state.reset();
        self.rx_state.reset();
        self.tx_stats.reset();
        self.rx_stats.reset();
    }
}

impl<Serial, Number, TxStats, RxStats, LossStats>
    Counting<Serial, Number, TxStats, RxStats, LossStats>
where
    Number: Counter,
    TxStats: Statistics,
    RxStats: Statistics,
    LossStats: Statistics,
{
    fn on_byte_received(&mut self, byte: u8) {
        self.rx_state.on_byte_received(byte);
        self.rx_stats.add_successful(1);
    }

    fn on_byte_sent(&mut self) {
        self.tx_state.take();
        self.tx_stats.add_successful(1);
    }
}
