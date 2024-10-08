use crate::byte_rate::{measure::AverageByteRateMeasurer, rate::ByteRate};

use super::Statistics;

/// Statistics that count average byte rate instead counting number of bytes.
pub struct AvgRateStatistics<'clk, Clk>
where
    Clk: crate::clock::Clock,
{
    successful_rate: AverageByteRateMeasurer<'clk, Clk>,
    failed_rate: AverageByteRateMeasurer<'clk, Clk>,
}

impl<'clk, Clk> AvgRateStatistics<'clk, Clk>
where
    Clk: crate::clock::Clock,
{
    pub fn new(clk: &'clk Clk) -> Self {
        Self {
            successful_rate: AverageByteRateMeasurer::new(clk),
            failed_rate: AverageByteRateMeasurer::new(clk),
        }
    }

    // pub fn total_rate(&self) -> Option<ByteRate> {
    //     let success_rate = self.successful_rate.byte_rate()?;
    //     let failed_rate = self.failed_rate.byte_rate()?;

    //     Some(success_rate + failed_rate)
    // }

    pub fn success_rate(&self) -> Option<ByteRate> {
        self.successful_rate.byte_rate()
    }

    pub fn failed_rate(&self) -> Option<ByteRate> {
        self.failed_rate.byte_rate()
    }
}

impl<'clk, Clk> Statistics for AvgRateStatistics<'clk, Clk>
where
    Clk: crate::clock::Clock,
{
    fn add_successful(&mut self, count: usize) {
        self.successful_rate.on_byte(count);
        if !self.failed_rate.is_started() {
            self.failed_rate.start();
        }
    }

    fn add_failed(&mut self, count: usize) {
        self.failed_rate.on_byte(count);
        if !self.successful_rate.is_started() {
            self.successful_rate.start();
        }
    }

    fn reset(&mut self) {
        self.successful_rate.start();
        self.failed_rate.start();
    }
}
