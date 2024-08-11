use super::{Error, Instance};
use crate::delay::wait_for_true_timeout;
use crate::gpio::AnyPin;
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;
use embassy_hal_internal::PeripheralRef;

const TIMEOUT: usize = 10000;

/// Master 角色
pub struct Master<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,

    // 引脚
    _sck: PeripheralRef<'d, AnyPin>,
    _mosi: Option<PeripheralRef<'d, AnyPin>>,
    _miso: Option<PeripheralRef<'d, AnyPin>>,
    _nss: Option<PeripheralRef<'d, AnyPin>>,
}

impl<'d, T: Instance, M: Mode> Master<'d, T, M> {
    pub(super) fn new(
        // 引脚
        _sck: PeripheralRef<'d, AnyPin>,
        _mosi: Option<PeripheralRef<'d, AnyPin>>,
        _miso: Option<PeripheralRef<'d, AnyPin>>,
        _nss: Option<PeripheralRef<'d, AnyPin>>,
    ) -> Self {
        Self {
            _t: PhantomData,
            _sck,
            _miso,
            _mosi,
            _nss,
        }
    }
}

impl<'d, T: Instance> Master<'d, T, Blocking> {
    pub fn write_block(&self, buf: &[u8]) -> Result<usize, Error> {
        // master 模式如果没有配置mosi引脚，则无法发送数据
        if self._mosi.is_none() {
            return Err(Error::Write);
        }
        for v in buf.iter() {
            if wait_for_true_timeout(TIMEOUT, || T::tx_empty()).is_err() {
                return Err(Error::Timeout);
            }
            T::data_write(*v as u16)
        }
        Ok(buf.len())
    }

    pub fn clear_errors(&self) {}

    pub fn read_block(&self, buf: &mut [u8]) -> Result<usize, Error> {
        // master 模式如果没有配置miso引脚，则无法发送数据
        if self._miso.is_none() {
            return Err(Error::Read);
        }
        let mut cnt: usize = 0;
        for v in buf.iter_mut() {
            if wait_for_true_timeout(TIMEOUT, || T::rx_not_empty()).is_err() {
                return Err(Error::Timeout);
            }
            *v = T::data_read() as u8;
            cnt += 1;
        }
        Ok(cnt)
    }
}
