use embedded_hal_nb::nb::{Error, Result};
use embedded_hal_nb::serial::{ErrorType, Read, Write};

use crate::statistics::Statistics;

use super::counter::Counter;
use super::{Counting, ValidCounting};

/// A valid counting test that haves Non-blocking error as base
pub trait ValidCountingNbError: ValidCounting {
    type Error;
}

/// A valid counting test that supports nonblocking read
pub trait ValidCountingNbRead: ValidCountingNbError {
    /// Receive byte from the serial port and verify it. Non-blocking.
    fn recv_nb(&mut self) -> Result<(), Self::Error>;
}

pub trait ValidCountingNbWrite: ValidCountingNbError {
    /// Send byte to the serial port. Non-blocking.
    fn send_nb(&mut self) -> Result<(), Self::Error>;

    fn flush_nb(&mut self) -> Result<(), Self::Error>;
}

pub trait ValidCountingNb: ValidCountingNbWrite + ValidCountingNbRead {
    fn loop_nb(&mut self) -> Result<(), Self::Error> {
        let (recv_res, send_res) = (self.recv_nb(), self.send_nb());

        match (recv_res, send_res) {
            // All good, both sides sent and received something
            (Ok(_), Ok(_)) => Ok(()),
            // Both is blocked
            (Err(Error::WouldBlock), Err(Error::WouldBlock)) => Err(Error::WouldBlock),
            // One of is blocked so client can call again to try to send or receive something
            (Err(Error::WouldBlock), _) | (_, Err(Error::WouldBlock)) => Ok(()),
            // One of the sides has an error
            (Err(e), _) | (_, Err(e)) => Err(e),
        }
    }
}

impl<Serial, Number, TxStats, RxStats, LossStats> ValidCountingNbError
    for Counting<Serial, Number, TxStats, RxStats, LossStats>
where
    Serial: ErrorType,
    Number: Counter,
    TxStats: Statistics,
    RxStats: Statistics,
    LossStats: Statistics,
{
    type Error = Serial::Error;
}

impl<Serial, Number, TxStats, RxStats, LossStats> ValidCountingNbRead
    for Counting<Serial, Number, TxStats, RxStats, LossStats>
where
    Serial: Read,
    Number: Counter,
    TxStats: Statistics,
    RxStats: Statistics,
    LossStats: Statistics,
{
    fn recv_nb(&mut self) -> Result<(), Serial::Error> {
        let byte_read = match self.serial.read() {
            Ok(b) => b,
            Err(Error::WouldBlock) => return Err(Error::WouldBlock),
            Err(e) => {
                self.rx_stats.add_failed(1);
                return Err(e);
            }
        };

        self.on_byte_received(byte_read);

        Ok(())
    }
}

impl<Serial, Number, TxStats, RxStats, LossStats> ValidCountingNbWrite
    for Counting<Serial, Number, TxStats, RxStats, LossStats>
where
    Serial: Write,
    Number: Counter,
    TxStats: Statistics,
    RxStats: Statistics,
    LossStats: Statistics,
{
    /// Sends next byte using non blocking API
    fn send_nb(&mut self) -> Result<(), Serial::Error> {
        let byte_to_send = self.tx_state.peek();

        match self.serial.write(byte_to_send) {
            Ok(_) => {
                self.on_byte_sent();
                Ok(())
            }
            Err(Error::WouldBlock) => Err(Error::WouldBlock),
            Err(e) => {
                self.tx_stats.add_failed(1);
                Err(e)
            }
        }
    }

    /// Flushes serial port using non blocking API
    ///
    /// # Warning
    /// The error happened here will not affect tx_state
    fn flush_nb(&mut self) -> Result<(), Serial::Error> {
        self.serial.flush()
    }
}

impl<T> ValidCountingNb for T where T: ValidCountingNbWrite + ValidCountingNbRead {}
