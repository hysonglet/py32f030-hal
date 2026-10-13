//! Segment LED display driver controller (LED)
//!
//! This peripheral drives multiplexed segment LCD/LED panels with multiple COM
//! lines. The on/off state of each segment (A–G plus DP, 8 segments total) on a
//! given COM line is set via `dr0`..`dr3`, and an internal scan circuit cycles
//! through them automatically. See the reference manual, LED controller chapter.

mod instance;
mod types;

pub use types::*;

use crate::clock::peripheral::PeripheralIdToClockIndex;
use crate::impl_sealed_peripheral_id;
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral};

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + instance::sealed::Instance + 'static + Send {}

impl_sealed_peripheral_id!(LED, Led1);

/// LED display controller handle
pub struct AnyLed<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance> AnyLed<'d, T> {
    /// Create and start scanning with the given configuration.
    pub fn new(_led: impl Peripheral<P = T> + 'd, config: LedConfig) -> Self {
        into_ref!(_led);

        T::id().clock().open();
        Self::apply_config(config);

        Self { _t: PhantomData }
    }

    fn apply_config(config: LedConfig) {
        T::set_prescaler(config.prescaler);
        T::set_com_count(config.com_count);
        T::set_timing(config.t1, config.t2);
        T::set_light_control(config.light_control);
        T::set_interrupt(config.interrupt);
    }

    /// Reconfigure the display.
    #[inline]
    pub fn reconfig(&self, config: LedConfig) {
        Self::apply_config(config);
    }

    /// Enable/disable the display scan.
    #[inline]
    pub fn enable(&self, en: bool) {
        T::enable(en);
    }

    /// Set the prescaler.
    #[inline]
    pub fn set_prescaler(&self, pr: u8) {
        T::set_prescaler(pr);
    }

    /// Set the number of COM lines (1–4).
    #[inline]
    pub fn set_com_count(&self, count: u8) {
        T::set_com_count(count);
    }

    /// Set display timing t1 / t2.
    #[inline]
    pub fn set_timing(&self, t1: u8, t2: u8) {
        T::set_timing(t1, t2);
    }

    /// Set brightness control.
    #[inline]
    pub fn set_light_control(&self, lc: LightControl) {
        T::set_light_control(lc);
    }

    /// Write the 8-segment display data for a COM line.
    ///
    /// bit0–bit7 of `seg` map to segments A, B, C, D, E, F, G, DP respectively.
    #[inline]
    pub fn set_display_data(&self, com: u8, seg: u32) {
        T::set_display_data(com, seg);
    }

    /// Enable the scan-complete interrupt.
    #[inline]
    pub fn enable_interrupt(&self, en: bool) {
        T::set_interrupt(en);
    }

    /// Whether the interrupt is pending.
    #[inline]
    pub fn is_interrupt_pending(&self) -> bool {
        T::is_interrupt_pending()
    }

    /// Clear the interrupt flag.
    #[inline]
    pub fn clear_interrupt(&self) {
        T::clear_interrupt();
    }

    /// Stop scanning and turn off the peripheral clock.
    pub fn deinit(self) {
        T::enable(false);
        T::id().clock().close();
    }
}

impl<'d, T: Instance> Drop for AnyLed<'d, T> {
    fn drop(&mut self) {
        T::enable(false);
        T::id().clock().close();
    }
}
