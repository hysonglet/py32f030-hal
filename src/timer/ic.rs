use super::{
    oc::channel_event, Channel, ChannelType, Event, Instance, InputCaptureFilter,
    InputCapturePolarity, InputCapturePrescaler, InputCaptureSelection,
};
use core::marker::PhantomData;

/// Input-capture channel configuration.
#[derive(Clone, Copy)]
pub struct InputCaptureConfig {
    /// Input mapping (CCxS).
    pub selection: InputCaptureSelection,
    /// Capture edge.
    pub polarity: InputCapturePolarity,
    /// Input prescaler (capture once every N events).
    pub prescaler: InputCapturePrescaler,
    /// Digital filter.
    pub filter: InputCaptureFilter,
}

impl Default for InputCaptureConfig {
    fn default() -> Self {
        Self {
            selection: InputCaptureSelection::DirectTI,
            polarity: InputCapturePolarity::Rising,
            prescaler: InputCapturePrescaler::Div1,
            filter: InputCaptureFilter::none(),
        }
    }
}

/// Channel over-capture event.
#[inline]
fn overcapture_event(channel: Channel) -> Event {
    match channel {
        Channel::CH1 => Event::CC1OF,
        Channel::CH2 => Event::CC2OF,
        Channel::CH3 => Event::CC3OF,
        Channel::CH4 => Event::CC4OF,
    }
}

/// Unified input capture.
pub struct InputCapture<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance> InputCapture<'d, T> {
    pub(super) fn new() -> Self {
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

    /// Configure an input-capture channel.
    pub fn channel_config(&self, channel: Channel, config: InputCaptureConfig) {
        Self::check_channel(channel);
        T::set_channel_type(channel, ChannelType::Capture1);
        T::set_channel_input_config(
            channel,
            config.selection,
            config.polarity,
            config.prescaler,
            config.filter,
        );
        // Enable capture.
        T::set_enable_channel(channel, super::ChannelOutput::P, true);
    }

    /// Read the captured value (CCR).
    #[inline]
    pub fn capture(&self, channel: Channel) -> u16 {
        Self::check_channel(channel);
        T::get_channel_capture(channel)
    }

    /// Whether a capture has occurred.
    #[inline]
    pub fn is_captured(&self, channel: Channel) -> bool {
        Self::check_channel(channel);
        T::event_flag(channel_event(channel))
    }

    /// Clear the capture flag.
    #[inline]
    pub fn clear_capture(&self, channel: Channel) {
        Self::check_channel(channel);
        T::event_clear(channel_event(channel));
    }

    /// Whether an over-capture has occurred.
    #[inline]
    pub fn is_overcaptured(&self, channel: Channel) -> bool {
        Self::check_channel(channel);
        T::event_flag(overcapture_event(channel))
    }

    /// Clear the over-capture flag.
    #[inline]
    pub fn clear_overcapture(&self, channel: Channel) {
        Self::check_channel(channel);
        T::event_clear(overcapture_event(channel));
    }

    /// Set the period (ARR), used to limit the counter ceiling.
    #[inline]
    pub fn set_period(&self, arr: u16) {
        T::set_auto_reload(arr);
    }

    /// Read the current counter value.
    #[inline]
    pub fn now(&self) -> u16 {
        T::get_cnt()
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
