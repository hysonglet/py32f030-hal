use super::{Error, Instance};
use crate::delay::wait_for_true_timeout_block;
use crate::dma::DmaChannel;
use crate::gpio::AnyPin;
use crate::mcu::peripherals::DMA;
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;
use embassy_hal_internal::PeripheralRef;

const TIMEOUT: usize = 10000;

/// Slave 角色
///
/// 从机由外部主机提供时钟，读写均以阻塞方式等待 RXNE / TXE。
pub struct Slave<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,

    _sck: PeripheralRef<'d, AnyPin>,
    _mosi: Option<PeripheralRef<'d, AnyPin>>,
    _miso: Option<PeripheralRef<'d, AnyPin>>,
    _nss: Option<PeripheralRef<'d, AnyPin>>,

    tx_dma: Option<DmaChannel<'d, DMA, M>>,
    rx_dma: Option<DmaChannel<'d, DMA, M>>,
}

impl<'d, T: Instance, M: Mode> Slave<'d, T, M> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        _sck: PeripheralRef<'d, AnyPin>,
        _mosi: Option<PeripheralRef<'d, AnyPin>>,
        _miso: Option<PeripheralRef<'d, AnyPin>>,
        _nss: Option<PeripheralRef<'d, AnyPin>>,
        tx_dma: Option<DmaChannel<'d, DMA, M>>,
        rx_dma: Option<DmaChannel<'d, DMA, M>>,
    ) -> Self {
        Self {
            _t: PhantomData,
            _sck,
            _miso,
            _mosi,
            _nss,
            tx_dma,
            rx_dma,
        }
    }

    /// 是否配置了发送 DMA 通道
    #[inline]
    pub fn has_tx_dma(&self) -> bool {
        self.tx_dma.is_some()
    }

    /// 是否配置了接收 DMA 通道
    #[inline]
    pub fn has_rx_dma(&self) -> bool {
        self.rx_dma.is_some()
    }

    /// 总线是否忙
    #[inline]
    pub fn is_busy(&self) -> bool {
        T::is_busy()
    }

    /// 是否发生溢出
    #[inline]
    pub fn is_overflow(&self) -> bool {
        T::is_overflow()
    }

    /// 清除错误标志
    #[inline]
    pub fn clear_errors(&self) {
        T::clear_errors()
    }
}

impl<'d, T: Instance> Slave<'d, T, Blocking> {
    /// 阻塞读取主机发来的数据（等待 RXNE）
    pub fn read_block(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if self._miso.is_none() {
            return Err(Error::Read);
        }
        for v in buf.iter_mut() {
            if wait_for_true_timeout_block(TIMEOUT, || T::rx_not_empty()).is_err() {
                return Err(Error::Timeout);
            }
            *v = T::data_read() as u8;
        }
        Ok(buf.len())
    }

    /// 阻塞向主机发送数据（等待 TXE）
    pub fn write_block(&mut self, buf: &[u8]) -> Result<usize, Error> {
        if self._mosi.is_none() {
            return Err(Error::Write);
        }
        for v in buf.iter() {
            if wait_for_true_timeout_block(TIMEOUT, || T::tx_empty()).is_err() {
                return Err(Error::Timeout);
            }
            T::data_write(*v as u16);
        }
        Ok(buf.len())
    }

    /// 全双工传输（收发同时进行）
    pub fn transfer_block(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        if self._mosi.is_none() || self._miso.is_none() {
            return Err(Error::Write);
        }

        let len = core::cmp::max(read.len(), write.len());
        for i in 0..len {
            let tx = write.get(i).copied().unwrap_or(0);

            if wait_for_true_timeout_block(TIMEOUT, || T::tx_empty()).is_err() {
                return Err(Error::Timeout);
            }
            T::data_write(tx as u16);

            if wait_for_true_timeout_block(TIMEOUT, || T::rx_not_empty()).is_err() {
                return Err(Error::Timeout);
            }
            let rx = T::data_read() as u8;
            if let Some(r) = read.get_mut(i) {
                *r = rx;
            }
        }
        Ok(())
    }

    /// 仅接收模式下的数据发送缓冲预填（用于下一次传输）
    #[inline]
    pub fn preload_tx(&self, data: u16) {
        T::data_write(data);
    }
}
