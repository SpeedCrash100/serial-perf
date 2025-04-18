use crate::statistics::Statistics;

use super::{
    counter::{Counter, LeBytes},
    MAX_PACKET_SIZE,
};

enum InternalState {
    Receiving,
    WaitingForCRC,
}

pub struct RxState<Number, LossStats> {
    /// The last number received to analyze the packet loss.
    number: Option<Number>,

    /// The current packet being received.
    current_packet: heapless::Vec<u8, MAX_PACKET_SIZE>,
    /// State for parsing incoming package
    internal_state: InternalState,

    /// The statistics of the packet loss. Note: this is not a rx_stats because it's analyze packets, not bytes
    loss_stats: LossStats,

    checksum_enabled: bool,
}

impl<Number, LossStats> RxState<Number, LossStats>
where
    Number: Counter,
    LossStats: Statistics,
{
    pub fn new(loss_stats: LossStats) -> Self {
        Self {
            number: None,
            current_packet: heapless::Vec::new(),
            internal_state: InternalState::Receiving,
            loss_stats,
            checksum_enabled: true,
        }
    }

    pub fn new_without_checksum(loss_stats: LossStats) -> Self {
        Self {
            number: None,
            current_packet: heapless::Vec::new(),
            internal_state: InternalState::Receiving,
            loss_stats,
            checksum_enabled: false,
        }
    }

    pub fn reset(&mut self) {
        self.number = None;
        self.current_packet.clear();
        self.internal_state = InternalState::Receiving;
        self.loss_stats.reset();
    }

    /// Parses and handling incoming packet
    fn parse_current_packet(&mut self, crc: u8) {
        let checksum = if self.checksum_enabled {
            Some(crc)
        } else {
            None
        };

        if let Some(new_number_raw) =
            Number::Bytes::from_slice_checked(&self.current_packet, checksum)
        {
            let new_number = Number::from_le_bytes(new_number_raw);
            self.on_new_number(new_number);
        }

        self.current_packet.clear();
    }

    fn on_new_number(&mut self, new_number: Number) {
        if let Some(ref old_number) = self.number {
            let distance = old_number.distance(&new_number);
            let loss = distance - 1;
            #[cfg(feature = "print")]
            if loss > 0 {
                let new_normal = Number::min_counter().distance(&new_number);
                let old_normal = Number::min_counter().distance(old_number);
                println!("LOST: {:?} -> {:?}: {} lost", old_normal, new_normal, loss);
            }

            self.loss_stats.add_failed(loss);
            // FIXME: Detect absurd jumps?
        }

        self.number = Some(new_number);
        self.loss_stats.add_successful(1);
    }

    pub fn on_byte_received(&mut self, byte: u8) {
        match self.internal_state {
            InternalState::Receiving => self.on_byte_received_normal(byte),
            InternalState::WaitingForCRC => self.on_byte_received_crc(byte),
        }
    }

    pub fn loss_stats(&self) -> &LossStats {
        &self.loss_stats
    }

    fn on_byte_received_normal(&mut self, byte: u8) {
        // Null terminator
        if byte == 0 {
            self.internal_state = InternalState::WaitingForCRC;
            return;
        }

        // We cannot insert more bytes so try parse current package and then insert
        if self.current_packet.is_full() {
            self.current_packet.clear();
            self.internal_state = InternalState::Receiving;
        }

        debug_assert!(!self.current_packet.is_full());
        self.current_packet.push(byte).unwrap();
    }

    fn on_byte_received_crc(&mut self, byte: u8) {
        self.parse_current_packet(byte);
        self.internal_state = InternalState::Receiving;
    }
}
