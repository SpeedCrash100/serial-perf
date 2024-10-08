use core::time::Duration;

use crate::byte_rate::{measure::IntervalByteRateMeasurer, rate::ByteRate};

use super::Statistics;

/// Statistics that count average byte rate instead counting number of bytes.
pub struct IntervalRateStatistics<'clk, Clk>
where
    Clk: crate::clock::Clock,
{
    successful_rate: IntervalByteRateMeasurer<'clk, Clk>,
    failed_rate: IntervalByteRateMeasurer<'clk, Clk>,
}

impl<'clk, Clk> IntervalRateStatistics<'clk, Clk>
where
    Clk: crate::clock::Clock,
{
    pub fn new(clk: &'clk Clk, interval: Duration) -> Self {
        Self {
            successful_rate: IntervalByteRateMeasurer::new(clk, interval),
            failed_rate: IntervalByteRateMeasurer::new(clk, interval),
        }
    }

    // pub fn total_rate(&self) -> Option<ByteRate> {
    //     let success_rate = self.successful_rate.byte_rate()?;
    //     let failed_rate = self.failed_rate.byte_rate()?;

    //     Some(success_rate + failed_rate)
    // }

    pub fn success_rate(&self) -> &ByteRate {
        self.successful_rate.byte_rate()
    }

    pub fn failed_rate(&self) -> &ByteRate {
        self.failed_rate.byte_rate()
    }
}

impl<'clk, Clk> Statistics for IntervalRateStatistics<'clk, Clk>
where
    Clk: crate::clock::Clock,
{
    fn add_successful(&mut self, count: usize) {
        self.successful_rate.on_byte(count);
    }

    fn add_failed(&mut self, count: usize) {
        self.failed_rate.on_byte(count);
    }

    fn reset(&mut self) {
        self.successful_rate.reset();
        self.failed_rate.reset();
    }
}
