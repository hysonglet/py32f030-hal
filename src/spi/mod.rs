//! SPI driver.
//!
//! Structurally consistent with other peripherals: the register layer is wrapped by `instance::sealed::Instance`,
//! the user-facing `Master` / `Slave` live in `master.rs` / `slave.rs`.
//! SPI1 and SPI2 share the same register-block layout.

mod instance;
pub mod master;
mod pins;
mod slave;
mod types;

pub use master::Master;
pub use slave::Slave;
pub use types::*;

use crate::clock::peripheral::{
    PeripheralClockIndex, PeripheralIdToClockIndex, PeripheralInterrupt,
};
use crate::dma::DmaChannel;
use crate::gpio::{self, AnyPin, PinIoType, Speed};
use crate::mcu::peripherals::DMA;
use crate::mode::Mode;
use crate::syscfg::DmaChannelMap;
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

/// SPI peripheral identifier.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Id {
    SPI1,
    SPI2,
}

impl Id {
    /// Return the DMA channel mapping values for (TX, RX).
    pub(crate) fn dma_channel_map(&self) -> (DmaChannelMap, DmaChannelMap) {
        match *self {
            Self::SPI1 => (DmaChannelMap::SPI1_TX, DmaChannelMap::SPI1_RX),
            Self::SPI2 => (DmaChannelMap::SPI2_TX, DmaChannelMap::SPI2_RX),
        }
    }
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

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + instance::sealed::Instance + 'static + Send {}

impl_sealed_peripheral_id!(SPI1, SPI1);
impl_sealed_peripheral_id!(SPI2, SPI2);

pub struct AnySpi<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _m: PhantomData<M>,

    // Pins
    _sck: PeripheralRef<'d, AnyPin>,
    _mosi: Option<PeripheralRef<'d, AnyPin>>,
    _miso: Option<PeripheralRef<'d, AnyPin>>,
    _nss: Option<PeripheralRef<'d, AnyPin>>,

    tx_dma: Option<DmaChannel<'d, DMA, M>>,
    rx_dma: Option<DmaChannel<'d, DMA, M>>,
}

impl<'d, T: Instance, M: Mode> AnySpi<'d, T, M> {
    #[allow(clippy::too_many_arguments)]
    fn new_inner(
        _spi: PeripheralRef<'d, T>,
        sck: PeripheralRef<'d, AnyPin>,
        mosi: Option<PeripheralRef<'d, AnyPin>>,
        miso: Option<PeripheralRef<'d, AnyPin>>,
        nss: Option<PeripheralRef<'d, AnyPin>>,
        tx_dma: Option<DmaChannel<'d, DMA, M>>,
        rx_dma: Option<DmaChannel<'d, DMA, M>>,
        config: Config,
    ) -> Result<Self, Error> {
        // SPI must be disabled before configuration.
        T::spi_enable(false);

        T::set_clock_phase(config.mode.phase);
        T::set_clock_polarity(config.mode.polarity);
        T::set_frame_format(config.bit_order);
        T::set_baud_rate_div(config.baud_rate_div);
        T::set_data_length(config.data_len);
        T::set_bidirectional_mode(config.bidirectional);
        T::enable_rx_readonly(config.rx_only);
        T::set_slave_mode(config.slave_speed_mode);
        T::set_fifo_rx_threshold_8bit(config.fifo_rx_threshold_8bit);
        T::enable_ss_output(config.ss_output_enable);

        // When the NSS pin is unused, software slave management must be enabled, otherwise a spurious MODF is reported.
        T::enable_soft_slave_management(config.software_slave_management || nss.is_none());
        T::slave_enable(true);

        T::spi_enable(true);

        Ok(Self {
            _t: PhantomData,
            _m: PhantomData,
            _sck: sck,
            _miso: miso,
            _mosi: mosi,
            _nss: nss,
            tx_dma,
            rx_dma,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        spi: impl Peripheral<P = T> + 'd,
        sck: impl Peripheral<P = impl SckPin<T>> + 'd,
        mosi: Option<impl Peripheral<P = impl MosiPin<T>> + 'd>,
        miso: Option<impl Peripheral<P = impl MisoPin<T>> + 'd>,
        nss: Option<impl Peripheral<P = impl NssPin<T>> + 'd>,
        tx_dma: Option<DmaChannel<'d, DMA, M>>,
        rx_dma: Option<DmaChannel<'d, DMA, M>>,
        config: Config,
    ) -> Result<Self, Error> {
        into_ref!(sck);

        sck.set_instance_af(
            Speed::VeryHigh,
            if config.mode.polarity == embedded_hal::spi::Polarity::IdleLow {
                PinIoType::PullDown
            } else {
                PinIoType::PullUp
            },
        );

        let mosi = mosi.map_or_else(
            || None,
            |mosi| {
                into_ref!(mosi);
                mosi.set_instance_af(Speed::VeryHigh, PinIoType::PullUp);
                Some(mosi.map_into())
            },
        );

        let miso = miso.map_or_else(
            || None,
            |miso| {
                into_ref!(miso);
                miso.set_instance_af(Speed::VeryHigh, PinIoType::OpenDrain);
                Some(miso.map_into())
            },
        );

        let nss = nss.map_or_else(
            || None,
            |nss| {
                into_ref!(nss);
                nss.set_instance_af(Speed::VeryHigh, PinIoType::Floating);
                Some(nss.map_into())
            },
        );

        T::id().clock().open();

        into_ref!(spi);

        Self::new_inner(spi, sck.map_into(), mosi, miso, nss, tx_dma, rx_dma, config)
    }

    /// Convert into master mode.
    pub fn as_master(self) -> Master<'d, T, M> {
        T::set_rule(Rule::Master);
        Master::<T, M>::new(self._sck, self._mosi, self._miso, self._nss, self.tx_dma, self.rx_dma)
    }

    /// Convert into slave mode.
    pub fn as_slave(self) -> Slave<'d, T, M> {
        T::set_rule(Rule::Slave);
        Slave::<T, M>::new(self._sck, self._mosi, self._miso, self._nss, self.tx_dma, self.rx_dma)
    }

    /// Disable SPI and release the peripheral clock.
    pub fn deinit(self) {
        T::spi_enable(false);
        T::id().clock().close();
    }
}

pub struct Tx<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,
}

pub struct Rx<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,
}

pin_af_for_instance_def!(SckPin, Instance);
pin_af_for_instance_def!(MisoPin, Instance);
pin_af_for_instance_def!(MosiPin, Instance);
pin_af_for_instance_def!(NssPin, Instance);

// Note: the `gpio` import is used for pin AF configuration; kept to match other peripherals' style.
#[allow(unused_imports)]
use gpio as _gpio;
