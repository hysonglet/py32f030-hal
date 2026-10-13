//! Timer capability markers (capability traits)
//!
//! Each trait is only implemented by timer instances that **have the corresponding
//! hardware capability**. High-level APIs put timer-specific functionality into `impl`
//! blocks bounded by these traits, so that unsupported calls are rejected at
//! **compile time** rather than panicking at runtime.
//!
//! For example, the break / dead-time unit (BDTR) only exists on TIM1 and TIM16:
//! ```ignore
//! impl<'d, T: Instance + WithBreak> Pwm<'d, T> {
//!     pub fn enable_break(&self, cfg: BreakConfig) { .. }
//! }
//! ```
//! Calling `enable_break` on TIM3 / TIM14 fails to compile.

use super::instance::sealed;

/// Number of timer channels (compile-time constant).
#[allow(private_bounds)]
pub trait ChannelCount: sealed::Instance {
    /// Number of channels actually available on this timer.
    const CHANNELS: usize;
}

/// Type-level channel marker: whether the instance has channel number `N` (1-based).
///
/// Used to pin the channel selection at compile time, so that e.g. requesting CH2
/// on a single-channel timer is rejected by the compiler.
#[allow(private_bounds)]
pub trait HasChannel<const N: usize>: sealed::Instance {}

/// Has complementary outputs (CHxN). TIM1, TIM16 only.
#[allow(private_bounds)]
pub trait WithComplementary: sealed::Instance {}

/// Has the break / dead-time unit (BDTR). TIM1, TIM16 only.
#[allow(private_bounds)]
pub trait WithBreak: sealed::Instance {}

/// Has the repetition counter (RCR). TIM1, TIM16 only.
#[allow(private_bounds)]
pub trait WithRepetition: sealed::Instance {}

/// Has the slave-mode controller / encoder interface (SMCR). TIM1, TIM3 only.
#[allow(private_bounds)]
pub trait WithEncoder: sealed::Instance {}

/// Has the one-pulse mode (OPM). TIM1, TIM3, TIM16.
#[allow(private_bounds)]
pub trait WithOnePulse: sealed::Instance {}

/// Has center-aligned mode and downcounting (CR1.CMS / DIR). TIM1, TIM3 only.
#[allow(private_bounds)]
pub trait WithCenterAlign: sealed::Instance {}

/// Has the master-mode trigger output TRGO (CR2.MMS). TIM1, TIM3 only.
#[allow(private_bounds)]
pub trait WithMasterMode: sealed::Instance {}

/// Has the TI1 input remap (OR). TIM14 only.
#[allow(private_bounds)]
pub trait WithTi1Remap: sealed::Instance {}
