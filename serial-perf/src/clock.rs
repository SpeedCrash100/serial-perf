//!
//! Definition of a clock, timers interfaces used in library
//!

pub use embedded_timers::clock::Clock;
pub use embedded_timers::timer::Timer;

/// A clock based on std::time
#[cfg(feature = "std")]
pub struct StdClock;

#[cfg(feature = "std")]
impl Clock for StdClock {
    type Instant = std::time::Instant;
    fn now(&self) -> Self::Instant {
        std::time::Instant::now()
    }
}
