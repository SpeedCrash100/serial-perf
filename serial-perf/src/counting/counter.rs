use super::MAX_PACKET_SIZE;

pub trait LeBytes: Sized {
    fn from_slice_checked(slice: &[u8]) -> Option<Self>;
    fn into_packet(self) -> heapless::Vec<u8, MAX_PACKET_SIZE>;
}

impl<const N: usize> LeBytes for [u8; N] {
    fn from_slice_checked(slice: &[u8]) -> Option<Self> {
        if N != slice.len() {
            return None;
        }

        let mut out: Self = [0; N];
        out.copy_from_slice(slice);
        Some(out)
    }

    fn into_packet(self) -> heapless::Vec<u8, MAX_PACKET_SIZE> {
        let mut out = heapless::Vec::new();

        for byte in self {
            out.insert(0, byte).unwrap();
        }
        out.insert(0, 0).unwrap();

        out
    }
}

pub trait Counter: Default {
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
}

macro_rules! impl_counter {
    ($x:ty, $sz:expr) => {
        impl Counter for $x {
            type Bytes = [u8; $sz];

            fn pop(&mut self) -> Self {
                let mut result = *self;
                if result == 0 {
                    result = 1;
                }

                *self = result.wrapping_add(1);

                result
            }

            fn push(&mut self) {
                *self = (*self).wrapping_sub(1);
                if *self == 0 {
                    *self = 1;
                }
            }

            fn distance(&self, value: &Self) -> usize {
                if *self < *value {
                    (value - self) as usize
                } else {
                    // 0 1 2 3 4 5 6 7 8 9
                    //   $     ^---------^
                    //   |         9-4=5
                    //    \ 1 step

                    let to_max_dist = <$x>::MAX - *self;
                    let from_min_dist = *value as usize;
                    to_max_dist as usize + from_min_dist
                }
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
        assert_eq!(pop_value, 5);
        assert_eq!(test_counter, 6);

        test_counter.push();
        assert_eq!(test_counter, 5);
    }

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64"
    ))]
    #[test]
    fn distance() {
        let mut test_counter = 5_u16;
        let pop_value = test_counter.pop();
        assert_eq!(pop_value.distance(&test_counter), 1);
        assert_eq!(test_counter.distance(&pop_value), u16::MAX as usize - 1);
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
        let mut as_data_queue = as_le_bytes.into_packet();
        assert_eq!(as_data_queue.len(), 2 + 1); // +1 for null terminator

        let mut recv_side = heapless::Vec::<u8, MAX_PACKET_SIZE>::new();
        for _ in 0..2 {
            recv_side.push(as_data_queue.pop().unwrap()).unwrap();
        }

        let recv_bytes = <u16 as Counter>::Bytes::from_slice_checked(&recv_side)
            .expect("failed to create from slice");

        assert_eq!(as_le_bytes, recv_bytes);

        let recv_value = u16::from_le_bytes(recv_bytes);
        assert_eq!(recv_value, test_counter)
    }
}
