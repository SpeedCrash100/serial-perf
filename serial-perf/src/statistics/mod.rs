//!
//! Struct for storing the statistics of a TX/RX paths(total bytes, errors, etc)
//!

mod counting;
pub use counting::CountingStatistics;

/// Trait for capturing statistics,
/// resetting is done by the creating new instance with Default::default() and replacing old one
pub trait Statistics: Default {
    /// Adds `count` successful packets to the statistics
    fn add_successful(&mut self, count: usize);

    /// Adds `count` failed packets to the statistics
    fn add_failed(&mut self, count: usize);
}
