use embedded_hal_nb::nb::Error;
use embedded_hal_nb::serial::{ErrorType, Read, Write};
use embedded_timers::clock::Clock;

use super::ByteRateSerialLimiter;

impl<'clock, Clk, Serial> ErrorType for ByteRateSerialLimiter<'clock, Clk, Serial>
where
    Clk: Clock,
    Serial: ErrorType,
{
    type Error = Serial::Error;
}

impl<'clock, Clk, Serial> Read for ByteRateSerialLimiter<'clock, Clk, Serial>
where
    Clk: Clock,
    Serial: Read,
{
    fn read(&mut self) -> embedded_hal_nb::nb::Result<u8, Self::Error> {
        self.serial.read()
    }
}

impl<'clock, Clk, Serial> Write for ByteRateSerialLimiter<'clock, Clk, Serial>
where
    Clk: Clock,
    Serial: Write,
{
    fn write(&mut self, word: u8) -> embedded_hal_nb::nb::Result<(), Self::Error> {
        if !self.rate_limit.can_send() {
            return Err(Error::WouldBlock);
        }

        let result = self.serial.write(word);
        if result.is_ok() {
            // FIXME: handle error here
            self.rate_limit.send().unwrap();
        }

        result
    }

    fn flush(&mut self) -> embedded_hal_nb::nb::Result<(), Self::Error> {
        self.serial.flush()
    }
}
