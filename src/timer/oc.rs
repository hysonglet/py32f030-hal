use super::{
    Channel, ChannelConfig, ChannelOutput, ChannelType, Event, Instance, WithComplementary,
};
use core::marker::PhantomData;

/// Event corresponding to a channel (used to query / clear flags).
#[inline]
pub(super) fn channel_event(channel: Channel) -> Event {
    match channel {
        Channel::CH1 => Event::CC1IF,
        Channel::CH2 => Event::CC2IF,
        Channel::CH3 => Event::CC3IF,
        Channel::CH4 => Event::CC4IF,
    }
}

/// Unified output compare.
pub struct OutputCompare<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance> OutputCompare<'d, T> {
    pub(super) fn new() -> Self {
        T::enable_auto_reload_buff(true);
        Self { _t: PhantomData }
    }

    #[inline]
    fn check_channel(channel: Channel) {
        assert!(
            channel.index() < T::CHANNELS,
            "this timer does not support this channel (available channels = {})",
            T::CHANNELS
        );
    }

    /// Configure an output-compare channel.
    pub fn channel_config(&self, channel: Channel, config: ChannelConfig) {
        Self::check_channel(channel);
        T::set_channel_type(channel, ChannelType::Pwm);
        T::set_channel_output_config(
            channel,
            config.mode,
            config.clear,
            config.fast,
            config.preload,
        );
        T::set_channel_compare(channel, config.compare);
        T::set_channel_output_effective_level(channel, ChannelOutput::P, true, true, false);
    }

    /// Set the compare value (CCR).
    #[inline]
    pub fn set_compare(&self, channel: Channel, ccr: u16) {
        Self::check_channel(channel);
        T::set_channel_compare(channel, ccr);
    }

    /// Read the compare value (CCR).
    #[inline]
    pub fn get_compare(&self, channel: Channel) -> u16 {
        Self::check_channel(channel);
        T::get_channel_capture(channel)
    }

    /// Enable / disable the channel output.
    #[inline]
    pub fn enable_channel(&self, channel: Channel, en: bool) {
        Self::check_channel(channel);
        T::set_enable_channel(channel, ChannelOutput::P, en);
    }

    /// Whether a compare match occurred.
    #[inline]
    pub fn is_matched(&self, channel: Channel) -> bool {
        Self::check_channel(channel);
        T::event_flag(channel_event(channel))
    }

    /// Clear the compare-match flag.
    #[inline]
    pub fn clear_match(&self, channel: Channel) {
        Self::check_channel(channel);
        T::event_clear(channel_event(channel));
    }

    /// Generate a compare event in software.
    #[inline]
    pub fn triggle(&self, channel: Channel) {
        Self::check_channel(channel);
        let sig = match channel {
            Channel::CH1 => super::Triggle::CC1G,
            Channel::CH2 => super::Triggle::CC2G,
            Channel::CH3 => super::Triggle::CC3G,
            Channel::CH4 => super::Triggle::CC4G,
        };
        T::triggle(sig);
    }

    /// Set the period (ARR).
    #[inline]
    pub fn set_period(&self, arr: u16) {
        T::set_auto_reload(arr);
    }

    #[inline]
    pub fn start(&self) {
        T::start();
    }

    #[inline]
    pub fn stop(&self) {
        T::stop();
    }
}


impl<'d, T: Instance + WithComplementary> OutputCompare<'d, T> {
    /// Enable / disable the complementary output.
    #[inline]
    pub fn enable_channel_n(&self, channel: Channel, en: bool) {
        Self::check_channel(channel);
        T::set_enable_channel(channel, ChannelOutput::N, en);
    }
}
