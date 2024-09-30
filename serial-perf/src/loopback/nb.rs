use embedded_hal_nb::nb::{Error, Result};
use embedded_hal_nb::serial::{Read, Write};

use super::{Loopback, State};

impl<Serial> Loopback<Serial>
where
    Serial: Read,
{
    pub fn recv_nb(&mut self) -> Result<(), Serial::Error> {
        let byte_read = self.serial.read()?;
        self.on_byte_received(byte_read);

        Ok(())
    }
}

impl<Serial> Loopback<Serial>
where
    Serial: Write,
{
    /// Sends next byte using non blocking API
    pub fn send_nb(&mut self) -> Result<(), Serial::Error> {
        let byte_to_send = self.byte_to_send().ok_or(Error::WouldBlock)?;

        self.serial.write(byte_to_send)?;

        self.on_byte_sent();

        Ok(())
    }

    /// Flushes serial port using non blocking API
    ///
    /// # Warning
    /// The error happened here will not affect tx_state
    pub fn flush_nb(&mut self) -> Result<(), Serial::Error> {
        self.serial.flush()
    }
}

impl<Serial> Loopback<Serial>
where
    Serial: Write + Read,
{
    pub fn loop_nb(&mut self) -> Result<(), Serial::Error> {
        match self.state {
            State::Receiving => self.recv_nb(),
            State::Transfer(_) => self.send_nb(),
        }
    }
}
