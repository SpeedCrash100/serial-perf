use core::time::Duration;

use embedded_timers::instant::Instant;

use crate::{byte_rate::rate::ByteRate, clock::Clock, clock::Timer, clock::TimerError};

/// Measurers byte rate of a stream of bytes with specified intervals between resets and starts again
///
/// # Note
/// You should not set intervals to be too small or byte rate will changed with big steps.  [ 0, 0, 1000000, 0, ... ] for example, because byte received in 1 interval
pub struct IntervalByteRateMeasurer<'clk, Clk>
where
    Clk: Clock,
{
    current_rate: ByteRate,
    output_rate: ByteRate,

    clock: &'clk Clk,
    timer: Timer<'clk, Clk>,
    timer_end_time: Clk::Instant,
}

impl<'clk, Clk> IntervalByteRateMeasurer<'clk, Clk>
where
    Clk: Clock,
{
    /// Create a new measurer with the given clock
    pub fn new(clk: &'clk Clk, interval: Duration) -> Self {
        let rate = ByteRate::new(0, interval);

        let mut timer = Timer::new(clk);
        timer.try_start(interval).ok();

        Self {
            clock: clk,
            current_rate: rate.clone(),
            output_rate: rate,
            timer,
            timer_end_time: clk.now(),
        }
    }

    /// Starts or restarts the measurer, resetting all results
    pub fn reset(&mut self) {
        self.current_rate.set_bytes(0);
        self.output_rate = self.current_rate.clone();
        self.timer_end_time = self.clock.now();
    }

    /// Handles `amount` of bytes received/sent
    ///
    /// # Note
    /// Starts the timer if not started yet.
    pub fn on_byte(&mut self, amount: usize) {
        if self.timer.is_expired().unwrap_or(true) {
            self.output_rate = self.current_rate.clone();
            self.current_rate.set_bytes(0);

            self.restart().ok();
        }

        let current_bytes = self.current_rate.bytes();
        self.current_rate.set_bytes(current_bytes + amount);
    }

    /// Returns the current `ByteRate` if the timer is running
    pub fn byte_rate(&self) -> &ByteRate {
        &self.output_rate
    }

    /// Forcefully restart the measurer from current time point
    pub fn restart(&mut self) -> Result<(), TimerError> {
        let new_duration = self.fit_timer_duration()?;
        self.timer.try_start(new_duration)?;

        Ok(())
    }

    fn fit_timer_duration(&mut self) -> Result<Duration, TimerError> {
        let now = self.clock.now();
        let duration = *self.current_rate.interval();

        while self.timer_end_time < now {
            self.timer_end_time = self
                .timer_end_time
                .checked_add(duration)
                .ok_or(TimerError::Overflow)?;
        }

        Ok(self.timer_end_time.duration_since(now))
    }
}
