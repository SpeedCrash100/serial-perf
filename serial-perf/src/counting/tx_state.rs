use heapless::Vec;

use super::{
    counter::{Counter, LeBytes},
    MAX_PACKET_SIZE,
};

pub struct TxState<Number> {
    number_to_send: Number,
    data_to_send: Vec<u8, MAX_PACKET_SIZE>,
    checksum_enabled: bool,
}

impl<Number> Default for TxState<Number>
where
    Number: Default,
{
    fn default() -> Self {
        Self {
            number_to_send: Default::default(),
            data_to_send: Vec::new(),
            checksum_enabled: true,
        }
    }
}

impl<Number> TxState<Number>
where
    Number: Counter,
{
    pub fn new_without_checksum() -> Self {
        Self {
            number_to_send: Default::default(),
            data_to_send: Vec::new(),
            checksum_enabled: false,
        }
    }

    pub fn peek(&mut self) -> u8 {
        if self.data_to_send.is_empty() {
            self.prepare_next_packet();
        }

        debug_assert!(!self.data_to_send.is_empty());

        self.data_to_send.last().copied().unwrap_or(0)
    }

    pub fn take(&mut self) -> u8 {
        let out = self.peek();
        self.data_to_send.pop();

        out
    }

    fn prepare_next_packet(&mut self) {
        let next = self.number_to_send.pop();
        let data = next.to_le_bytes().into_packet(self.checksum_enabled);
        self.data_to_send = data;
    }
}
