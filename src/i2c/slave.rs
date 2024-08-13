use super::Instance;
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;

/// Slave 角色
pub struct Slave<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,
}

impl<'d, T: Instance, M: Mode> Slave<'d, T, M> {
    pub(super) fn new() -> Self {
        todo!();
        // Self { _t: PhantomData }
    }
}

impl<'d, T: Instance> Slave<'d, T, Blocking> {}
