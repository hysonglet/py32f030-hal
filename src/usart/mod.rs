//! USART (serial) driver.
//!
//! Structurally similar to the timer module: the register layer is wrapped by
//! `instance::sealed::Instance`, the user-facing `UsartRx` / `UsartTx` are split
//! into `rx.rs` / `tx.rs`, and `AnyUsart` owns and dispatches the pins, clock
//! and DMA channels.
//!
//! USART1 and USART2 share the same register-block layout and identical features.

#[cfg(feature = "embassy")]
mod future;
mod instance;
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
use instance::sealed;

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + sealed::Instance + 'static + Send {}

// Define the USART-specific pin traits.
pin_af_for_instance_def!(TxPin, Instance);
pin_af_for_instance_def!(RxPin, Instance);
pin_af_for_instance_def!(RtsPin, Instance);
pin_af_for_instance_def!(CtsPin, Instance);

/// USART peripheral identifier.
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
    // Return the DMA mapping values for the USART rx and tx.
    pub(crate) fn dma_channel_map(&self) -> (DmaChannelMap, DmaChannelMap) {
        match *self {
            Self::USART1 => (DmaChannelMap::USART1_RX, DmaChannelMap::USART1_TX),
            Self::USART2 => (DmaChannelMap::USART2_RX, DmaChannelMap::USART2_TX),
        }
    }
}

/// USART events.
#[derive(EnumSetType, Debug)]
pub enum Event {
    /// Auto-baud-rate error flag.
    ABRE,
    /// Auto-baud-rate detection flag.
    ABRF,
    /// CTS flag.
    CTS,
    /// Transmit data register empty flag.
    TXE,
    /// Transmission complete flag.
    TC,
    /// Receive data register not-empty flag.
    RXNE,
    /// Line-idle flag.
    IDLE,
    /// Overrun error flag (shares CR3.EIE with NE/FE/PE).
    ORE,
    /// Noise error flag.
    NE,
    /// Framing error flag.
    FE,
    /// Parity error flag.
    PE,
}

/// Convert an optional pin into an `AnyPin` configured for the alternate function.
///
/// Written as a macro rather than a generic function: the pin's `set_instance_af` /
/// `map_into` come from the `RxPin` / `TxPin` etc. trait bounds, which a generic
/// function cannot express.
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

/// USART object.
pub struct AnyUsart<'d, T: Instance, M: Mode> {
    pub rx: UsartRx<'d, T, M>,
    pub tx: UsartTx<'d, T, M>,
    _p: PhantomData<PeripheralRef<'d, T>>,
}

impl<'d, T: Instance, M: Mode> AnyUsart<'d, T, M> {
    /// Split into independent receive and transmit objects.
    pub fn split(self) -> (UsartRx<'d, T, M>, UsartTx<'d, T, M>) {
        (self.rx, self.tx)
    }

    /// Create a USART (without hardware flow control).
    pub fn new(
        usart: impl Peripheral<P = T> + 'd,
        rxd: Option<impl Peripheral<P = impl RxPin<T>> + 'd>,
        txd: Option<impl Peripheral<P = impl TxPin<T>> + 'd>,

        rx_dma: Option<DmaChannel<'d, DMA, M>>,
        tx_dma: Option<DmaChannel<'d, DMA, M>>,

        config: Config,
    ) -> Self {
        // Initialize the rxd pin.
        let rxd = into_af_pin!(rxd);
        // Initialize the txd pin.
        let txd = into_af_pin!(txd);

        into_ref!(usart);

        Self::new_inner(usart, rxd, txd, None, None, rx_dma, tx_dma, config)
    }

    /// Create a USART and specify the hardware flow-control pins (RTS / CTS).
    ///
    /// When using hardware flow control, `config.hw_flow_ctrl` must be set to the
    /// corresponding `HwFlowCtrl::Rts` / `Cts` / `RtsCts`, and the matching pins
    /// must be supplied.
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

    /// Stop the USART and turn off the peripheral clock.
    ///
    /// Note: `AnyUsart` does not implement `Drop` (because `split()` needs to move
    /// out its fields); call this method explicitly when the peripheral must be freed.
    pub fn deinit(self) {
        T::stop();
        T::disable();
    }

    /// Stop the USART (keeps the clock so it can be `start()`ed again).
    pub fn stop(&self) {
        T::stop();
    }

    /// Start the USART.
    pub fn start(&self) {
        T::start();
    }

    /// Reconfigure the USART parameters.
    pub fn reconfig(&self, config: Config) {
        T::config(config);
    }

    /// Read the auto-baud-rate measurement result (the BRR value).
    pub fn auto_baud_result(&self) -> u16 {
        T::auto_baud_result()
    }

    /// Trigger a single auto-baud-rate measurement.
    pub fn request_auto_baud(&self) {
        T::request_auto_baud()
    }
}


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
