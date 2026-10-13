mod counter;
mod instance;
mod types;

pub use counter::Counter;
pub use types::*;

use core::marker::PhantomData;
use crate::clock::peripheral::{PeripheralClockIndex, PeripheralIdToClockIndex};
use embassy_hal_internal::{into_ref, Peripheral};

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + instance::sealed::Instance + 'static + Send {}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LpTimer {
    LPTIM,
}

impl PeripheralIdToClockIndex for LpTimer {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            Self::LPTIM => PeripheralClockIndex::LPTIM,
        }
    }
}

// Note: the PAC `Interrupt` enum lacks the `LPTIM1` variant (hardware vector 17 exists),
// so this module does not implement `PeripheralInterrupt` / embassy async yet, and only
// provides a blocking polling API.

macro_rules! impl_sealed_timer {
    (
        $peripheral: ident, $id: ident
    ) => {
        impl instance::sealed::Instance for crate::mcu::peripherals::$peripheral {
            fn id() -> LpTimer {
                LpTimer::$id
            }
        }
        impl Instance for crate::mcu::peripherals::$peripheral {}
    };
}

impl_sealed_timer!(LPTIM, LPTIM);

pub struct AnyLpTimer<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance> AnyLpTimer<'d, T> {
    /// Create a new low-power timer.
    pub fn new(_timer: impl Peripheral<P = T>) -> Result<Self, Error> {
        into_ref!(_timer);

        T::id().clock().open();

        Ok(Self { _t: PhantomData })
    }

    /// Convert into counter / delay mode.
    pub fn as_counter(self, config: LpTimerConfig) -> Counter<'d, T> {
        Counter::new(config)
    }
}

impl<'d, T: Instance> Drop for AnyLpTimer<'d, T> {
    fn drop(&mut self) {
        T::stop();
        T::id().clock().close();
    }
}
