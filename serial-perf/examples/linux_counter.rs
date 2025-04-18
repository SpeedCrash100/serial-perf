use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};
use embedded_hal_nb::serial::Read;
use linux_embedded_hal::Serial;
use serial_perf::{
    byte_rate::{
        limit::{ByteRateSerialLimiter, PollingByteRateLimiter},
        rate::ByteRate,
    },
    clock::StdClock,
    counting::Counting,
    statistics::{CountingStatistics, IntervalRateStatistics},
};

const PRINT_INTERVAL_MS: u64 = 5000;

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

    #[clap(long, default_value_t = Mode::Double)]
    mode: Mode,
}

impl CommonArgs {
    pub fn create_serial(&self) -> Serial {
        Serial::open(self.port.clone(), self.baud_rate).expect("failed to create serial")
    }
}

fn main() -> anyhow::Result<()> {
    let args = CommonArgs::parse();

    let clock = StdClock;
    let rate_limit = ByteRate::new(
        args.byte_limit,
        Duration::from_micros(args.byte_limit_interval_us as u64),
    );
    let rate_limiter = PollingByteRateLimiter::new(rate_limit, &clock);

    let mut serial = args.create_serial();
    while !matches!(serial.read(), Err(nb::Error::WouldBlock)) {
        // Linux can return some data from previous run, resulting in high loss value
        // The old counter returns 9000, the new one 1 which is 9000-> u16::MAX + 1 -> 9000 -> About a u16::max value value loss packages
    }

    let limited_serial = ByteRateSerialLimiter::new(serial, rate_limiter);
    let mut counter = Counting::<_, u64, _, _, _>::new(
        limited_serial,
        IntervalRateStatistics::new(&clock, Duration::from_millis(PRINT_INTERVAL_MS)),
        IntervalRateStatistics::new(&clock, Duration::from_millis(PRINT_INTERVAL_MS)),
        CountingStatistics::default(),
    );

    let mut last_print = Instant::now();

    println!("Start loop");
    loop {
        match args.mode {
            Mode::Client => {
                nb::block!(counter.send_nb())?;
            }

            Mode::Server => {
                nb::block!(counter.recv_nb())?;
            }

            Mode::Double => {
                nb::block!(counter.loop_nb())?;
            }
        }

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
