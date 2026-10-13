use super::{EncoderMode, HasChannel, Instance, SlaveMode, TriggerSelection, WithEncoder};
use core::marker::PhantomData;

/// Quadrature encoder interface.
///
/// Only TIM1 / TIM3 have the slave-mode controller (SMCR), so all methods are
/// bounded by `WithEncoder` — calling them on TIM14 / TIM16 fails to compile.
pub struct Encoder<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance + WithEncoder> Encoder<'d, T> {
    pub(super) fn new() -> Self {
        Self { _t: PhantomData }
    }

    /// Configure as a quadrature encoder (CH1 / CH2 used as TI1 / TI2 inputs).
    pub fn config(&self, mode: EncoderMode) {
        // The encoder needs CH1 and CH2.
        assert!(
            T::CHANNELS >= 2,
            "encoder mode requires at least 2 channels (current = {})",
            T::CHANNELS
        );
        T::set_encoder_mode(mode);
    }

    /// Set the counter limit (ARR).
    #[inline]
    pub fn set_period(&self, arr: u16) {
        T::set_auto_reload(arr);
    }

    /// Set the prescaler.
    #[inline]
    pub fn set_prescaler(&self, psc: u16) {
        T::set_prescaler(psc);
    }

    /// Read the current counter value.
    #[inline]
    pub fn now(&self) -> u16 {
        T::get_cnt()
    }

    /// Write the current counter value.
    #[inline]
    pub fn set_count(&self, cnt: u16) {
        T::set_cnt(cnt);
    }

    /// Start.
    #[inline]
    pub fn start(&self) {
        T::start();
    }

    /// Stop.
    #[inline]
    pub fn stop(&self) {
        T::stop();
    }
}

/// Slave-mode related (the SMCR capability that the encoder relies on).
impl<'d, T: Instance + WithEncoder> Encoder<'d, T> {
    /// Set the slave mode.
    #[inline]
    pub fn set_slave_mode(&self, mode: SlaveMode) {
        T::set_slave_mode(mode);
    }

    /// Set the trigger input.
    #[inline]
    pub fn set_trigger_selection(&self, ts: TriggerSelection) {
        T::set_trigger_selection(ts);
    }

    /// Enable master/slave synchronization.
    #[inline]
    pub fn set_master_slave_mode(&self, en: bool) {
        T::set_master_slave_mode(en);
    }
}

/// The encoder occupies CH1 / CH2: the instance must actually have both channels.
impl<'d, T: Instance + WithEncoder + HasChannel<1> + HasChannel<2>> Encoder<'d, T> {
    /// Confirm that this instance can be used for dual-channel encoder input.
    #[inline]
    pub fn has_two_inputs(&self) -> bool {
        T::CHANNELS >= 2
    }
}
