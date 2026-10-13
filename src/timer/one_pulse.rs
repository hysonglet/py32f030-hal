use super::{
    Channel, ChannelConfig, ChannelOutput, ChannelType, Event, HasChannel, Instance,
    InputCaptureConfig, OnePulseMode, WithEncoder, WithOnePulse,
};
use core::marker::PhantomData;

/// One-pulse configuration.
#[derive(Clone, Copy)]
pub struct OnePulseConfig {
    /// Output channel.
    pub output_channel: Channel,
    /// Output channel configuration.
    pub channel: ChannelConfig,
    /// Count period (ARR).
    pub period: u16,
}

impl Default for OnePulseConfig {
    fn default() -> Self {
        Self {
            output_channel: Channel::CH1,
            channel: ChannelConfig {
                mode: super::ChannelMode::PWM1,
                ..Default::default()
            },
            period: 0xFFFF,
        }
    }
}

/// One Pulse Mode.
///
/// Only TIM1 / TIM3 / TIM16 have OPM, so it is gated at compile time by
/// `WithOnePulse`. External trigger input needs SMCR (i.e. the `WithEncoder`
/// capability) and a second channel, so it is gated separately.
pub struct OnePulse<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance + WithOnePulse> OnePulse<'d, T> {
    pub(super) fn new(cfg: OnePulseConfig, op_mode: OnePulseMode) -> Self {
        assert!(
            cfg.output_channel.index() < T::CHANNELS,
            "this timer does not support this output channel"
        );

        T::stop();
        T::set_channel_type(cfg.output_channel, ChannelType::Pwm);
        T::set_channel_output_config(
            cfg.output_channel,
            cfg.channel.mode,
            cfg.channel.clear,
            cfg.channel.fast,
            cfg.channel.preload,
        );
        T::set_channel_compare(cfg.output_channel, cfg.channel.compare);
        T::set_channel_output_effective_level(cfg.output_channel, ChannelOutput::P, true, true, false);
        T::set_auto_reload(cfg.period);
        // With OPM=1 the counter stops after the update event.
        T::enable_single_mode(op_mode == OnePulseMode::Single);

        Self { _t: PhantomData }
    }

    /// Start the one-pulse (software trigger).
    #[inline]
    pub fn start(&self) {
        T::event_clear(Event::UIF);
        T::start();
    }

    /// Stop.
    #[inline]
    pub fn stop(&self) {
        T::stop();
    }

    /// Whether it has finished (the update event has occurred).
    #[inline]
    pub fn is_finished(&self) -> bool {
        T::event_flag(Event::UIF)
    }

    /// Clear the finished flag.
    #[inline]
    pub fn clear_finished(&self) {
        T::event_clear(Event::UIF);
    }

    /// Block until the pulse ends.
    #[inline]
    pub fn wait(&self) {
        while !T::event_flag(Event::UIF) {}
        T::event_clear(Event::UIF);
    }
}

/// External-trigger start (needs SMCR + a second channel, TIM1 / TIM3 only).
impl<'d, T: Instance + WithOnePulse + WithEncoder + HasChannel<2>> OnePulse<'d, T> {
    /// Configure the external trigger input (CH2 used as the TI2 trigger source).
    pub fn set_external_trigger(&self, config: InputCaptureConfig) {
        T::set_channel_input_config(
            Channel::CH2,
            config.selection,
            config.polarity,
            config.prescaler,
            config.filter,
        );
        // Trigger mode: a rising edge on TRGI starts the counter.
        T::set_trigger_selection(super::TriggerSelection::TI2FP2);
        T::set_slave_mode(super::SlaveMode::Trigger);
    }
}
