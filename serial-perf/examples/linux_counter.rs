use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};
use linux_embedded_hal::{Serial, SerialError};
use serial_perf::{
    byte_rate::{
        limit::{ByteRateSerialLimiter, PollingByteRateLimiter},
        rate::ByteRate,
    },
    clock::StdClock,
    counting::{prelude::*, Counting},
    statistics::{CountingStatistics, IntervalRateStatistics},
};

const PRINT_INTERVAL_MS: u64 = 5000;

/// Global clock source for the application
static CLOCK: StdClock = StdClock;

#[derive(Debug, Clone, ValueEnum)]
pub enum Mode {
    Client,
    Server,
    Double,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Client => write!(f, "client"),
            Self::Server => write!(f, "server"),
            Self::Double => write!(f, "double"),
        }
    }
}

#[derive(Parser)]
pub struct CommonArgs {
    /// The port to connect to.
    port: String,

    /// Baud rate for serial
    #[clap(short, long, default_value_t = 115200)]
    baud_rate: u32,

    /// Byte rate limit per specified time
    #[clap(long, default_value_t = 11520)]
    byte_limit: usize,

    /// Time for byte rate limit if zero - unlimited
    #[clap(long, default_value_t = 0)]
    byte_limit_interval_us: usize,

    /// Warm up time before test starts. Allows to clear data from previous runs
    #[clap(long, default_value_t = 5000)]
    warm_up_time_ms: u32,

    #[clap(long, default_value_t = Mode::Double)]
    mode: Mode,
}

impl CommonArgs {
    fn create_serial(&self) -> Serial {
        Serial::open(self.port.clone(), self.baud_rate).expect("failed to create serial")
    }

    fn create_counting_test(&self) -> impl AppCounting {
        let rate_limit = ByteRate::new(
            self.byte_limit,
            Duration::from_micros(self.byte_limit_interval_us as u64),
        );
        let rate_limiter = PollingByteRateLimiter::new(rate_limit, &CLOCK);

        let serial = self.create_serial();

        let limited_serial = ByteRateSerialLimiter::new(serial, rate_limiter);
        let counter = Counting::<_, u64, _, _, _>::new(
            limited_serial,
            IntervalRateStatistics::new(&CLOCK, Duration::from_millis(PRINT_INTERVAL_MS)),
            IntervalRateStatistics::new(&CLOCK, Duration::from_millis(PRINT_INTERVAL_MS)),
            CountingStatistics::default(),
        );

        counter
    }
}

trait AppCounting:
    ValidCountingNb<
    Error = SerialError,
    TxStats = IntervalRateStatistics<'static, StdClock>,
    RxStats = IntervalRateStatistics<'static, StdClock>,
    LossStats = CountingStatistics,
>
{
    fn tick_io(&mut self, mode: &Mode) -> Result<(), SerialError> {
        match mode {
            Mode::Client => {
                nb::block!(self.send_nb())?;
            }

            Mode::Server => {
                nb::block!(self.recv_nb())?;
            }

            Mode::Double => {
                nb::block!(self.loop_nb())?;
            }
        }

        Ok(())
    }

    fn warm_up(&mut self, args: &CommonArgs) -> anyhow::Result<()> {
        let warm_up_duration = Duration::from_millis(args.warm_up_time_ms as u64);
        println!("Warm up {:.3} seconds...", warm_up_duration.as_secs_f64());

        let start = Instant::now();
        let end = start + warm_up_duration;

        while Instant::now() < end {
            match self.recv_nb() {
                Ok(_) => {}
                Err(nb::Error::WouldBlock) => {}
                Err(nb::Error::Other(err)) => {
                    return Err(err.into());
                }
            }
        }

        self.reset();

        Ok(())
    }
}

impl<T> AppCounting for T where
    T: ValidCountingNb<
        Error = SerialError,
        TxStats = IntervalRateStatistics<'static, StdClock>,
        RxStats = IntervalRateStatistics<'static, StdClock>,
        LossStats = CountingStatistics,
    >
{
}

fn main() -> anyhow::Result<()> {
    let args = CommonArgs::parse();

    let mut counter = args.create_counting_test();

    counter.warm_up(&args)?;

    let mut last_print = Instant::now();

    println!("Test started");
    loop {
        counter.tick_io(&args.mode)?;

        if Duration::from_millis(PRINT_INTERVAL_MS) < last_print.elapsed() {
            if matches!(args.mode, Mode::Client | Mode::Double) {
                println!(
                    "TX(bytes): sent: {} B/s, errors: {} B/s",
                    counter
                        .tx_stats()
                        .success_rate()
                        .bytes_per_second_f64()
                        .unwrap_or(0.0),
                    counter
                        .tx_stats()
                        .failed_rate()
                        .bytes_per_second_f64()
                        .unwrap_or(0.0)
                );
            }

            if matches!(args.mode, Mode::Server | Mode::Double) {
                println!(
                    "RX(bytes): success: {} B/s, errors: {} B/s",
                    counter
                        .rx_stats()
                        .success_rate()
                        .bytes_per_second_f64()
                        .unwrap_or(0.0),
                    counter
                        .rx_stats()
                        .failed_rate()
                        .bytes_per_second_f64()
                        .unwrap_or(0.0)
                );

                if counter.loss_stats().total() != 0 {
                    println!(
                        "RX(packet): loss: {}, total: {}, {:.02}%",
                        counter.loss_stats().failed(),
                        counter.loss_stats().total(),
                        (counter.loss_stats().failed() * 10000 / counter.loss_stats().total())
                            as f64
                            / 100.0
                    );
                }
            }

            last_print = Instant::now();
        }
    }
}
