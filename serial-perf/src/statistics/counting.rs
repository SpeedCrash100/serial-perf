use super::Statistics;

/// Counting is a simple statistics for count the number of packets sent/received successfully, failed.
#[derive(Debug, Default)]
pub struct CountingStatistics {
    /// Number of packets that were successfully sent/received
    successful: usize,

    /// Number of packets that were not sent/received
    failed: usize,
}

impl CountingStatistics {
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
}

impl Statistics for CountingStatistics {
    fn add_failed(&mut self, count: usize) {
        self.failed = self.failed.saturating_add(count);
    }

    fn add_successful(&mut self, count: usize) {
        self.successful = self.successful.saturating_add(count);
    }

    fn reset(&mut self) {
        self.failed = 0;
        self.successful = 0;
    }
}
