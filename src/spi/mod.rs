mod hal;
pub mod master;
mod pins;
mod types;

use crate::clock::peripheral::{
    PeripheralClockIndex, PeripheralIdToClockIndex, PeripheralInterrupt,
};
use crate::gpio::AnyPin;
use crate::gpio::{PinIoType, PinSpeed};
use crate::mode::Mode;
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

use embedded_hal::spi;
use types::*;

pub use master::Master;

/// spi 的 索引
#[derive(PartialEq)]
pub enum Id {
    SPI1,
    SPI2,
}

impl PeripheralIdToClockIndex for Id {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            Self::SPI1 => PeripheralClockIndex::SPI1,
            Self::SPI2 => PeripheralClockIndex::SPI2,
        }
    }
}

impl PeripheralInterrupt for Id {
    fn interrupt(&self) -> crate::pac::interrupt {
        match *self {
            Self::SPI1 => crate::pac::interrupt::SPI1,
            Self::SPI2 => crate::pac::interrupt::SPI2,
        }
    }
}

pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

impl_sealed_peripheral_id!(SPI1, SPI1);
impl_sealed_peripheral_id!(SPI2, SPI2);

pub struct AnySpi<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _m: PhantomData<M>,

    // 引脚
    _sck: PeripheralRef<'d, AnyPin>,
    _mosi: Option<PeripheralRef<'d, AnyPin>>,
    _miso: Option<PeripheralRef<'d, AnyPin>>,
    _nss: Option<PeripheralRef<'d, AnyPin>>,
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
        T::id().clock().open();

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
