//!
//! The example shows how to use library for measure incoming byte rate without any check
//!
//! The example uses Loopback that can be used. If send_nb is never is called we can use it as sink
//!

use std::time::{Duration, Instant};

use clap::Parser;
use linux_embedded_hal::Serial;
use serial_perf::{
    clock::StdClock,
    loopback::Loopback,
    statistics::{AvgRateStatistics, DummyStatistics},
};

const PRINT_INTERVAL_MS: u64 = 5000;

#[derive(Parser)]
pub struct CommonArgs {
    /// The port to connect to.
    port: String,

    #[clap(short, long, default_value_t = 115200)]
    baud_rate: u32,
}

impl CommonArgs {
    pub fn create_serial(&self) -> Serial {
        Serial::open(self.port.clone(), self.baud_rate).expect("failed to create serial")
    }
}

fn main() -> anyhow::Result<()> {
    let args = CommonArgs::parse();

    let serial = args.create_serial();

    let clock = StdClock;
    let rx_stats = AvgRateStatistics::new(&clock);
    let mut loopback = Loopback::new(serial, DummyStatistics, rx_stats);

    let mut last_print = Instant::now();

    loop {
        nb::block!(loopback.recv_nb())?;

        if Duration::from_millis(PRINT_INTERVAL_MS) < last_print.elapsed() {
            let rx_stats = loopback.rx_stats();

            let success = rx_stats
                .success_rate()
                .unwrap_or_default()
                .bytes_per_second_f64()
                .unwrap_or_default();

            let fail = rx_stats
                .failed_rate()
                .unwrap_or_default()
                .bytes_per_second_f64()
                .unwrap_or_default();

            let total = success + fail;

            println!(
                "RX (Succ Fail Total): {:.02} {:.02} {:.02}",
                success, fail, total
            );

            last_print = Instant::now();
        }
    }
}
