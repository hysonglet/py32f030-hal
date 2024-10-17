mod future;
mod hal;
mod pins;
mod types;

use crate::clock::peripheral::{
    PeripheralClockIndex, PeripheralIdToClockIndex, PeripheralInterrupt,
};
use crate::dma::DmaChannel;
use crate::gpio::{self, AnyPin};
use crate::macro_def::pin_af_for_instance_def;
use crate::mcu::peripherals::DMA;
use crate::mode::{Async, Blocking, Mode};
use crate::syscfg::DmaChannelMap;
use crate::{clock, dma};
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;
use drop_move::DropGuard;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use enumset::{EnumSet, EnumSetType};
use future::EventFuture;
use hal::sealed;
use types::*;

pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {}

// 定义 串口的特殊引脚 trait
pin_af_for_instance_def!(TxPin, Instance);
pin_af_for_instance_def!(RxPin, Instance);
pin_af_for_instance_def!(RtsPin, Instance);
pin_af_for_instance_def!(CtsPin, Instance);

/// 串口的综合配置结构体
#[derive(Default)]
pub struct Config {
    pub baud_rate: BaudRate,
    pub stop_bit: StopBits,
    pub word_len: WordLen,
    pub parity: Parity,
    // pub hw_flow_ctrl: HwFlowCtrl,
    pub data_bits: DataBits,
    pub over_sampling: OverSampling,
    // pub mode: T,
}

/// 串口号定义
#[derive(Clone, Copy, PartialEq)]
pub enum Id {
    USART1,
    USART2,
}

// 为 usart1/2 实现 Instance 和 sealed::Instance trait
impl_sealed_peripheral_id!(USART1, USART1);
impl_sealed_peripheral_id!(USART2, USART2);

impl PeripheralIdToClockIndex for Id {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            Self::USART1 => PeripheralClockIndex::USART1,
            Self::USART2 => PeripheralClockIndex::UART2,
        }
    }
}

impl PeripheralInterrupt for Id {
    fn interrupt(&self) -> PY32f030xx_pac::interrupt {
        match *self {
            Self::USART1 => PY32f030xx_pac::interrupt::USART1,
            Self::USART2 => PY32f030xx_pac::interrupt::USART2,
        }
    }
}

impl Id {
    // 返回串口 rx 和 tx 的 dma 映射值
    fn dma_channel_map(&self) -> (DmaChannelMap, DmaChannelMap) {
        match *self {
            Self::USART1 => (DmaChannelMap::USART1_RX, DmaChannelMap::USART1_TX),
            Self::USART2 => (DmaChannelMap::USART2_RX, DmaChannelMap::USART2_TX),
        }
    }
}

#[derive(EnumSetType)]
pub enum Event {
    /// 自动波特率错误标志
    ABRE,
    /// 自动波特率检测标志
    ABRF,
    /// CTS 标志
    CTS,
    /// 传输寄存器空标志
    TXE,
    /// 传送完成标志
    TC,
    /// 读数据寄存器不空标志
    RXNE,
    /// 空闲标志
    IDLE,
    /// Over 正常运行错误标志
    ORE,
    /// 噪声错误标志
    NE,
    /// 噪声错误标志
    FE,
    /// 校验值错误
    PE,
}

/// 串口接收处理的对象
pub struct UsartRx<'d, T: Instance, M: Mode> {
    // _p: PeripheralRef<'d, T>,
    _p: PhantomData<(T, M)>,
    _rxd: Option<PeripheralRef<'d, AnyPin>>,
    _rts: Option<PeripheralRef<'d, AnyPin>>,

    rx_dma: Option<DmaChannel<'d, DMA, M>>,
}

/// 串口发送对象
pub struct UsartTx<'d, T: Instance, M: Mode> {
    _p: PhantomData<(T, M)>,
    _txd: Option<PeripheralRef<'d, AnyPin>>,
    _cts: Option<PeripheralRef<'d, AnyPin>>,

    tx_dma: Option<DmaChannel<'d, DMA, M>>,
}

/// 串口对象
pub struct AnyUsart<'d, T: Instance, M: Mode> {
    pub rx: UsartRx<'d, T, M>,
    pub tx: UsartTx<'d, T, M>,
}

impl<'d, T: Instance, M: Mode> AnyUsart<'d, T, M> {
    pub fn split(self) -> (UsartRx<'d, T, M>, UsartTx<'d, T, M>) {
        (self.rx, self.tx)
    }

    pub fn new(
        usart: impl Peripheral<P = T> + 'd,
        rxd: Option<impl Peripheral<P = impl RxPin<T>> + 'd>,
        txd: Option<impl Peripheral<P = impl TxPin<T>> + 'd>,

        rx_dma: Option<DmaChannel<'d, DMA, M>>,
        tx_dma: Option<DmaChannel<'d, DMA, M>>,

        config: Config,
    ) -> Self {
        // 初始化 rxd 引脚
        let rxd = rxd.map_or_else(
            || None,
            |rxd| {
                into_ref!(rxd);
                rxd.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::PullUp);
                Some(rxd.map_into())
            },
        );
        // 初始化 txd 引脚
        let txd = txd.map_or_else(
            || None,
            |txd| {
                into_ref!(txd);
                txd.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::PullUp);
                Some(txd.map_into())
            },
        );

        into_ref!(usart);

        Self::new_inner(usart, rxd, txd, None, None, rx_dma, tx_dma, config)
    }

    #[allow(clippy::too_many_arguments)]
    fn new_inner(
        _usart: PeripheralRef<'d, T>,
        rxd: Option<PeripheralRef<'d, AnyPin>>,
        txd: Option<PeripheralRef<'d, AnyPin>>,
        cts: Option<PeripheralRef<'d, AnyPin>>,
        rts: Option<PeripheralRef<'d, AnyPin>>,
        rx_dma: Option<DmaChannel<'d, DMA, M>>,
        tx_dma: Option<DmaChannel<'d, DMA, M>>,
        config: Config,
    ) -> Self {
        T::enable();
        T::config(config);

        if M::is_async() {
            T::id().enable_interrupt();
        }

        Self {
            rx: UsartRx::<T, M>::new(rxd, rts, rx_dma),
            tx: UsartTx::<T, M>::new(txd, cts, tx_dma),
        }
    }
}

impl<'d, T: Instance> UsartRx<'d, T, Blocking> {
    pub fn read_blocking(&self, buf: &mut [u8]) -> usize {
        T::read_bytes_blocking(buf)
    }

    pub fn read_idle_blocking(&self, buf: &mut [u8]) -> usize {
        T::read_bytes_idle_blocking(buf)
    }

    pub fn nb_read(&self) -> Result<u8, nb::Error<Error>> {
        if T::rx_ready() {
            Ok(T::read_byte_blocking())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<'d, T: Instance> UsartRx<'d, T, Async> {
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let mut cnt = 0;
        for v in buf {
            let events = Event::RXNE | Event::PE | Event::NE | Event::FE | Event::ORE; //

            let event_future = poll_fn(move |cx| {
                events.iter().for_each(|e| {
                    future::EVENT_WAKERS[T::id() as usize][e as usize].register(cx.waker());
                    T::event_config(e, true);
                });

                // 查看事件
                let mut events_happen = EnumSet::empty();
                events.iter().for_each(|e| {
                    if T::event_flag(e) {
                        T::event_clear(e);
                        events_happen |= e;
                    }
                });

                // some event happen, so reset the event future
                if !events_happen.is_empty() {
                    events.iter().for_each(|e| {
                        T::event_config(e, false);
                    });
                    return Poll::Ready(events_happen);
                }

                Poll::Pending
            });

            let events_happen = event_future.await;

            // grab rx data or clear some flag
            *v = T::read();
            cnt += 1;

            if events_happen != Event::RXNE {
                return Err(Error::Others);
            }
        }

        Ok(cnt)
    }

    pub async fn read_with_idle(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let mut cnt = 0;
        let events = Event::RXNE | Event::PE | Event::NE | Event::FE | Event::ORE | Event::IDLE;
        for v in buf {
            let event = EventFuture::<T>::new(events).await;
            if event == Event::RXNE {
                *v = T::read();
                cnt += 1;
            } else if event.contains(Event::IDLE) {
                return Ok(cnt);
            } else {
                return Err(Error::Others);
            }
        }
        Ok(cnt)
    }
}

impl<'d, T: Instance, M: Mode> UsartRx<'d, T, M> {
    pub(crate) fn new(
        rxd: Option<PeripheralRef<'d, AnyPin>>,
        rts: Option<PeripheralRef<'d, AnyPin>>,
        rx_dma: Option<DmaChannel<'d, DMA, M>>,
    ) -> Self {
        T::rx_enable(rxd.is_some());
        T::rts_enable(rts.is_none());

        Self {
            _p: PhantomData,
            _rxd: rxd,
            _rts: rts,
            rx_dma,
        }
    }
}

impl<'d, T: Instance> UsartTx<'d, T, Blocking> {
    fn write_bytes_dma_blocking(&mut self, buf: &[u8]) -> Result<(), Error> {
        if let Some(dma) = &mut self.tx_dma {
            // 返回dma 通道的映射值
            let (_rx_dma_map, tx_dma_map) = T::id().dma_channel_map();

            // 不管成功与否都关闭dma触发
            let _tx_dmp_close = DropGuard::new(|| T::tx_dma_enable(false));

            // 配置dma channel
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

            // 将 tx 信号绑定到 通道
            dma.bind(tx_dma_map);
            // 使能 dma channel
            dma.start();

            // 串口开启dma
            T::tx_dma_enable(true);
            // 等待dma传输完成
            dma.wait_complet().map_err(|_| Error::DMA)?;
            Ok(())
        } else {
            Err(Error::DMA)
        }
    }

    fn write_bytes_blocking(&mut self, buf: &[u8]) -> Result<(), Error> {
        T::write_bytes_blocking(buf)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        if self.tx_dma.is_none() {
            self.write_bytes_blocking(buf)
        } else {
            self.write_bytes_dma_blocking(buf)
        }
    }

    fn flush(&self) -> nb::Result<(), Error> {
        T::tx_flush();
        Ok(())
    }
}

impl<'d, T: Instance> UsartTx<'d, T, Async> {
    pub async fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        let events = Event::TXE | Event::CTS;
        for v in buf {
            T::write(*v);
            let rst = EventFuture::<T>::new(events).await;
            if rst != Event::TXE {
                for e in rst {
                    defmt::error!("events: {}", e as usize);
                }
                return Err(Error::Others);
            }
        }
        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), Error> {
        EventFuture::<T>::new(EnumSet::empty() | Event::TC).await;
        Ok(())
    }
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
}

/// embedded_hal::serial for AnyUsart<'d, T, Blocking>
impl<'d, T: Instance> embedded_hal::serial::Read<u8> for AnyUsart<'d, T, Blocking> {
    type Error = Error;
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.rx.nb_read()
    }
}

impl<'d, T: Instance> embedded_hal::serial::Write<u8> for AnyUsart<'d, T, Blocking> {
    type Error = Error;
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.tx.flush()
    }
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.tx.write(&[word])?;
        Ok(())
    }
}

/// embedded_hal::serial for UsartRx<'d, T, Blocking>
impl<'d, T: Instance> embedded_hal::serial::Read<u8> for UsartRx<'d, T, Blocking> {
    type Error = Error;
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.nb_read()
    }
}

/// embedded_hal::serial for UsartTx<'d, T, Blocking>
impl<'d, T: Instance> embedded_hal::serial::Write<u8> for UsartTx<'d, T, Blocking> {
    type Error = Error;
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        T::tx_flush();
        Ok(())
    }

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        T::write_byte_blocking(word)?;
        Ok(())
    }
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl<'d, T: Instance> embedded_io::ErrorType for AnyUsart<'d, T, Blocking> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io::ErrorType for UsartRx<'d, T, Blocking> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io::Read for AnyUsart<'d, T, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf)
    }
}

impl<'d, T: Instance> embedded_io::Read for UsartRx<'d, T, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(T::read_bytes_blocking(buf))
    }
}

impl<'d, T: Instance> embedded_io::ErrorType for UsartTx<'d, T, Blocking> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io::Write for AnyUsart<'d, T, Blocking> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        T::tx_flush();
        Ok(())
    }
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

impl<'d, T: Instance> embedded_io_async::ErrorType for AnyUsart<'d, T, Async> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io_async::ErrorType for UsartRx<'d, T, Async> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io_async::Read for AnyUsart<'d, T, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf).await
    }
}

impl<'d, T: Instance> embedded_io_async::Read for UsartRx<'d, T, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_with_idle(buf).await
    }
}

impl<'d, T: Instance> embedded_io_async::ErrorType for UsartTx<'d, T, Async> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io_async::Write for UsartTx<'d, T, Async> {
    async fn flush(&mut self) -> Result<(), Self::Error> {
        EventFuture::<T>::new(EnumSet::empty() | Event::TC).await;
        Ok(())
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write(buf).await?;
        Ok(buf.len())
    }
}
