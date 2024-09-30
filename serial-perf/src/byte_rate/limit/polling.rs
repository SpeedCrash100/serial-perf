use core::time::Duration;

use embedded_timers::instant::Instant;

use crate::byte_rate::rate::ByteRate;
use crate::clock::{Clock, Timer, TimerError};

enum State {
    Idle,
    Running(usize),
    Limiting,
    Unlimited,
}

/// Polling byte rate limiter
pub struct PollingByteRateLimiter<'clk, Clk>
where
    Clk: Clock,
{
    max_rate: ByteRate,
    state: State,

    clock: &'clk Clk,
    timer: Timer<'clk, Clk>,
    timer_end_time: Clk::Instant,
}

impl<'clk, Clk> PollingByteRateLimiter<'clk, Clk>
where
    Clk: Clock,
{
    /// Creates new rate limiter
    pub fn new(max_rate: ByteRate, clock: &'clk Clk) -> Self {
        let mut out = Self {
            max_rate: ByteRate::new(0, Duration::ZERO),
            state: State::Unlimited,
            clock,
            timer: Timer::new(clock),
            timer_end_time: clock.now(),
        };

        out.set_byte_rate(max_rate);

        out
    }

    /// Sets new byte rate and resets the limiter to initial state
    pub fn set_byte_rate(&mut self, max_rate: ByteRate) {
        self.state = if max_rate.interval().is_zero() {
            State::Unlimited
        } else if max_rate.bytes() == 0 {
            State::Limiting
        } else {
            State::Idle
        };

        self.max_rate = max_rate;
        self.timer = Timer::new(self.clock);
        self.timer_end_time = self.clock.now();
    }

    /// Check if sending is possible in current interval but doesn't assume you will send byte if it is true
    ///
    /// Use `send` to notify limiter about send
    pub fn can_send(&self) -> bool {
        match self.state {
            State::Idle => true,
            State::Unlimited => true,
            State::Running(n) if n > 0 => true,
            State::Running(_) => self.timer_expired(),
            State::Limiting if self.max_rate.bytes() == 0 => false,
            State::Limiting => self.timer_expired(),
        }
    }

    /// Notify that you have sent byte successfully, returns true if limit NOT reached yet or false otherwise
    ///
    /// Always check for `can_send` before otherwise it send will do nothing if you try to send more than allowed.
    /// Can return overflow error if restarting timer was not successful
    pub fn send(&mut self) -> Result<bool, TimerError> {
        match self.state {
            State::Idle => self.send_idle(),
            State::Unlimited => Ok(true),
            State::Running(remain) => self.send_running(remain),
            State::Limiting => self.send_limiting(),
        }
    }

    /// Forcefully restart the limiter from current time point
    pub fn restart(&mut self) -> Result<(), TimerError> {
        let new_duration = self.fit_timer_duration()?;
        self.timer.try_start(new_duration)?;

        self.state = State::Running(self.max_rate.bytes());

        Ok(())
    }

    /// Gets time until timer resets and new bytes can be send
    /// Returns None if limiter is unlimited or duration cannot be found out(timer not started)
    pub fn duration_until_reset(&self) -> Option<Duration> {
        if let State::Unlimited = self.state {
            return None;
        }

        self.timer.duration_left().ok()
    }

    fn send_idle(&mut self) -> Result<bool, TimerError> {
        let now = self.clock.now();
        self.timer.try_start(*self.max_rate.interval())?;
        self.timer_end_time = now
            .checked_add(*self.max_rate.interval())
            .ok_or(TimerError::Overflow)?;

        self.send_running(self.max_rate.bytes())
    }

    fn send_running(&mut self, remaining: usize) -> Result<bool, TimerError> {
        if self.timer_expired() {
            self.restart()?;
            return self.send();
        }

        let mut limit_reached = false;

        self.state = if remaining > 1 {
            State::Running(remaining - 1)
        } else {
            limit_reached = true;
            State::Limiting
        };

        Ok(!limit_reached)
    }

    fn send_limiting(&mut self) -> Result<bool, TimerError> {
        if self.max_rate.bytes() == 0 {
            return Ok(false);
        }

        if self.timer_expired() {
            self.restart()?;
            self.send()
        } else {
            Ok(false)
        }
    }

    fn timer_expired(&self) -> bool {
        self.timer.is_expired().expect("timer malfunction")
    }

    fn fit_timer_duration(&mut self) -> Result<Duration, TimerError> {
        let now = self.clock.now();
        let duration = *self.max_rate.interval();

        while self.timer_end_time < now {
            self.timer_end_time = self
                .timer_end_time
                .checked_add(duration)
                .ok_or(TimerError::Overflow)?;
        }

        Ok(self.timer_end_time.duration_since(now))
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use core::time::Duration;

    use crate::{byte_rate::rate::ByteRate, clock::StdClock};

    use super::PollingByteRateLimiter;

    #[test]
    fn unlimited() {
        let clock = StdClock;
        let max_rate = ByteRate::new(10, Duration::ZERO);
        let mut limiter = PollingByteRateLimiter::new(max_rate, &clock);

        const COUNT: usize = 1_000_000;
        for _ in 0..COUNT {
            assert!(limiter.send().unwrap())
        }
    }

    #[test]
    fn limited() {
        let clock = StdClock;
        let max_rate = ByteRate::new(0, Duration::from_secs(1));
        let limiter = PollingByteRateLimiter::new(max_rate, &clock);

        assert!(!limiter.can_send())
    }

    #[test]
    fn limit_activated() {
        const LIMIT: usize = 10;

        let clock = StdClock;
        let max_rate = ByteRate::new(LIMIT, Duration::from_secs(1));
        let mut limiter = PollingByteRateLimiter::new(max_rate, &clock);

        // First LIMIT-1 bytes should be allowed to send and limit not reached
        for _ in 0..(LIMIT - 1) {
            assert!(limiter.send().unwrap());
            assert!(limiter.can_send());
        }

        // Last byte should reach the limit
        assert!(!limiter.send().unwrap());
        assert!(!limiter.can_send());
    }

    #[test]
    fn restart_resets_limit() {
        const LIMIT: usize = 10;

        let clock = StdClock;
        let max_rate = ByteRate::new(LIMIT, Duration::from_secs(1));
        let mut limiter = PollingByteRateLimiter::new(max_rate, &clock);

        // First LIMIT-1 bytes should be allowed to send and limit not reached
        for _ in 0..(LIMIT - 1) {
            assert!(limiter.send().unwrap());
            assert!(limiter.can_send());
        }

        limiter.restart().unwrap();

        // Limit reset, we should be able to send new bytes
        for _ in 0..(LIMIT - 1) {
            assert!(limiter.send().unwrap());
            assert!(limiter.can_send());
        }

        // Last byte should reach the limit
        assert!(!limiter.send().unwrap());
        assert!(!limiter.can_send());
    }

    #[test]
    fn restart_on_timer() {
        const LIMIT: usize = 10;

        let clock = StdClock;
        let max_rate = ByteRate::new(LIMIT, Duration::from_secs(1));
        let mut limiter = PollingByteRateLimiter::new(max_rate, &clock);

        // First LIMIT-1 bytes should be allowed to send and limit not reached
        for _ in 0..(LIMIT - 1) {
            assert!(limiter.send().unwrap());
            assert!(limiter.can_send());
        }

        std::thread::sleep(limiter.duration_until_reset().unwrap());

        // Limit reset, we should be able to send new bytes
        for _ in 0..(LIMIT - 1) {
            assert!(limiter.send().unwrap());
            assert!(limiter.can_send());
        }

        // Last byte should reach the limit
        assert!(!limiter.send().unwrap());
        assert!(!limiter.can_send());
    }
}
