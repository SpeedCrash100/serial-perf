//!
//! Structs for measuring byte rate
//!

mod avg;
pub use avg::AverageByteRateMeasurer;

mod interval;
pub use interval::IntervalByteRateMeasurer;
