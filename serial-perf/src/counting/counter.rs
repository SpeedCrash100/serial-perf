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

    fn filled() -> Self;
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

    fn filled() -> Self {
        [0xFF; N]
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

    /// Normalize the counter(value only with non-zero bytes) to normal number [0..]
    fn normalize(&self) -> Option<Self>;
    /// Converts a value into a counter. Reverse of `normalize`.
    fn to_counter_value(self) -> Option<Self>;

    fn min_counter() -> Self {
        let ones = Self::Bytes::ones();
        Self::from_le_bytes(ones)
    }

    fn max_counter() -> Self {
        let filled = Self::Bytes::filled();
        Self::from_le_bytes(filled)
    }

    fn max_normalized() -> Self {
        Self::max_counter().normalize().unwrap()
    }
}

macro_rules! impl_counter {
    ($x:ty, $sz:expr) => {
        impl Counter for $x {
            type Bytes = [u8; $sz];

            fn pop(&mut self) -> Self {
                if self.normalize().is_none() {
                    *self = Self::min_counter();
                }

                let out_value = *self;

                // Safe: We checked above and the counter is valid.
                let mut normalized = unsafe { self.normalize().unwrap_unchecked() };
                if normalized == Self::max_normalized() {
                    normalized = 0;
                } else {
                    normalized += 1;
                }

                // Safe: We ensured that we in the valid range above.
                *self = unsafe { normalized.to_counter_value().unwrap_unchecked() };
                out_value
            }

            fn push(&mut self) {
                if self.normalize().is_none() {
                    *self = Self::min_counter();
                }

                // Safe: We checked above and the counter is valid.
                let mut normalized = unsafe { self.normalize().unwrap_unchecked() };
                if normalized == 0 {
                    normalized = Self::max_normalized();
                } else {
                    normalized -= 1;
                }

                // Safe: We ensured that we in the valid range above.
                *self = unsafe { normalized.to_counter_value().unwrap_unchecked() };
            }

            fn distance(&self, value: &Self) -> usize {
                let normalized_left = self
                    .normalize()
                    .expect("The left operand of distance is not a Counter");
                let normalized_right = value
                    .normalize()
                    .expect("The right operand of distance is not a Counter");

                if normalized_left <= normalized_right {
                    return normalized_right as usize - normalized_left as usize;
                }

                let to_max = Self::max_normalized() - normalized_left;
                let from_min = normalized_right /*- 0*/ + 1;

                to_max as usize + from_min as usize
            }

            fn to_le_bytes(&self) -> Self::Bytes {
                Self::to_le_bytes(*self)
            }

            fn from_le_bytes(bytes: Self::Bytes) -> Self {
                Self::from_le_bytes(bytes)
            }

            fn normalize(&self) -> Option<Self> {
                let mut out_value = 0;

                let bytes = self.to_le_bytes();
                for (pos, byte) in bytes.iter().copied().enumerate() {
                    if byte < 1 {
                        return None;
                    }
                    out_value += (byte as $x - 1) * (255 as $x).pow(pos as u32);
                }

                Some(out_value)
            }

            fn to_counter_value(self) -> Option<Self> {
                if Self::max_normalized() < self {
                    return None;
                }

                let mut out_value = 0;

                let mut cur_value = self;
                for i in 0..$sz {
                    let new_reminder = cur_value % 255; // is the amount of possible values in [1..255]
                    cur_value /= 255;

                    out_value += (new_reminder + 1) << (8 * i);
                }

                debug_assert_eq!(0, cur_value);

                Some(out_value)
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

    /// Test that incrementing a decrementing is working
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

    /// Check tha distance between value before and after increment is equal to 1
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
    }

    /// Check tha distance between value before and after increment is equal to 1, if the counter reaches the maximum value
    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    ))]
    #[test]
    fn distance_overflow() {
        let mut test_counter = 0xFFFF_u16;
        let pop_value = test_counter.pop();
        assert_eq!(pop_value.distance(&test_counter), 1);
    }

    /// Checks amount of values between max and min counter for u8
    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    ))]
    #[test]
    fn interval_u8() {
        let max_counter = 0xFF_u8;
        let min_counter = 0x01_u8;

        let distance = min_counter.distance(&max_counter);
        assert_eq!(distance, u8::max_normalized() as usize);
    }

    /// Checks amount of values between max and min counter for u16
    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    ))]
    #[test]
    fn interval_u16() {
        let max_counter = 0xFFFF_u16;
        let min_counter = 0x0101_u16;

        let distance = min_counter.distance(&max_counter);
        assert_eq!(distance, u16::max_normalized() as usize);
    }

    /// Checks amount of values between max and min counter for u32
    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
    #[test]
    fn interval_u32() {
        let max_counter = 0xFFFFFFFF_u32;
        let min_counter = 0x01010101_u32;

        let distance = min_counter.distance(&max_counter);
        assert_eq!(distance, u32::max_normalized() as usize);
    }

    /// Checks amount of values between max and min counter for u64
    #[cfg(target_pointer_width = "64")]
    #[test]
    fn interval_u64() {
        let max_counter = 0xFFFFFFFF_FFFFFFFF_u64;
        let min_counter = 0x01010101_01010101_u64;

        let distance = min_counter.distance(&max_counter);
        assert_eq!(distance, u64::max_normalized() as usize);
    }

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    ))]
    #[test]
    fn distance_u8() {
        let mut test_counter = 0x01_u8;
        for _ in 0..255 {
            let pop_value = test_counter.pop();
            assert_eq!(pop_value.distance(&test_counter), 1);
            assert_eq!(test_counter.distance(&pop_value), 254);
            test_counter = pop_value;
        }
    }

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    ))]
    #[test]
    fn distance_u16() {
        let mut test_counter = 0x0101_u16;
        for _ in 0..65025 {
            let pop_value = test_counter.pop();
            assert_eq!(pop_value.distance(&test_counter), 1);
        }
    }

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    ))]
    #[test]
    fn distance_u16_big_diff() {
        let small = 0x0101_u16;
        let big = 0x0201_u16;

        assert_eq!(small.distance(&big), 255);
    }

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    ))]
    #[test]
    fn distance_overflow_u8() {
        let mut test_counter = 0xFF_u8;
        let pop_value = test_counter.pop();
        assert_eq!(pop_value.distance(&test_counter), 1);
        assert_eq!(test_counter.distance(&pop_value), 254);
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

    // #[cfg(any(
    //     target_pointer_width = "16",
    //     target_pointer_width = "32",
    //     target_pointer_width = "64"
    // ))]
    // #[test]
    // fn print_u16() {
    //     let mut test_counter = 0_u16;

    //     for i in 0..65535 {
    //         let pop_value = test_counter.pop();
    //         println!("{}\t: {:04x}", i, pop_value);
    //     }
    // }
}
