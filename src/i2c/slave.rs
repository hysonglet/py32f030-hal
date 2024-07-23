use super::{Error, I2cInstance};
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;

pub struct Slave<'d, T: I2cInstance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,
}

impl<'d, T: I2cInstance, M: Mode> Slave<'d, T, M> {
    pub(super) fn new() -> Self {
        Self { _t: PhantomData }
    }
}

impl<'d, T: I2cInstance> Slave<'d, T, Blocking> {}
