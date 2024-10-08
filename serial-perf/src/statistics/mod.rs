//!
//! Struct for storing the statistics of a TX/RX paths(total bytes, errors, etc)
//!

mod dummy;
pub use dummy::DummyStatistics;

mod counting;
pub use counting::CountingStatistics;

mod avg_rate;
pub use avg_rate::AvgRateStatistics;

/// Trait for capturing statistics,
pub trait Statistics {
    /// Adds `count` successful packets to the statistics
    fn add_successful(&mut self, count: usize);

    /// Adds `count` failed packets to the statistics
    fn add_failed(&mut self, count: usize);

    /// Resets all stats in this struct.
    fn reset(&mut self);
}
