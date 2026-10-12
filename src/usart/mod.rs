//! 串口（USART）驱动
//!
//! 结构上参考定时器模块：寄存器层由 `hal::sealed::Instance` 封装，
//! 面向用户的 `UsartRx` / `UsartTx` 分别拆到 `rx.rs` / `tx.rs`，
//! `AnyUsart` 负责引脚、时钟与 DMA 通道的持有与分发。
//!
//! USART1 与 USART2 复用同一套寄存器块布局，功能一致。

#[cfg(feature = "embassy")]
mod future;
mod hal;
mod pins;
mod rx;
mod tx;
mod types;

pub use rx::UsartRx;
pub use tx::UsartTx;
pub use types::*;

use crate::clock::peripheral::{
    PeripheralClockIndex, PeripheralIdToClockIndex, PeripheralInterrupt,
};
use crate::dma::DmaChannel;
use crate::gpio::{self, AnyPin};
use crate::mcu::peripherals::DMA;
#[cfg(feature = "embassy")]
use crate::mode::Async;
use crate::mode::{Blocking, Mode};
use crate::syscfg::DmaChannelMap;
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use enumset::EnumSetType;
use hal::sealed;

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {}

// 定义 串口的特殊引脚 trait
pin_af_for_instance_def!(TxPin, Instance);
pin_af_for_instance_def!(RxPin, Instance);
pin_af_for_instance_def!(RtsPin, Instance);
pin_af_for_instance_def!(CtsPin, Instance);

/// 串口号
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Id {
    USART1 = 0,
    USART2 = 1,
}

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
    pub(crate) fn dma_channel_map(&self) -> (DmaChannelMap, DmaChannelMap) {
        match *self {
            Self::USART1 => (DmaChannelMap::USART1_RX, DmaChannelMap::USART1_TX),
            Self::USART2 => (DmaChannelMap::USART2_RX, DmaChannelMap::USART2_TX),
        }
    }
}

/// 串口事件
#[derive(EnumSetType, Debug)]
pub enum Event {
    /// 自动波特率错误标志
    ABRE,
    /// 自动波特率检测标志
    ABRF,
    /// CTS 标志
    CTS,
    /// 发送数据寄存器空标志
    TXE,
    /// 发送完成标志
    TC,
    /// 接收数据寄存器非空标志
    RXNE,
    /// 线路空闲标志
    IDLE,
    /// 溢出错误标志（与 NE/FE/PE 共用 CR3.EIE）
    ORE,
    /// 噪声错误标志
    NE,
    /// 帧错误标志
    FE,
    /// 校验错误标志
    PE,
}

/// 把可选引脚转为已配置为复用功能的 `AnyPin`
///
/// 写成宏而不是泛型函数：引脚的 `set_instance_af` / `map_into` 来自
/// `RxPin` / `TxPin` 等 trait 约束，泛型函数无法表达这些约束。
macro_rules! into_af_pin {
    ($pin:expr) => {
        $pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::OpenDrain);
                Some(pin.map_into())
            },
        )
    };
}

/// 串口对象
pub struct AnyUsart<'d, T: Instance, M: Mode> {
    pub rx: UsartRx<'d, T, M>,
    pub tx: UsartTx<'d, T, M>,
    _p: PhantomData<PeripheralRef<'d, T>>,
}

impl<'d, T: Instance, M: Mode> AnyUsart<'d, T, M> {
    /// 拆分为独立的接收与发送对象
    pub fn split(self) -> (UsartRx<'d, T, M>, UsartTx<'d, T, M>) {
        (self.rx, self.tx)
    }

    /// 新建串口（不使用硬件流控）
    pub fn new(
        usart: impl Peripheral<P = T> + 'd,
        rxd: Option<impl Peripheral<P = impl RxPin<T>> + 'd>,
        txd: Option<impl Peripheral<P = impl TxPin<T>> + 'd>,

        rx_dma: Option<DmaChannel<'d, DMA, M>>,
        tx_dma: Option<DmaChannel<'d, DMA, M>>,

        config: Config,
    ) -> Self {
        // 初始化 rxd 引脚
        let rxd = into_af_pin!(rxd);
        // 初始化 txd 引脚
        let txd = into_af_pin!(txd);

        into_ref!(usart);

        Self::new_inner(usart, rxd, txd, None, None, rx_dma, tx_dma, config)
    }

    /// 新建串口并指定硬件流控引脚（RTS / CTS）
    ///
    /// 使用硬件流控时，`config.hw_flow_ctrl` 需配置为对应的
    /// `HwFlowCtrl::Rts` / `Cts` / `RtsCts`，且传入对应引脚。
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_flow_control(
        usart: impl Peripheral<P = T> + 'd,
        rxd: Option<impl Peripheral<P = impl RxPin<T>> + 'd>,
        txd: Option<impl Peripheral<P = impl TxPin<T>> + 'd>,
        rts: Option<impl Peripheral<P = impl RtsPin<T>> + 'd>,
        cts: Option<impl Peripheral<P = impl CtsPin<T>> + 'd>,

        rx_dma: Option<DmaChannel<'d, DMA, M>>,
        tx_dma: Option<DmaChannel<'d, DMA, M>>,

        config: Config,
    ) -> Self {
        let rxd = into_af_pin!(rxd);
        let txd = into_af_pin!(txd);
        let rts = into_af_pin!(rts);
        let cts = into_af_pin!(cts);

        into_ref!(usart);

        Self::new_inner(usart, rxd, txd, cts, rts, rx_dma, tx_dma, config)
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
            _p: PhantomData,
        }
    }

    /// 停止串口并关闭外设时钟
    ///
    /// 说明：`AnyUsart` 未实现 `Drop`（`split()` 需要移出字段），
    /// 需要释放外设时请显式调用本方法。
    pub fn deinit(self) {
        T::stop();
        T::disable();
    }

    /// 停止串口（保留时钟，可再次 `start()`）
    pub fn stop(&self) {
        T::stop();
    }

    /// 启动串口
    pub fn start(&self) {
        T::start();
    }

    /// 重新配置串口参数
    pub fn reconfig(&self, config: Config) {
        T::config(config);
    }

    /// 读取自动波特率测量结果（BRR 值）
    pub fn auto_baud_result(&self) -> u16 {
        T::auto_baud_result()
    }

    /// 触发一次自动波特率测量
    pub fn request_auto_baud(&self) {
        T::request_auto_baud()
    }
}

////////////////////////////// trait 实现 //////////////////////////////

impl<'d, T: Instance> embedded_hal_027::serial::Read<u8> for AnyUsart<'d, T, Blocking> {
    type Error = Error;
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.rx.nb_read()
    }
}

impl<'d, T: Instance> embedded_hal_027::serial::Write<u8> for AnyUsart<'d, T, Blocking> {
    type Error = Error;
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.tx.flush()
    }
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.tx.write(&[word])?;
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

impl<'d, T: Instance> embedded_io::Read for AnyUsart<'d, T, Blocking> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf)
    }
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

#[cfg(feature = "embassy")]
impl<'d, T: Instance> embedded_io_async::ErrorType for AnyUsart<'d, T, Async> {
    type Error = Error;
}

#[cfg(feature = "embassy")]
impl<'d, T: Instance> embedded_io_async::Read for AnyUsart<'d, T, Async> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.rx.read(buf).await
    }
}

#[cfg(feature = "embassy")]
impl<'d, T: Instance> embedded_io_async::Write for AnyUsart<'d, T, Async> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.tx.write(buf).await?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.tx.flush().await
    }
}
