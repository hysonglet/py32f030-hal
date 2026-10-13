#[cfg(feature = "embassy")]
mod future;
mod instance;
mod pins;
mod types;

use core::convert::Infallible;
use embedded_hal::digital::PinState;

pub use types::*;

// Internal register-access marker type (in instance.rs::sealed, always compiled).
use crate::exti::instance::sealed::{ExtiImpl as ExtiReg, Instance as _};

// use self::instance::sealed::Instance;
use crate::gpio::Pin;
use crate::gpio::{Input, Pull, Speed};
#[cfg(feature = "embassy")]
use crate::mode::Async;
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral};

// pub trait ExitPin:
//     Peripheral<P = Self> + sealed::Instance + crate::gpio::Pin + 'static + Send
// {
//     fn set_pin_select(&self) {
//         Self::exit_channle_select(self.pin().into(), self.port().into());
//     }
// }

pub struct ExtiInput<'d, M: Mode> {
    pin: Input<'d>,
    _mode: PhantomData<M>,
}

impl<'d, M: Mode> Unpin for ExtiInput<'d, M> {}

impl<'d, M: Mode> ExtiInput<'d, M> {
    pub fn new(pin: impl Peripheral<P = impl Pin> + 'd, pull: Pull, speed: Speed) -> Self {
        into_ref!(pin);

        Self {
            pin: Input::new(pin, pull, speed),
            _mode: PhantomData,
        }
    }

    /// Return the pin level.
    #[inline]
    fn get_level(&self) -> PinState {
        self.pin.read()
    }
}

impl<'d> ExtiInput<'d, Blocking> {
    /// Wait until the pin level goes low.
    #[inline]
    pub fn wait_for_low(&self) {
        while self.get_level() == PinState::High {}
    }

    /// Wait until the pin level goes high.
    #[inline]
    pub fn wait_for_high(&self) {
        while self.get_level() == PinState::Low {}
    }

    /// Wait for a rising edge.
    #[inline]
    pub fn wait_for_rising_edge(&self) {
        self.wait_for_low();
        self.wait_for_high();
    }

    /// Wait for a falling edge.
    #[inline]
    pub fn wait_for_falling_edge(&self) {
        self.wait_for_high();
        self.wait_for_low();
    }

    /// Wait for any edge (rising or falling).
    #[inline]
    pub fn wait_for_any_edge(&self) {
        let last = self.get_level();
        while last == self.get_level() {}
    }
}

#[cfg(feature = "embassy")]
impl<'d> ExtiInput<'d, Async> {
    /// Wait until the pin level goes low.
    pub async fn wait_for_low(&self) {
        if self.get_level() == PinState::Low {
            return;
        }

        future::ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::Falling).await
    }

    /// Wait until the pin level goes high.
    pub async fn wait_for_high(&self) {
        if self.get_level() == PinState::High {
            return;
        }
        future::ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::Rising).await
    }

    /// Wait for a rising edge.
    pub async fn wait_for_rising_edge(&self) {
        future::ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::Rising).await
    }

    /// Wait for a falling edge.
    pub async fn wait_for_falling_edge(&self) {
        future::ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::Falling).await
    }

    /// Wait for any edge (rising or falling).
    pub async fn wait_for_any_edge(&self) {
        future::ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::RisingFalling)
            .await
    }
}

impl<'a, M: Mode> embedded_hal::digital::ErrorType for ExtiInput<'a, M> {
    type Error = core::convert::Infallible;
}

impl<'a, M: Mode> embedded_hal::digital::InputPin for ExtiInput<'a, M> {
    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.pin.is_low()
    }

    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        self.pin.is_high()
    }
}

impl<'a, M: Mode> embedded_hal_027::digital::v2::InputPin for ExtiInput<'a, M> {
    type Error = Infallible;
    fn is_low(&self) -> Result<bool, Self::Error> {
        self.pin.is_low()
    }

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.pin.is_high()
    }
}

/// Directly drive the EXTI peripheral (line enable, edges, software trigger, pending flags).
///
/// `ExtiInput` is a GPIO-pin-oriented convenience wrapper; for lower-level control
/// (software trigger, pending clear, internal lines PVD/RTC/COMP), use this struct.
pub struct Exti;

impl Exti {
    /// Select the GPIO line-to-port mapping (write SYSCFG EXTICR); valid for Line0–15 only.
    #[inline]
    pub fn select_gpio_source(line: Line, port: crate::gpio::GpioPort) {
        ExtiReg::exit_channle_select(line, port.into());
    }

    /// Set the edge trigger condition (rising / falling / both).
    #[inline]
    pub fn set_edge(line: Line, edge: Edge) {
        ExtiReg::line_ring_edge(line, edge.is_rising());
        ExtiReg::line_falling_edge(line, edge.is_falling());
    }

    /// Enable / mask a line's interrupt and event (IMR).
    #[inline]
    pub fn enable(line: Line, en: bool) {
        ExtiReg::line_pend_enable(line, en);
    }

    /// Software-trigger an EXTI line (sets SWIER; the hardware clears it after setting PR).
    #[inline]
    pub fn software_trigger(line: Line) {
        ExtiReg::software_trigger(line);
    }

    /// Read a line's pending status.
    #[inline]
    pub fn is_pending(line: Line) -> bool {
        ExtiReg::pending_raw(line)
    }

    /// Clear a line's pending flag (PR).
    #[inline]
    pub fn clear_pending(line: Line) {
        ExtiReg::clear_pending(line);
    }

    /// Wait for a line to become pending (blocking poll).
    #[inline]
    pub fn wait_pending(line: Line) {
        while !Self::is_pending(line) {}
    }
}
