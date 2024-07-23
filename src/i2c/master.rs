use super::{Error, I2cInstance};
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;

pub struct Master<'d, T: I2cInstance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,
}

impl<'d, T: I2cInstance, M: Mode> Master<'d, T, M> {
    pub(super) fn new() -> Self {
        Self { _t: PhantomData }
    }
}

impl<'d, T: I2cInstance> Master<'d, T, Blocking> {
    pub fn write_block(&self, address: u8, buf: &[u8]) -> Result<usize, Error> {
        T::master_transmit_block(address, buf)
    }

    pub fn clear_errors(&self) {
        // T::soft_reset()
    }

    pub fn read_block(&self, address: u8, buf: &mut [u8]) -> Result<usize, Error> {
        T::master_receive_block(address, buf)
    }
}

////////////////////////////////////////////////////////////////////////////////
impl<'d, T: I2cInstance> embedded_hal::blocking::i2c::Write for Master<'d, T, Blocking> {
    type Error = Error;
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write_block(address, bytes)
            .map_or_else(|e| Err(e), |_| Ok(()))
    }
}

impl<'d, T: I2cInstance> embedded_hal::blocking::i2c::Read for Master<'d, T, Blocking> {
    type Error = Error;
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.read_block(address, buffer)
            .map_or_else(|e| Err(e), |_| Ok(()))
    }
}

impl<'d, T: I2cInstance> embedded_hal::blocking::i2c::WriteRead for Master<'d, T, Blocking> {
    type Error = Error;
    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.write_block(address, bytes)
            .map_or_else(|e| Err(e), |_| Ok(()))?;
        self.read_block(address, buffer)
            .map_or_else(|e| Err(e), |_| Ok(()))
    }
}
