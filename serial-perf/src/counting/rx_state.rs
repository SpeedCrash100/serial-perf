use crate::statistics::Statistics;

use super::{
    counter::{Counter, LeBytes},
    MAX_PACKET_SIZE,
};

pub struct RxState<Number> {
    /// The last number received to analyze the packet loss.
    number: Option<Number>,

    /// The current packet being received.
    current_packet: heapless::Vec<u8, MAX_PACKET_SIZE>,

    /// The statistics of the packet loss. Note: this is not a rx_stats because it's analyze packets, not bytes
    loss_stats: Statistics,
}

impl<Number> Default for RxState<Number>
where
    Number: Default,
{
    fn default() -> Self {
        Self {
            number: None,
            current_packet: heapless::Vec::new(),
            loss_stats: Default::default(),
        }
    }
}

impl<Number> RxState<Number>
where
    Number: Counter,
{
    /// Parses and handling incoming packet
    fn parse_current_packet(&mut self) {
        if let Some(new_number_raw) = Number::Bytes::from_slice_checked(&self.current_packet) {
            let new_number = Number::from_le_bytes(new_number_raw);
            self.on_new_number(new_number);
        }

        self.current_packet.clear();
    }

    fn on_new_number(&mut self, new_number: Number) {
        if let Some(ref old_number) = self.number {
            let distance = old_number.distance(&new_number);
            let loss = distance - 1;
            self.loss_stats.add_failed(loss);
        }

        self.number = Some(new_number);
        self.loss_stats.add_successful(1);
    }

    pub fn on_byte_received(&mut self, byte: u8) {
        // Null terminator
        if byte == 0 {
            self.parse_current_packet();
            return;
        }

        // We cannot insert more bytes so try parse current package and then insert
        if self.current_packet.is_full() {
            self.parse_current_packet();
        }

        debug_assert!(!self.current_packet.is_full());
        self.current_packet.insert(0, byte).unwrap();
    }
}
