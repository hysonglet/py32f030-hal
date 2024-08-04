use crate::clock;
use crate::clock::peripheral::{PeripheralClock, PeripheralEnable};
use crate::mode::Mode;
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral};

mod hal;

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
pub enum FrameFormat {
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
}

#[derive(PartialEq, Debug)]
pub enum Error {}

impl<'d, T: Instance, M: Mode> AnySpi<'d, T, M> {
    fn enable(en: bool) {
        T::id().enable(en)
    }

    fn new_inner() -> Result<(), Error> {
        todo!()
    }

    fn new(_spi: impl Peripheral<P = T>) -> Result<Self, Error> {
        into_ref!(_spi);
        todo!()
    }
}
