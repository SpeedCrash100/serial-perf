use criterion::{criterion_group, criterion_main, Criterion};
use serial_perf::{
    clock::StdClock, counting::prelude::*, counting::Counting, statistics::IntervalRateStatistics,
};
use std::{convert::Infallible, time::Duration};

const BENCH_GROUP: &str = "counting rate stats";

pub struct DummySerial;

impl embedded_hal_nb::serial::ErrorType for DummySerial {
    type Error = Infallible;
}

impl embedded_hal_nb::serial::Read for DummySerial {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        Ok(0)
    }
}

impl embedded_hal_nb::serial::Write for DummySerial {
    fn write(&mut self, _word: u8) -> nb::Result<(), Self::Error> {
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let clk = StdClock;
    let serial = DummySerial;
    let mut counter = Counting::<_, u64, _, _, _>::new(
        serial,
        IntervalRateStatistics::new(&clk, Duration::from_millis(10)),
        IntervalRateStatistics::new(&clk, Duration::from_millis(10)),
        IntervalRateStatistics::new(&clk, Duration::from_millis(10)),
    );

    let mut tx_group = c.benchmark_group(BENCH_GROUP);
    tx_group.throughput(criterion::Throughput::Bytes(1));
    tx_group.bench_function("send", |b| b.iter(|| counter.send_nb()));
    tx_group.finish();

    let mut rx_group = c.benchmark_group(BENCH_GROUP);
    rx_group.throughput(criterion::Throughput::Bytes(1));
    rx_group.bench_function("recv", |b| b.iter(|| counter.recv_nb()));
    rx_group.finish();

    let serial = DummySerial;
    let mut counter = Counting::<_, u64, _, _, _>::new_without_checksum(
        serial,
        IntervalRateStatistics::new(&clk, Duration::from_millis(10)),
        IntervalRateStatistics::new(&clk, Duration::from_millis(10)),
        IntervalRateStatistics::new(&clk, Duration::from_millis(10)),
    );

    let mut tx_group = c.benchmark_group(BENCH_GROUP);
    tx_group.throughput(criterion::Throughput::Bytes(1));
    tx_group.bench_function("send no crc", |b| b.iter(|| counter.send_nb()));
    tx_group.finish();

    let mut rx_group = c.benchmark_group(BENCH_GROUP);
    rx_group.throughput(criterion::Throughput::Bytes(1));
    rx_group.bench_function("recv no crc", |b| b.iter(|| counter.recv_nb()));
    rx_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
