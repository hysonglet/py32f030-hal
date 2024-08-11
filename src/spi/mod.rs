use crate::clock;
use crate::clock::peripheral::{PeripheralClock, PeripheralEnable};
use crate::gpio::AnyPin;
use crate::gpio::{PinIoType, PinSpeed};
use crate::mode::Mode;
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

use embedded_hal::spi;
mod hal;
pub mod master;
mod pins;

pub use master::Master;

#[derive(PartialEq)]
pub(crate) enum Id {
    SPI1,
    SPI2,
}

impl PeripheralEnable for Id {
    fn enable(&self, en: bool) {
        match *self {
            Self::SPI1 => PeripheralClock::SPI1.enable(en),
            Self::SPI2 => PeripheralClock::SPI2.enable(en),
        }
    }
    fn reset(&self) {
        match *self {
            Self::SPI1 => PeripheralClock::SPI1.reset(),
            Self::SPI2 => PeripheralClock::SPI2.reset(),
        }
    }
}

/// Bidirectional data mode enable
#[derive(PartialEq)]
pub enum BidirectionalMode {
    /// 0: 2-line unidirectional data mode
    Line2Unidirectional = 0,
    /// 1: 1-line bidirectional data mode
    Line1Bidirectional = 1,
}

/// Frame format
///
/// This bit should not be changed when communication is ongoing
#[derive(PartialEq)]
pub enum BitOrder {
    /// 0: Data is transmitted with the MSB first.
    MSB = 0,
    /// 1: Data is transmitted with the LSB first.
    LSB = 1,
}

/// Baud rate control
///
/// These bits should not be changed when communication is ongoing.
/// Note: In slave mode, the fastest baud rate only supports fPCLK/4
#[derive(PartialEq, Clone, Copy)]
pub enum BaudRateDiv {
    /// 000: fPCLK/2
    Div2 = 0,
    /// 001: fPCLK/4
    Div4 = 1,
    /// 010: fPCLK/8
    Div8 = 2,
    /// 011: fPCLK/16
    Div16 = 3,
    /// 100: fPCLK/32
    Div32 = 4,
    /// 101: fPCLK/64
    Div64 = 5,
    /// 110: fPCLK/128
    Div128 = 6,
    /// 111: fPCLK/256
    Div256 = 7,
}

impl BaudRateDiv {
    pub fn baud_rate(&self) -> u32 {
        clock::sys_pclk() / (0x02 << (*self as usize))
    }
}

impl From<u8> for BaudRateDiv {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Div2,
            1 => Self::Div4,
            2 => Self::Div8,
            3 => Self::Div16,
            4 => Self::Div32,
            5 => Self::Div64,
            6 => Self::Div128,
            7 => Self::Div256,
            _ => unreachable!("value({}) must <= 7", value),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Rule {
    /// 0: Slave configuration
    Slave = 0,
    /// 1: Master configuration
    Master = 1,
}

/// Clock polarity
#[derive(PartialEq, Clone, Copy)]
pub enum ClockPolarity {
    /// 0: CK to 0 when idle
    Low = 0,
    /// 1: CK to 1 when idle
    Hight = 1,
}

/// Clock phase
#[derive(PartialEq, Clone, Copy)]
pub enum ClockPhase {
    /// 0: The first clock transition is the first data capture edge
    Low = 0,
    /// 1: The second clock transition is the first data capture edge
    Hight = 1,
}

/// Slave fast mode enable
#[derive(PartialEq)]
pub enum SlaveSpeedMode {
    /// 0: Slave normal mode, the speed of the slave mode supporting the fastest SPI clock is less than pclk/4
    Normal = 0,
    /// 1: Slave fast mode, can support SPI clock speed in slave mode up to pclk/4
    Fast = 1,
}

/// SPI transmission data length
#[derive(PartialEq)]
pub enum DataLength {
    /// 0: 8-bit data frame transmission
    Lenght8 = 0,
    /// 1: 16-bit data frame transmission
    Length16 = 1,
}

pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

macro_rules! impl_sealed_instance {
    (
        $peripheral: ident, $id: ident
    ) => {
        impl hal::sealed::Instance for crate::mcu::peripherals::$peripheral {
            fn id() -> Id {
                Id::$id
            }
        }
        impl Instance for crate::mcu::peripherals::$peripheral {}
    };
}

impl_sealed_instance!(SPI1, SPI1);
impl_sealed_instance!(SPI2, SPI2);

pub struct AnySpi<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _m: PhantomData<M>,

    // 引脚
    _sck: PeripheralRef<'d, AnyPin>,
    _mosi: Option<PeripheralRef<'d, AnyPin>>,
    _miso: Option<PeripheralRef<'d, AnyPin>>,
    _nss: Option<PeripheralRef<'d, AnyPin>>,
}

#[derive(PartialEq, Debug)]
pub enum Error {
    Init,
    Read,
    Write,
    Timeout,
}

impl<'d, T: Instance, M: Mode> AnySpi<'d, T, M> {
    fn new_inner(
        _spi: PeripheralRef<'d, T>,
        sck: PeripheralRef<'d, AnyPin>,
        mosi: Option<PeripheralRef<'d, AnyPin>>,
        miso: Option<PeripheralRef<'d, AnyPin>>,
        nss: Option<PeripheralRef<'d, AnyPin>>,
        config: Config,
    ) -> Result<Self, Error> {
        let phase = if config.mode.phase == spi::Phase::CaptureOnFirstTransition {
            ClockPhase::Low
        } else {
            ClockPhase::Hight
        };
        let polarity = if config.mode.polarity == spi::Polarity::IdleLow {
            ClockPolarity::Low
        } else {
            ClockPolarity::Hight
        };

        T::set_clock_phase(phase);
        T::set_clock_polarity(polarity);
        T::set_frame_format(config.bit_order);
        T::set_baud_rate_div(config.baud_rate_div);

        Ok(Self {
            _t: PhantomData,
            _m: PhantomData,
            _sck: sck,
            _miso: miso,
            _mosi: mosi,
            _nss: nss,
        })
    }

    pub fn new(
        spi: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        mosi: Option<impl Peripheral<P = impl MosiPin<T>> + 'd>,
        miso: Option<impl Peripheral<P = impl MisoPin<T>> + 'd>,
        nss: Option<impl Peripheral<P = impl NssPin<T>> + 'd>,
        config: Config,
    ) -> Result<Self, Error> {
        into_ref!(sck);

        sck.set_instance_af(
            PinSpeed::VeryHigh,
            if config.mode.polarity == spi::Polarity::IdleLow {
                PinIoType::PullDown
            } else {
                PinIoType::PullUp
            },
        );

        let mosi = mosi.map_or_else(
            || None,
            |mosi| {
                into_ref!(mosi);
                mosi.set_instance_af(PinSpeed::VeryHigh, PinIoType::PullUp);
                Some(mosi.map_into())
            },
        );

        let miso = miso.map_or_else(
            || None,
            |miso| {
                into_ref!(miso);
                miso.set_instance_af(PinSpeed::VeryHigh, PinIoType::OpenDrain);
                Some(miso.map_into())
            },
        );

        let nss = nss.map_or_else(
            || {
                // nss 引脚没有使用，因此
                // 必须通过 SSM=1, SSI=1 来防止任何 MODF 错误。
                T::enable_soft_slave_management(true);
                None
            },
            |nss| {
                into_ref!(nss);
                nss.set_instance_af(PinSpeed::VeryHigh, PinIoType::Floating);
                Some(nss.map_into())
            },
        );

        // 使能外设时钟
        T::id().enable(true);

        into_ref!(spi);

        Self::new_inner(spi, sck.map_into(), mosi, miso, nss, config)
    }
}

impl<'d, T: Instance, M: Mode> AnySpi<'d, T, M> {
    pub fn as_master(self) -> Master<'d, T, M> {
        T::set_rule(Rule::Master);
        Master::<T, M>::new(self._sck, self._mosi, self._miso, self._nss)
    }
}

pub struct Tx<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,
}

pub struct Rx<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,
}

pub struct Config {
    pub mode: spi::Mode,
    pub bit_order: BitOrder,
    pub baud_rate_div: BaudRateDiv,
    pub data_len: DataLength,
}

pin_af_for_instance_def!(SckPin, Instance);
pin_af_for_instance_def!(MisoPin, Instance);
pin_af_for_instance_def!(MosiPin, Instance);
pin_af_for_instance_def!(NssPin, Instance);
