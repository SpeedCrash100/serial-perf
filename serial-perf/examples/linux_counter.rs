use std::time::{Duration, Instant};

use clap::Parser;
use linux_embedded_hal::Serial;
use serial_perf::{
    byte_rate::{
        limit::{ByteRateSerialLimiter, PollingByteRateLimiter},
        rate::ByteRate,
    },
    clock::StdClock,
    counting::Counting,
    statistics::CountingStatistics,
};

const PRINT_INTERVAL_MS: u64 = 5000;

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

    let serial = args.create_serial();
    let limited_serial = ByteRateSerialLimiter::new(serial, rate_limiter);
    let mut counter = Counting::<_, u16>::new(
        limited_serial,
        CountingStatistics::default(),
        CountingStatistics::default(),
        CountingStatistics::default(),
    );

    let mut last_print = Instant::now();

    println!("Start loop");
    loop {
        nb::block!(counter.loop_nb())?;

        if Duration::from_millis(PRINT_INTERVAL_MS) < last_print.elapsed() {
            println!(
                "TX(bytes): sent: {}, errors: {}",
                counter.tx_stats().successful(),
                counter.tx_stats().failed()
            );

            println!(
                "RX(bytes): total: {}, errors: {}",
                counter.rx_stats().total(),
                counter.rx_stats().failed()
            );

            if counter.loss_stats().total() != 0 {
                println!(
                    "RX(packet): loss: {}, total: {}, {:.02}%",
                    counter.loss_stats().failed(),
                    counter.loss_stats().total(),
                    (counter.loss_stats().failed() * 10000 / counter.loss_stats().total()) as f64
                        / 100.0
                );
            }

            last_print = Instant::now();
        }
    }
}
