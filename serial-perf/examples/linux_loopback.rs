use clap::Parser;
use linux_embedded_hal::Serial;
use serial_perf::{loopback::Loopback, statistics::DummyStatistics};

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
    let mut loopback = Loopback::new(serial, DummyStatistics, DummyStatistics);

    println!("Start loop");
    loop {
        nb::block!(loopback.loop_nb())?;
    }
}
