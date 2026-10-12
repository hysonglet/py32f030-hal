use super::{Error, Event, Instance};
#[cfg(feature = "embassy")]
use super::{future::EventFuture, Async as AsyncMode};
use crate::dma::{self, DmaChannel};
use crate::gpio::AnyPin;
use crate::mcu::peripherals::DMA;
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;
use embassy_hal_internal::PeripheralRef;
use enumset::EnumSet;

/// 串口发送对象
pub struct UsartTx<'d, T: Instance, M: Mode> {
    _p: PhantomData<(T, M)>,
    _txd: Option<PeripheralRef<'d, AnyPin>>,
    _cts: Option<PeripheralRef<'d, AnyPin>>,

    pub(crate) tx_dma: Option<DmaChannel<'d, DMA, M>>,
}

impl<'d, T: Instance, M: Mode> UsartTx<'d, T, M> {
    pub(crate) fn new(
        txd: Option<PeripheralRef<'d, AnyPin>>,
        cts: Option<PeripheralRef<'d, AnyPin>>,
        tx_dma: Option<DmaChannel<'d, DMA, M>>,
    ) -> Self {
        T::tx_enable(txd.is_some());
        T::cts_enable(cts.is_some());

        Self {
            _p: PhantomData,
            _txd: txd,
            _cts: cts,
            tx_dma,
        }
    }

    /// 是否配置了 DMA 通道
    #[inline]
    pub fn has_dma(&self) -> bool {
        self.tx_dma.is_some()
    }

    /// 发送断开符（常用于 LIN / 帧起始标识）
    #[inline]
    pub fn send_break(&self) {
        T::send_break()
    }

    /// 停止发送断开符
    #[inline]
    pub fn clear_break(&self) {
        T::clear_break()
    }
}

////////////////////////////// Blocking //////////////////////////////

impl<'d, T: Instance> UsartTx<'d, T, Blocking> {
    /// 阻塞发送；若配置了 DMA 通道则走 DMA
    pub fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        if self.tx_dma.is_none() {
            self.write_bytes_blocking(buf)
        } else {
            self.write_bytes_dma_blocking(buf)
        }
    }

    fn write_bytes_blocking(&mut self, buf: &[u8]) -> Result<(), Error> {
        T::write_bytes_blocking(buf)
    }

    fn write_bytes_dma_blocking(&mut self, buf: &[u8]) -> Result<(), Error> {
        if let Some(dma) = &mut self.tx_dma {
            let (_rx_dma_map, tx_dma_map) = T::id().dma_channel_map();

            // 不管成功与否都关闭 dma 触发
            let _tx_dmp_close = drop_move::DropGuard::new(|| T::tx_dma_enable(false));

            dma.clear_flag(EnumSet::all());

            dma.config(dma::Config::new_mem2periph(
                buf.as_ptr() as u32,
                true,
                dma::Burst::Single,
                T::block().dr.as_ptr() as u32,
                false,
                dma::Burst::Single,
                dma::Priorities::Medium,
                dma::RepeatMode::OneTime(buf.len() as u16),
            ));

            dma.bind(tx_dma_map);
            dma.start();

            T::tx_dma_enable(true);
            dma.wait_complet().map_err(|_| Error::DMA)?;
            Ok(())
        } else {
            Err(Error::DMA)
        }
    }

    /// 阻塞等待发送完成（TC 置位）
    pub fn flush(&self) -> nb::Result<(), Error> {
        T::tx_flush();
        Ok(())
    }
}

////////////////////////////// Async //////////////////////////////

#[cfg(feature = "embassy")]
impl<'d, T: Instance> UsartTx<'d, T, AsyncMode> {
    /// 异步发送；若配置了 DMA 通道则走 DMA
    pub async fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        if self.tx_dma.is_some() {
            self.write_bytes_dma(buf).await
        } else {
            self.write_bytes(buf).await
        }
    }

    async fn write_bytes(&mut self, buf: &[u8]) -> Result<(), Error> {
        let events = Event::TXE | Event::CTS;
        for v in buf {
            T::write(*v);
            let happened = EventFuture::<T>::new(events).await;
            if !happened.contains(Event::TXE) {
                return Err(Error::Others);
            }
        }
        Ok(())
    }

    /// 通过 dma 异步发送数据
    async fn write_bytes_dma(&mut self, buf: &[u8]) -> Result<(), Error> {
        if let Some(dma) = &mut self.tx_dma {
            let (_rx_dma_map, tx_dma_map) = T::id().dma_channel_map();

            let _tx_dmp_close = drop_move::DropGuard::new(|| T::tx_dma_enable(false));

            dma.config(dma::Config::new_mem2periph(
                buf.as_ptr() as u32,
                true,
                dma::Burst::Single,
                T::block().dr.as_ptr() as u32,
                false,
                dma::Burst::Single,
                dma::Priorities::Medium,
                dma::RepeatMode::OneTime(buf.len() as u16),
            ));

            dma.bind(tx_dma_map);
            dma.start();

            T::tx_dma_enable(true);

            dma.wait_complet().await.map_err(|_| Error::DMA)?;

            Ok(())
        } else {
            Err(Error::DMA)
        }
    }

    /// 异步等待发送完成
    pub async fn flush(&mut self) -> Result<(), Error> {
        EventFuture::<T>::new(EnumSet::empty() | Event::TC).await;
        Ok(())
    }
}

////////////////////////////// trait 实现 //////////////////////////////

impl<'d, T: Instance> embedded_hal_027::serial::Write<u8> for UsartTx<'d, T, Blocking> {
    type Error = Error;

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        T::tx_flush();
        Ok(())
    }

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.write(&[word])?;
        Ok(())
    }
}

impl<'d, T: Instance> embedded_io::ErrorType for UsartTx<'d, T, Blocking> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io::Write for UsartTx<'d, T, Blocking> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        T::tx_flush();
        Ok(())
    }
}

#[cfg(feature = "embassy")]
impl<'d, T: Instance> embedded_io_async::ErrorType for UsartTx<'d, T, AsyncMode> {
    type Error = Error;
}

#[cfg(feature = "embassy")]
impl<'d, T: Instance> embedded_io_async::Write for UsartTx<'d, T, AsyncMode> {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        EventFuture::<T>::new(EnumSet::empty() | Event::TC).await;
        Ok(())
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write(buf).await?;
        Ok(buf.len())
    }
}
