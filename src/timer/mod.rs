//! Unified APB timer module.
//!
//! TIM1 / TIM3 / TIM14 / TIM16 share a single set of `Counter` / `Pwm` /
//! `OutputCompare` / `InputCapture` / `Encoder` / `OnePulse` interfaces. Timer
//! **specific** capabilities (complementary output, break & dead-time, repetition
//! counter, encoder, one-pulse, center-aligned mode, master-mode TRGO, TI1 remap)
//! are gated at **compile time** through the marker traits in [`capability`]:
//! calling the corresponding method on a timer that lacks the hardware fails to
//! compile, instead of panicking at runtime.
//!
//! LPTIM keeps a separate [`lptim`] submodule because its register set, event
//! model and clock selection differ too much.

mod capability;
mod counter;
mod encoder;
#[cfg(feature = "embassy")]
mod future;
mod ic;
mod instance;
mod oc;
mod one_pulse;
mod pins;
mod pwm;
mod types;

pub mod lptim;

pub use capability::*;
pub use counter::Counter;
pub use encoder::Encoder;
pub use ic::{InputCapture, InputCaptureConfig};
pub use oc::OutputCompare;
pub use one_pulse::{OnePulse, OnePulseConfig};
pub use pwm::Pwm;
pub use types::*;

use core::marker::PhantomData;
use crate::clock::peripheral::PeripheralIdToClockIndex;
use crate::mode::Mode;
use embassy_hal_internal::{into_ref, Peripheral};

/// Unified timer instance bound (sealed: users cannot implement it).
///
/// Requires `ChannelCount` so that high-level APIs can validate channel
/// availability with `T::CHANNELS`.
#[allow(private_bounds)]
pub trait Instance:
    Peripheral<P = Self> + instance::sealed::Instance + ChannelCount + 'static + Send
{
}

impl<T> Instance for T where
    T: Peripheral<P = T> + instance::sealed::Instance + ChannelCount + 'static + Send
{
}

/// Timer handle: responsible for enabling/disabling the peripheral clock and
/// converting into a concrete function.
pub struct AnyTimer<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _m: PhantomData<M>,
}

impl<'d, T: Instance, M: Mode> AnyTimer<'d, T, M> {
    /// Create a timer (enables the peripheral clock).
    pub fn new(_timer: impl Peripheral<P = T>) -> Result<Self, Error> {
        into_ref!(_timer);
        T::id().clock().open();
        Ok(Self {
            _t: PhantomData,
            _m: PhantomData,
        })
    }

    /// Returns the timer peripheral clock.
    pub fn get_timer_clock() -> u32 {
        T::get_time_pclk()
    }

    /// Convert into a counter / delay.
    pub fn as_counter(self) -> Counter<'d, T, M> {
        Counter::new()
    }

    /// Convert into PWM.
    pub fn as_pwm(self) -> Pwm<'d, T> {
        Pwm::new()
    }

    /// Convert into input capture.
    pub fn as_input_capture(self) -> InputCapture<'d, T> {
        InputCapture::new()
    }

    /// Convert into output compare.
    pub fn as_output_compare(self) -> OutputCompare<'d, T> {
        OutputCompare::new()
    }
}

/// One-pulse: TIM1 / TIM3 / TIM16 only.
impl<'d, T: Instance + WithOnePulse, M: Mode> AnyTimer<'d, T, M> {
    pub fn as_one_pulse(self, cfg: OnePulseConfig, mode: OnePulseMode) -> OnePulse<'d, T> {
        OnePulse::new(cfg, mode)
    }
}

/// Encoder: TIM1 / TIM3 only.
impl<'d, T: Instance + WithEncoder, M: Mode> AnyTimer<'d, T, M> {
    pub fn as_encoder(self) -> Encoder<'d, T> {
        Encoder::new()
    }
}

impl<'d, T: Instance, M: Mode> Drop for AnyTimer<'d, T, M> {
    fn drop(&mut self) {
        T::stop();
        T::id().clock().close();
    }
}


pin_af_for_instance_def!(TimerChannel1Pin, Instance);
pin_af_for_instance_def!(TimerChannel1NPin, Instance);
pin_af_for_instance_def!(TimerChannel2Pin, Instance);
pin_af_for_instance_def!(TimerChannel2NPin, Instance);
pin_af_for_instance_def!(TimerChannel3Pin, Instance);
pin_af_for_instance_def!(TimerChannel3NPin, Instance);
pin_af_for_instance_def!(TimerChannel4Pin, Instance);
pin_af_for_instance_def!(TimerBkInPin, Instance);
pin_af_for_instance_def!(TimerEtrPin, Instance);

// Before the merge, each timer was a separate module (timer::advanced_timer /
// timer::general_purpose_timer / timer::timer14 / timer::timer16). These same-name
// aliases are kept so existing code keeps compiling unchanged.

/// Compatibility with `timer::advanced_timer::*` (the original TIM1 module).
pub mod advanced_timer {
    pub use super::*;
}

/// Compatibility with `timer::general_purpose_timer::*` (the original TIM3 module).
pub mod general_purpose_timer {
    pub use super::*;
}

/// Compatibility with `timer::timer14::*`.
pub mod timer14 {
    pub use super::*;
}

/// Compatibility with `timer::timer16::*`.
pub mod timer16 {
    pub use super::*;
}
