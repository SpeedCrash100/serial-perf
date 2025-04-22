use criterion::{criterion_group, criterion_main, Criterion};
use serial_perf::{counting::prelude::*, counting::Counting, statistics::DummyStatistics};
use std::convert::Infallible;

const BENCH_GROUP: &str = "counting no stats";

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
    let serial = DummySerial;
    let mut counter =
        Counting::<_, u64, _, _, _>::new(serial, DummyStatistics, DummyStatistics, DummyStatistics);

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
        DummyStatistics,
        DummyStatistics,
        DummyStatistics,
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
