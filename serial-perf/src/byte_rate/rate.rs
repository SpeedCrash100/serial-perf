use core::time::Duration;

/// Holds a data needed to calculate the byte rate.
pub struct ByteRate {
    bytes: usize,
    interval: Duration,
}

impl ByteRate {
    /// Creates a byte rate from amount of bytes passed over specified interval
    ///
    pub fn new(bytes: usize, interval: Duration) -> Self {
        Self { bytes, interval }
    }

    /// Return amount of bytes passed over interval
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    /// Set amount of bytes passed over interval
    pub fn set_bytes(&mut self, bytes: usize) {
        self.bytes = bytes;
    }

    /// Increment amount of bytes passed over interval
    pub fn incr_bytes(&mut self) {
        self.bytes += 1;
    }

    /// Return interval between measurements
    pub fn interval(&self) -> &Duration {
        &self.interval
    }

    /// Set interval between measurements
    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    /// Calculates amount of bytes passed over seconds, floor value.
    /// Returns `None` when interval is zero or overflow occurred.
    ///
    /// # Note
    /// The method will prefer to use the most accurate type for calculation starting
    /// from nanoseconds and ending with seconds. That means, if you get 1 byte over 100 ms,
    /// it will return 10 bytes per second instead of 0 bytes per second because it will use nanoseconds internally.
    /// If you need stable values use `bytes_per_second_*` variants
    ///
    pub fn bytes_per_second(&self) -> Option<usize> {
        if self.interval.is_zero() {
            return None;
        }

        if let Some(result_ns) = self.bytes_per_second_ns_accuracy() {
            return Some(result_ns);
        }

        if let Some(result_us) = self.bytes_per_second_us_accuracy() {
            return Some(result_us);
        }

        if let Some(result_ms) = self.bytes_per_second_ms_accuracy() {
            return Some(result_ms);
        }

        self.bytes_per_second_sec_accuracy()
    }

    /// Calculates amount of bytes passed over seconds, floor value.
    /// Returns `None` when interval is zero or below 1 second.
    pub fn bytes_per_second_sec_accuracy(&self) -> Option<usize> {
        if self.interval.is_zero() {
            return None;
        }

        let sec = usize::try_from(self.interval.as_secs()).ok()?;
        if sec == 0 {
            return None;
        }

        Some(self.bytes / sec)
    }

    /// Calculates amount of bytes passed over milliseconds, floor value and converts to bytes per second
    /// Returns `None` when interval is below 1 ms or calculation overflowed
    pub fn bytes_per_second_ms_accuracy(&self) -> Option<usize> {
        if self.interval.is_zero() {
            return None;
        }

        let ms = usize::try_from(self.interval.as_millis()).ok()?;
        if ms == 0 {
            return None;
        }

        let bytes_ms = self.bytes.checked_mul(1_000)?;

        Some(bytes_ms / ms)
    }

    /// Calculates amount of bytes passed over microseconds, floor value and converts to bytes per second
    /// Returns `None` when interval is below 1 us or calculation overflowed
    pub fn bytes_per_second_us_accuracy(&self) -> Option<usize> {
        if self.interval.is_zero() {
            return None;
        }

        let us = usize::try_from(self.interval.as_micros()).ok()?;
        if us == 0 {
            return None;
        }

        let bytes_us = self.bytes.checked_mul(1_000_000)?;

        Some(bytes_us / us)
    }

    /// Calculates amount of bytes passed over nanoseconds, floor value and converts to bytes per second
    /// Returns `None` when interval is below 1 ns or calculation overflowed
    pub fn bytes_per_second_ns_accuracy(&self) -> Option<usize> {
        if self.interval.is_zero() {
            return None;
        }

        let ns = usize::try_from(self.interval.as_nanos()).ok()?;
        let bytes_ns = self.bytes.checked_mul(1_000_000_000)?;

        Some(bytes_ns / ns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation() {
        let rate = ByteRate::new(146, Duration::from_secs(2));
        assert_eq!(rate.bytes(), 146);
        assert_eq!(rate.interval().as_secs(), 2);
    }

    #[test]
    fn bytes_per_second_sec_accuracy_whole() {
        let rate = ByteRate::new(146, Duration::from_secs(2));
        let rate_per_sec = rate.bytes_per_second_sec_accuracy();
        assert!(rate_per_sec.is_some());
        assert_eq!(rate_per_sec.unwrap(), 73);
    }

    #[test]
    fn bytes_per_second_sec_accuracy_reminder() {
        let rate = ByteRate::new(73, Duration::from_secs(2));
        let rate_per_sec = rate.bytes_per_second_sec_accuracy();
        assert!(rate_per_sec.is_some());
        assert_eq!(rate_per_sec.unwrap(), 36);
    }

    #[test]
    fn bytes_per_second_auto() {
        let rate = ByteRate::new(146, Duration::from_millis(500));
        let rate_per_sec = rate.bytes_per_second();
        assert!(rate_per_sec.is_some());
        assert_eq!(rate_per_sec.unwrap(), 292);
    }

    #[test]
    fn bytes_per_second_zero() {
        let rate = ByteRate::new(146, Duration::ZERO);
        let rate_per_sec = rate.bytes_per_second();
        assert!(rate_per_sec.is_none());
    }

    #[test]
    fn bytes_per_second_sec_accuracy_below_1_sec() {
        let rate = ByteRate::new(146, Duration::from_millis(250));
        let rate_per_sec = rate.bytes_per_second_sec_accuracy();
        assert!(rate_per_sec.is_none());
    }

    #[test]
    fn bytes_per_second_ms_accuracy_whole() {
        let rate = ByteRate::new(146, Duration::from_millis(250));
        let rate_per_sec = rate.bytes_per_second_ms_accuracy();
        assert!(rate_per_sec.is_some());
        assert_eq!(rate_per_sec.unwrap(), 146 * 4);
    }

    #[test]
    fn bytes_per_second_us_accuracy_whole() {
        let rate = ByteRate::new(146, Duration::from_micros(250));
        let rate_per_sec = rate.bytes_per_second_us_accuracy();
        assert!(rate_per_sec.is_some());
        assert_eq!(rate_per_sec.unwrap(), 146 * 4000);
    }

    #[test]
    fn bytes_per_second_ns_accuracy_whole() {
        let rate = ByteRate::new(146, Duration::from_nanos(250));
        let rate_per_sec = rate.bytes_per_second_ns_accuracy();
        assert!(rate_per_sec.is_some());
        assert_eq!(rate_per_sec.unwrap(), 146 * 4_000_000);
    }

    #[test]
    fn bytes_per_second_ns_overflow() {
        let rate = ByteRate::new(usize::MAX, Duration::from_nanos(250));
        let rate_per_sec = rate.bytes_per_second_ns_accuracy();
        assert!(rate_per_sec.is_none());
    }

    #[test]
    fn bytes_per_second_auto_overflow() {
        let rate = ByteRate::new(usize::MAX / 2, Duration::from_secs(2));
        let rate_per_sec = rate.bytes_per_second();
        assert!(rate_per_sec.is_some());
        assert_eq!(rate_per_sec.unwrap(), usize::MAX / 4);
    }
}
