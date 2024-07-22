mod hal;
mod pins;
use core::marker::PhantomData;

use crate::clock;
use crate::gpio::{self, AnyPin};
use crate::macro_def::pin_af_for_instance_def;
use crate::mode::{Async, Blocking, Mode};
use hal::sealed;

pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {}

// 定义 串口的特殊引脚 trait
pin_af_for_instance_def!(TxPin, Instance);
pin_af_for_instance_def!(RxPin, Instance);
pin_af_for_instance_def!(RtsPin, Instance);
pin_af_for_instance_def!(CtsPin, Instance);

// sealed usart impl
macro_rules! impl_sealed_usart {
    (
        $peripheral: ident, $usart_id: ident
    ) => {
        impl Instance for crate::mcu::peripherals::$peripheral {}

        impl sealed::Instance for crate::mcu::peripherals::$peripheral {
            fn id() -> Usart {
                Usart::$usart_id
            }
        }
    };
}

// 为 usart1/2 实现 Instance 和 sealed::Instance trait
impl_sealed_usart!(USART1, USART1);
impl_sealed_usart!(USART2, USART2);

#[derive(Debug)]
pub enum Error {
    StartTimeout,
    ReadTimeout,
    WriteTimeout,
}

/// 串口停止位
#[derive(Default)]
pub enum StopBits {
    #[default]
    Stop1 = 0,
    Stop2 = 1,
}

/// 串口数据长度
#[derive(Default)]
pub enum WordLen {
    #[default]
    WordLen8 = 0,
    WordLen9 = 1,
}

/// 串口配置的校验位
#[derive(Default, PartialEq)]
pub enum Parity {
    #[default]
    None = 0,
    Even = 1,
    Odd = 2,
}

/// 串口流控
#[derive(Default)]
pub enum HwFlowCtrl {
    #[default]
    None,
    Rts = 1,
    Cts = 2,
    RtsCts = 3,
}

/// 串口的波特率定义
#[derive(Default)]
pub enum BaudRate {
    // Auto = 0,
    Bps300 = 300,
    Bps1200 = 1200,
    Bps2400 = 2400,
    Bps4800 = 4800,
    Bps9600 = 9600,
    Bps1440 = 1440,
    Bps19200 = 19200,
    Bps28800 = 28800,
    Bps38400 = 38400,
    Bps57600 = 57600,
    Bps74880 = 74880,
    #[default]
    Bps115200 = 115200,
    Bps230400 = 230400,
}

/// 串口时钟过采样配置
#[derive(Default, Clone, Copy, PartialEq)]
pub enum OverSampling {
    #[default]
    OverSampling16 = 0,
    OverSampling8 = 1,
}

impl OverSampling {
    fn div(&self) -> u32 {
        if *self == Self::OverSampling16 {
            16
        } else {
            8
        }
    }
}

impl From<OverSampling> for bool {
    fn from(value: OverSampling) -> Self {
        value == OverSampling::OverSampling8
    }
}

/// 串口数据位定义
#[derive(Default, PartialEq)]
pub enum DataBits {
    #[default]
    DataBits8 = 0,
    DataBits9 = 1,
}

impl From<DataBits> for bool {
    fn from(value: DataBits) -> Self {
        value == DataBits::DataBits9
    }
}

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

// use defmt::Debug2Format;
use embassy_hal_internal::into_ref;
use embassy_hal_internal::Peripheral;
use embassy_hal_internal::PeripheralRef;

/// 串口号定义
#[derive(Clone, Copy, PartialEq)]
pub enum Usart {
    USART1,
    USART2,
}

impl Usart {
    /// 使能串口外设时钟
    fn enable(&self, en: bool) {
        match *self {
            Self::USART1 => clock::peripheral::PeripheralClock::USART1.enable(en),
            Self::USART2 => clock::peripheral::PeripheralClock::UART2.enable(en),
        }
    }

    /// 复位串口外设
    fn reset(&self) {
        match *self {
            Self::USART1 => clock::peripheral::PeripheralClock::USART1.reset(),
            Self::USART2 => clock::peripheral::PeripheralClock::UART2.reset(),
        }
    }
}

/// 串口接收处理的对象
pub struct UsartRx<'d, T: Instance, M: Mode> {
    // _p: PeripheralRef<'d, T>,
    _p: PhantomData<(T, M)>,
    _rxd: Option<PeripheralRef<'d, AnyPin>>,
    _rts: Option<PeripheralRef<'d, AnyPin>>,
}

/// 串口发送对象
pub struct UsartTx<'d, T: Instance, M: Mode> {
    _p: PhantomData<(T, M)>,
    _txd: Option<PeripheralRef<'d, AnyPin>>,
    _cts: Option<PeripheralRef<'d, AnyPin>>,
}

/// 串口对象
pub struct FlexUsart<'d, T: Instance, M: Mode> {
    pub rx: UsartRx<'d, T, M>,
    pub tx: UsartTx<'d, T, M>,
}

// use crate::gpio::sealed::Pin;

impl<'d, T: Instance, M: Mode> FlexUsart<'d, T, M> {
    pub fn split(self) -> (UsartRx<'d, T, M>, UsartTx<'d, T, M>) {
        (self.rx, self.tx)
    }

    pub fn new(
        usart: impl Peripheral<P = T> + 'd,
        rxd: Option<impl Peripheral<P = impl RxPin<T>> + 'd>,
        txd: Option<impl Peripheral<P = impl TxPin<T>> + 'd>,
        config: Config,
    ) -> Self {
        // 初始化 rxd 引脚
        let rxd = rxd.map_or_else(
            || None,
            |rxd| {
                into_ref!(rxd);
                rxd.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::Pullup);
                // defmt::info!("rxd: {} ", Debug2Format(&(rxd.af())));
                Some(rxd.map_into())
            },
        );
        // 初始化 txd 引脚
        let txd = txd.map_or_else(
            || None,
            |txd| {
                into_ref!(txd);
                txd.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::Pullup);
                // defmt::info!("txd: {} ", Debug2Format(&(txd.af())));
                Some(txd.map_into())
            },
        );

        into_ref!(usart);

        Self::new_inner(usart, rxd, txd, None, None, config)
    }

    fn new_inner(
        _usart: PeripheralRef<'d, T>,
        rxd: Option<PeripheralRef<'d, AnyPin>>,
        txd: Option<PeripheralRef<'d, AnyPin>>,
        cts: Option<PeripheralRef<'d, AnyPin>>,
        rts: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
    ) -> Self {
        // let block = T::block();
        T::enable(true);
        T::config(config);

        Self {
            rx: UsartRx::<T, M>::new(rxd, rts),
            tx: UsartTx::<T, M>::new(txd, cts),
        }
    }
}

impl<'d, T: Instance> UsartRx<'d, T, Blocking> {
    pub fn read_blocking(&self, buf: &mut [u8]) -> usize {
        T::read_bytes_blocking(buf)
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
    pub fn read(&self, _buf: &mut [u8]) {
        todo!()
    }
}

impl<'d, T: Instance, M: Mode> UsartRx<'d, T, M> {
    pub(crate) fn new(
        rxd: Option<PeripheralRef<'d, AnyPin>>,
        rts: Option<PeripheralRef<'d, AnyPin>>,
    ) -> Self {
        T::rx_enable(rxd.is_some());
        T::rts_enable(rts.is_none());

        Self {
            _p: PhantomData,
            _rxd: rxd,
            _rts: rts,
        }
    }
}

impl<'d, T: Instance> UsartTx<'d, T, Blocking> {
    pub fn write_bytes_blocking(&self, buf: &[u8]) {
        T::write_bytes_blocking(buf);
    }
}

impl<'d, T: Instance> UsartTx<'d, T, Async> {
    pub fn write(&self, _buf: &mut [u8]) {
        todo!()
    }
}

impl<'d, T: Instance, M: Mode> UsartTx<'d, T, M> {
    pub(crate) fn new(
        txd: Option<PeripheralRef<'d, AnyPin>>,
        cts: Option<PeripheralRef<'d, AnyPin>>,
    ) -> Self {
        T::tx_enable(txd.is_some());
        T::cts_enable(cts.is_some());

        Self {
            _p: PhantomData,
            _txd: txd,
            _cts: cts,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'d, T: Instance> embedded_hal::serial::Read<u8> for UsartRx<'d, T, Blocking> {
    type Error = Error;
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.nb_read()
    }
}

impl<'d, T: Instance> embedded_hal::serial::Write<u8> for UsartTx<'d, T, Blocking> {
    type Error = Error;
    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        while !T::tx_ready() {}
        Ok(())
    }

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        T::write_byte_blocking(word);
        Ok(())
    }
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
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

impl<'d, T: Instance> embedded_io::ErrorType for UsartTx<'d, T, Blocking> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_io::Write for UsartTx<'d, T, Blocking> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(T::write_bytes_blocking(buf))
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

#[cfg(test)]
fn test() {
    // let uart = FlexUsart::new(USART1);
}
