use crate::{byte_rate::rate::ByteRate, clock::Clock};

enum State<Clk>
where
    Clk: Clock,
{
    Idle,
    Measuring(Clk::Instant, usize),
}

/// A simple byte rate measurer which returns byte rate from the moment it is started
///
/// This measurer starts timer automatically when first byte arrived
pub struct AverageByteRateMeasurer<'clk, Clk>
where
    Clk: Clock,
{
    clk: &'clk Clk,
    state: State<Clk>,
}

impl<'clk, Clk> AverageByteRateMeasurer<'clk, Clk>
where
    Clk: Clock,
{
    /// Create a new measurer with the given clock
    pub fn new(clk: &'clk Clk) -> Self {
        AverageByteRateMeasurer {
            clk,
            state: State::Idle,
        }
    }

    /// Starts or restarts the measurer, resetting all results
    pub fn start(&mut self) {
        let time = self.clk.now();
        self.state = State::Measuring(time, 0)
    }

    /// Handles `amount` of bytes received/sent
    ///
    /// # Note
    /// Starts the timer if not started yet.
    pub fn on_byte(&mut self, amount: usize) {
        if let State::Idle = self.state {
            self.start();
        }

        match self.state {
            State::Idle => unreachable!(),
            State::Measuring(start_time, bytes_sent) => {
                self.state = State::Measuring(start_time, bytes_sent + amount);
            }
        }
    }

    /// Returns the current `ByteRate` if the timer is running
    pub fn byte_rate(&self) -> Option<ByteRate> {
        match self.state {
            State::Idle => None,
            State::Measuring(start_time, bytes_sent) => {
                Some(ByteRate::new(bytes_sent, self.clk.elapsed(start_time)))
            }
        }
    }
}
