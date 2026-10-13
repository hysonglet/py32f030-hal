//! Analog comparator (COMP1 / COMP2)
//!
//! Note: on this chip the COMP interrupt shares the `ADC_COMP` vector with the ADC
//! (which already owns it), so this module only provides **polling** output reads and
//! exposes no interrupt/async interface (same approach as LPTIM).

mod instance;
mod types;

pub use types::*;

use crate::clock::peripheral::{PeripheralClockIndex, PeripheralIdToClockIndex};
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + instance::sealed::CompInstance + 'static + Send {}

/// Comparator index
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Id {
    COMP1 = 0,
    COMP2 = 1,
}

impl PeripheralIdToClockIndex for Id {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            Self::COMP1 => PeripheralClockIndex::COMP1,
            Self::COMP2 => PeripheralClockIndex::COMP2,
        }
    }
}

instance::sealed::impl_comp_instance!(COMP1, COMP1, COMP1, has_hyst);
instance::sealed::impl_comp_instance!(COMP2, COMP2, COMP2, no_hyst);

/// Comparator handle
pub struct AnyComp<'d, T: Instance> {
    _t: PhantomData<&'d T>,
    inner: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> AnyComp<'d, T> {
    /// Create and configure the comparator (disabled by default; call `enable()`).
    pub fn new(comp: impl Peripheral<P = T> + 'd, config: Config) -> Self {
        into_ref!(comp);

        T::id().clock().open();
        T::config(config);

        Self {
            _t: PhantomData,
            inner: comp,
        }
    }

    /// Enable the comparator.
    #[inline]
    pub fn enable(&self) {
        T::enable(true);
    }

    /// Disable the comparator.
    #[inline]
    pub fn disable(&self) {
        T::enable(false);
    }

    /// Read the comparator output level.
    ///
    /// true means the non-inverting input is above the inverting input
    /// (reversed when polarity is inverted).
    #[inline]
    pub fn output(&self) -> bool {
        T::output()
    }

    /// Block until the output reaches the given level.
    #[inline]
    pub fn wait_for(&self, level: bool) {
        while T::output() != level {}
    }

    /// Reconfigure (the comparator is disabled first).
    #[inline]
    pub fn reconfig(&self, config: Config) {
        T::config(config);
    }

    #[inline]
    pub fn set_hysteresis(&self, hyst: Hysteresis) {
        T::set_hysteresis(hyst);
    }

    #[inline]
    pub fn set_power_mode(&self, mode: PowerMode) {
        T::set_power_mode(mode);
    }

    #[inline]
    pub fn set_polarity(&self, polarity: Polarity) {
        T::set_polarity(polarity);
    }

    #[inline]
    pub fn set_window_mode(&self, en: bool) {
        T::set_window_mode(en);
    }

    /// Lock the configuration; after locking, CSR becomes read-only until reset.
    #[inline]
    pub fn lock(&self) {
        T::lock();
    }

    /// Whether the configuration is locked.
    #[inline]
    pub fn is_locked(&self) -> bool {
        T::is_locked()
    }

    /// Configure the digital filter.
    #[inline]
    pub fn set_filter(&self, en: bool, count: u8) {
        T::set_filter(en, count);
    }

    /// Release: disable the comparator and turn off its clock.
    pub fn deinit(self) {
        T::enable(false);
        T::id().clock().close();
    }
}

impl<'d, T: Instance> Drop for AnyComp<'d, T> {
    fn drop(&mut self) {
        T::enable(false);
    }
}

// Keep the peripheral reference alive.
impl<'d, T: Instance> core::ops::Deref for AnyComp<'d, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
