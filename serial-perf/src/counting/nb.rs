use embedded_hal_nb::nb::{Error, Result};
use embedded_hal_nb::serial::{Read, Write};

use super::counter::Counter;
use super::Counting;

impl<Serial, Number> Counting<Serial, Number>
where
    Serial: Read,
    Number: Counter,
{
    /// Receive byte from the serial port and verify it. Non-blocking.
    pub fn recv_nb(&mut self) -> Result<(), Serial::Error> {
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

impl<Serial, Number> Counting<Serial, Number>
where
    Serial: Write,
    Number: Counter,
{
    /// Sends next byte using non blocking API
    pub fn send_nb(&mut self) -> Result<(), Serial::Error> {
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
    pub fn flush_nb(&mut self) -> Result<(), Serial::Error> {
        self.serial.flush()
    }
}

impl<Serial, Number> Counting<Serial, Number>
where
    Serial: Write + Read,
    Number: Counter,
{
    pub fn loop_nb(&mut self) -> Result<(), Serial::Error> {
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
