use embedded_timers::clock::Clock;

use super::PollingByteRateLimiter;

mod nb;

/// A wrapper around embedded-hal serial that will stop sending data above specified byte rate limit
pub struct ByteRateSerialLimiter<'clock, Clk, Serial>
where
    Clk: Clock,
{
    rate_limit: PollingByteRateLimiter<'clock, Clk>,
    serial: Serial,
}

impl<'clock, Clk, Serial> ByteRateSerialLimiter<'clock, Clk, Serial>
where
    Clk: Clock,
{
    pub fn new(serial: Serial, rate_limit: PollingByteRateLimiter<'clock, Clk>) -> Self {
        Self { rate_limit, serial }
    }
}
