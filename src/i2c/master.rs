use super::{Error, Instance};
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;

/// Master 角色
pub struct Master<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,
}

impl<'d, T: Instance, M: Mode> Master<'d, T, M> {
    pub(super) fn new() -> Self {
        Self { _t: PhantomData }
    }
}

impl<'d, T: Instance> Master<'d, T, Blocking> {
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
impl<'d, T: Instance> embedded_hal::blocking::i2c::Write for Master<'d, T, Blocking> {
    type Error = Error;
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write_block(address, bytes)
            .map_or_else(Err, |_| Ok(()))
    }
}

impl<'d, T: Instance> embedded_hal::blocking::i2c::Read for Master<'d, T, Blocking> {
    type Error = Error;
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.read_block(address, buffer)
            .map_or_else(Err, |_| Ok(()))
    }
}

impl<'d, T: Instance> embedded_hal::blocking::i2c::WriteRead for Master<'d, T, Blocking> {
    type Error = Error;
    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.write_block(address, bytes)
            .map_or_else(Err, |_| Ok(()))?;
        self.read_block(address, buffer)
            .map_or_else(Err, |_| Ok(()))
    }
}
