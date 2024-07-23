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
