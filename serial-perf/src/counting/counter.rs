use crc::Crc;

use super::MAX_PACKET_SIZE;
use core::fmt::Debug;

/// Internal bytes for counter that should always have non zero bytes
pub trait LeBytes: Sized + Debug {
    fn from_slice_checked(slice: &[u8], checksum: Option<u8>) -> Option<Self>;
    /// Returns package for sending these bytes.
    ///
    /// If checksum enabled crc will be calculated and appended to the end of packet,
    /// otherwise it will be set to value of the first byte.
    fn into_packet(self, checksum_enabled: bool) -> heapless::Vec<u8, MAX_PACKET_SIZE>;

    fn ones() -> Self;

    fn next(self) -> Self;
    fn prev(self) -> Self;
    fn distance(&self, other: &Self) -> usize;
}

impl<const N: usize> LeBytes for [u8; N] {
    fn from_slice_checked(slice: &[u8], checksum: Option<u8>) -> Option<Self> {
        if N != slice.len() {
            return None;
        }

        if let Some(checksum) = checksum {
            let crc = Crc::<u8>::new(&crc::CRC_8_AUTOSAR);
            let checksum_input = crc.checksum(slice);
            if checksum_input != checksum {
                return None;
            }
        }

        let mut out: Self = [0; N];
        out.copy_from_slice(slice);
        Some(out)
    }

    fn into_packet(self, checksum_enabled: bool) -> heapless::Vec<u8, MAX_PACKET_SIZE> {
        let mut out = heapless::Vec::new();
        let mut crc_data = heapless::Vec::<_, MAX_PACKET_SIZE>::new();

        for byte in self {
            out.insert(0, byte).unwrap();
            crc_data.push(byte).unwrap();
        }

        let mut checksum = crc_data.first().copied().unwrap_or(0);
        if checksum_enabled {
            let crc = Crc::<u8>::new(&crc::CRC_8_AUTOSAR);
            checksum = crc.checksum(crc_data.as_slice());
        }

        out.insert(0, 0).unwrap();
        out.insert(0, checksum).unwrap();

        out
    }

    fn ones() -> Self {
        [0x01; N]
    }

    fn next(mut self) -> Self {
        for byte in &mut self {
            // LE byte overflow, add carry to the next one.
            if *byte == 0xFF {
                *byte = 0x01;
                continue;
            }

            *byte = byte.wrapping_add(1);
            break;
        }

        self
    }
    fn prev(mut self) -> Self {
        for byte in &mut self {
            // LE byte underflow, add carry to the next one.
            if *byte <= 0x01 {
                *byte = 0xFF;
                continue;
            }

            *byte = byte.wrapping_sub(1);
            break;
        }

        self
    }

    fn distance(&self, other: &Self) -> usize {
        // Use allowed only if self < other
        let iter_pair = self.iter().zip(other.iter());

        // 0b0000_1101
        // 0b1111_0000
        // 0b1110_1101

        let mut total_distance = 0_usize;
        let mut overflow = false;
        for (pos, (left, right)) in iter_pair.enumerate() {
            let mut local_distance = if left <= right {
                right - left
            } else {
                let to_max_dist = 0xFF - left;
                let from_min_dist = *right;
                to_max_dist + from_min_dist
            };

            if overflow {
                local_distance = local_distance.wrapping_sub(1);
            }

            total_distance += (local_distance as usize) << (8 * pos);

            if right < left {
                overflow = true;
            }
        }

        total_distance
    }
}

pub trait Counter: Default + Debug {
    type Bytes: LeBytes;

    /// Increment the counter and return its previous value.
    fn pop(&mut self) -> Self;
    /// Decrement the counter.
    fn push(&mut self);
    /// Gets the one sided distance between two counters.
    ///
    /// If self < value then the distance is value-self, otherwise it's the distance to the MAX and from 1 to value.
    fn distance(&self, value: &Self) -> usize;

    fn to_le_bytes(&self) -> Self::Bytes;
    fn from_le_bytes(bytes: Self::Bytes) -> Self;

    fn min_counter() -> Self {
        let ones = Self::Bytes::ones();
        Self::from_le_bytes(ones)
    }
}

macro_rules! impl_counter {
    ($x:ty, $sz:expr) => {
        impl Counter for $x {
            type Bytes = [u8; $sz];

            fn pop(&mut self) -> Self {
                let mut result = *self;
                if result < Self::min_counter() {
                    result = Self::min_counter();
                }

                let bytes = result.to_le_bytes();
                let next_bytes = bytes.next();
                *self = Self::from_le_bytes(next_bytes);

                result
            }

            fn push(&mut self) {
                let bytes = self.to_le_bytes();
                let prev_bytes = bytes.prev();

                *self = Self::from_le_bytes(prev_bytes);
                if *self < Self::min_counter() {
                    *self = Self::min_counter();
                }
            }

            fn distance(&self, value: &Self) -> usize {
                let left = self.to_le_bytes();
                let right = value.to_le_bytes();

                left.distance(&right)

                // if *self < *value {
                //     (value - self) as usize
                // } else {
                //     // 0 1 2 3 4 5 6 7 8 9
                //     //   $     ^---------^
                //     //   |         9-4=5
                //     //    \ 1 step

                //     let to_max_dist = <$x>::MAX - *self;
                //     let from_min_dist = *value - <$x>::min_counter() + 1;
                //     to_max_dist as usize + from_min_dist as usize
                // }
            }

            fn to_le_bytes(&self) -> Self::Bytes {
                Self::to_le_bytes(*self)
            }

            fn from_le_bytes(bytes: Self::Bytes) -> Self {
                Self::from_le_bytes(bytes)
            }
        }
    };
}

#[cfg(any(
    target_pointer_width = "8",
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl_counter!(u8, 1);
#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl_counter!(u16, 2);
#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl_counter!(u32, 4);
#[cfg(target_pointer_width = "64")]
impl_counter!(u64, 8);

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    ))]
    #[test]
    fn push_pop() {
        let mut test_counter = 5_u16;
        let pop_value = test_counter.pop();
        assert_eq!(pop_value, u16::min_counter());
        assert_eq!(test_counter, u16::min_counter() + 1);

        test_counter.push();
        assert_eq!(test_counter, u16::min_counter());
    }

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    ))]
    #[test]
    fn distance() {
        let mut test_counter = 0x0101_u16;
        let pop_value = test_counter.pop();
        assert_eq!(pop_value.distance(&test_counter), 1);
        assert_eq!(test_counter.distance(&pop_value), 65534);
    }

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    ))]
    #[test]
    fn double_conversion() {
        let test_counter = 5_u16;
        let as_le_bytes = test_counter.to_le_bytes();
        let mut as_data_queue = as_le_bytes.into_packet(true);
        assert_eq!(as_data_queue.len(), 2 + 1 + 1); // +1 for null terminator +1 crc

        let crc = *as_data_queue.first().unwrap();

        let mut recv_side = heapless::Vec::<u8, MAX_PACKET_SIZE>::new();
        for _ in 0..2 {
            recv_side.push(as_data_queue.pop().unwrap()).unwrap();
        }

        let recv_bytes = <u16 as Counter>::Bytes::from_slice_checked(&recv_side, Some(crc))
            .expect("failed to create from slice");

        assert_eq!(as_le_bytes, recv_bytes);

        let recv_value = u16::from_le_bytes(recv_bytes);
        assert_eq!(recv_value, test_counter)
    }

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    ))]
    #[test]
    fn double_conversion_no_checksum() {
        let test_counter = 5_u16;
        let as_le_bytes = test_counter.to_le_bytes();
        let mut as_data_queue = as_le_bytes.into_packet(false);
        assert_eq!(as_data_queue.len(), 2 + 1 + 1); // +1 for null terminator +1 crc

        let _crc = *as_data_queue.first().unwrap();

        let mut recv_side = heapless::Vec::<u8, MAX_PACKET_SIZE>::new();
        for _ in 0..2 {
            recv_side.push(as_data_queue.pop().unwrap()).unwrap();
        }

        let recv_bytes = <u16 as Counter>::Bytes::from_slice_checked(&recv_side, None)
            .expect("failed to create from slice");

        assert_eq!(as_le_bytes, recv_bytes);

        let recv_value = u16::from_le_bytes(recv_bytes);
        assert_eq!(recv_value, test_counter)
    }
}
