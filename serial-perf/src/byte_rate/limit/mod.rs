//!
//! Structs for limiting the byte rate
//!

mod polling;
pub use polling::PollingByteRateLimiter;

mod limited_serial;
pub use limited_serial::ByteRateSerialLimiter;
