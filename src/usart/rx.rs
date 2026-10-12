use super::{Error, Event, Instance};
#[cfg(feature = "embassy")]
use super::{future::EventFuture, Async as AsyncMode};
use crate::dma::{self, DmaChannel};
use crate::gpio::AnyPin;
use crate::mcu::peripherals::DMA;
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;
#[cfg(feature = "embassy")]
use embassy_hal_internal::PeripheralRef;
use enumset::EnumSet;

/// 串口接收对象
pub struct UsartRx<'d, T: Instance, M: Mode> {
    _p: PhantomData<(T, M)>,
    _rxd: Option<PeripheralRef<'d, AnyPin>>,
    _rts: Option<PeripheralRef<'d, AnyPin>>,

    pub(crate) rx_dma: Option<DmaChannel<'d, DMA, M>>,
}

impl<'d, T: Instance, M: Mode> UsartRx<'d, T, M> {
    pub(crate) fn new(
        rxd: Option<PeripheralRef<'d, AnyPin>>,
        rts: Option<PeripheralRef<'d, AnyPin>>,
        rx_dma: Option<DmaChannel<'d, DMA, M>>,
    ) -> Self {
        T::rx_enable(rxd.is_some());
        T::rts_enable(rts.is_some());

        Self {
            _p: PhantomData,
            _rxd: rxd,
            _rts: rts,
            rx_dma,
        }
    }

    /// 是否配置了 DMA 通道
    #[inline]
    pub fn has_dma(&self) -> bool {
        self.rx_dma.is_some()
    }

    /// 非阻塞读取单字节
    #[inline]
    pub fn nb_read(&self) -> nb::Result<u8, Error> {
        if T::rx_ready() {
            Ok(T::read())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

////////////////////////////// Blocking //////////////////////////////

impl<'d, T: Instance> UsartRx<'d, T, Blocking> {
    /// 阻塞读取，直到填满缓冲区
    pub fn read_blocking(&self, buf: &mut [u8]) -> usize {
        T::read_bytes_blocking(buf)
    }

    /// 阻塞读取，遇到线路空闲（IDLE）提前结束，返回实际收到的字节数
    pub fn read_idle_blocking(&self, buf: &mut [u8]) -> usize {
        T::read_bytes_idle_blocking(buf)
    }

    /// 使用 DMA 接收，IDLE 或缓冲区满时结束，返回实际收到的字节数
    pub fn read_dma_idle_blocking(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let len = buf.len();

        if let Some(dma) = &mut self.rx_dma {
            // 清除上一次可能残留的标志
            let clear_events = Event::IDLE | Event::RXNE | Event::NE | Event::FE | Event::PE;
            clear_events.iter().for_each(|e| T::event_clear(e));

            // 返回 dma 通道的映射值
            let (rx_dma_map, _tx_dma_map) = T::id().dma_channel_map();

            dma.clear_flag(EnumSet::all());
            // 不管成功与否都关闭 dma 触发
            let _rx_dmp_close = drop_move::DropGuard::new(|| T::rx_dma_enable(false));

            dma.config(dma::Config::new_periph2mem(
                T::block().dr.as_ptr() as u32,
                false,
                dma::Burst::Single,
                buf.as_ptr() as u32,
                true,
                dma::Burst::Single,
                dma::Priorities::Medium,
                dma::RepeatMode::OneTime(buf.len() as u16),
            ));

            dma.bind(rx_dma_map);
            dma.start();

            T::rx_dma_enable(true);

            let remain = loop {
                if T::event_flag(Event::FE) {
                    return Err(Error::Frame);
                }
                if T::event_flag(Event::PE) {
                    return Err(Error::Parity);
                }
                if T::event_flag(Event::NE) {
                    return Err(Error::Noise);
                }
                if dma.is_error() {
                    return Err(Error::DMA);
                }
                if dma.is_finish() {
                    break 0usize;
                }
                if T::event_flag(Event::IDLE) {
                    break dma.remain() as usize;
                }
            };
            Ok(len - remain)
        } else {
            Err(Error::DMA)
        }
    }
}

////////////////////////////// Async //////////////////////////////

#[cfg(feature = "embassy")]
impl<'d, T: Instance> UsartRx<'d, T, AsyncMode> {
    /// 异步读取，直到填满缓冲区
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let events = Event::RXNE | Event::PE | Event::NE | Event::FE | Event::ORE;
        let mut cnt = 0;

        for v in buf {
            let happened = EventFuture::<T>::new(events).await;

            // 先判定错误，再取数据
            if let Some(e) = Self::to_error(&happened) {
                return Err(e);
            }

            if happened.contains(Event::RXNE) {
                *v = T::read();
                cnt += 1;
            }
        }

        Ok(cnt)
    }

    /// 异步读取，遇到线路空闲（IDLE）提前结束
    pub async fn read_with_idle(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let events =
            Event::RXNE | Event::PE | Event::NE | Event::FE | Event::ORE | Event::IDLE;
        let mut cnt = 0;

        for v in buf {
            let happened = EventFuture::<T>::new(events).await;

            if happened.contains(Event::IDLE) {
                T::event_clear(Event::IDLE);
                return Ok(cnt);
            }

            if let Some(e) = Self::to_error(&happened) {
                return Err(e);
            }

            if happened.contains(Event::RXNE) {
                *v = T::read();
                cnt += 1;
            }
        }

        Ok(cnt)
    }

    fn to_error(happened: &EnumSet<Event>) -> Option<Error> {
        if happened.contains(Event::ORE) {
            Some(Error::Overrun)
        } else if happened.contains(Event::NE) {
            Some(Error::Noise)
        } else if happened.contains(Event::FE) {
            Some(Error::Frame)
        } else if happened.contains(Event::PE) {
            Some(Error::Parity)
        } else {
            None
        }
    }
}

////////////////////////////// trait 实现 //////////////////////////////

impl<'d, T: Instance> embedded_hal_027::serial::Read<u8> for UsartRx<'d, T, Blocking> {
    type Error = Error;
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.nb_read()
    }
}

impl<'d, T: Instance> embedded_io::ErrorType for UsartRx<'d, T, Blocking> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io::Read for UsartRx<'d, T, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(T::read_bytes_blocking(buf))
    }
}

#[cfg(feature = "embassy")]
impl<'d, T: Instance> embedded_io_async::ErrorType for UsartRx<'d, T, AsyncMode> {
    type Error = Error;
}

#[cfg(feature = "embassy")]
impl<'d, T: Instance> embedded_io_async::Read for UsartRx<'d, T, AsyncMode> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_with_idle(buf).await
    }
}
