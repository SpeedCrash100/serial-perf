//!
//! Struct for storing the statistics of a TX/RX paths(total bytes, errors, etc)
//!

#[derive(Debug, Default)]
pub struct Statistics {
    /// Number of packets that were successfully sent/received
    successful: usize,

    /// Number of packets that were not sent/received
    failed: usize,
}

impl Statistics {
    /// Returns the total number of packets sent/received
    pub fn total(&self) -> usize {
        self.successful + self.failed
    }

    /// Returns the number of packets that were successfully sent/received
    pub fn successful(&self) -> usize {
        self.successful
    }

    /// Returns the number of packets that were not sent/received or rejected when received
    pub fn failed(&self) -> usize {
        self.failed
    }

    /// Resets the statistics to initial state
    pub fn reset(&mut self) {
        self.successful = 0;
        self.failed = 0;
    }

    pub fn add_successful(&mut self, count: usize) {
        self.successful += count;
    }

    pub fn add_failed(&mut self, count: usize) {
        self.failed += count;
    }
}
