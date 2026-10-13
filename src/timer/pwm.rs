use super::{
    BreakConfig, Channel, ChannelConfig, ChannelMode, ChannelOutput, ChannelOutputConfig,
    ChannelType, Event, Instance, WithBreak, WithComplementary, WithRepetition,
};
use crate::gpio::{self, AnyPin};
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, PeripheralRef};

/// Unified PWM.
///
/// TIM1 / TIM3 / TIM14 / TIM16 share a single implementation. Channels are
/// selected via [`Channel`]; an out-of-range channel triggers an `assert`
/// (validated by `ChannelCount::CHANNELS`).
///
/// Timer-specific capabilities are gated by capability traits:
/// - Main output pins only: `set_channel_N_pin` (all timers)
/// - With complementary output pins: `set_channel_N_pins` (TIM1 / TIM16 only, `WithComplementary`)
/// - Break / dead-time: `enable_break` (`WithBreak`)
/// - Repetition counter: `set_repetition` (`WithRepetition`)
pub struct Pwm<'d, T: Instance> {
    _t: PhantomData<&'d T>,

    _channel_1_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_1_n_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_2_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_2_n_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_3_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_3_n_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_4_pin: Option<PeripheralRef<'d, AnyPin>>,
}

impl<'d, T: Instance> Default for Pwm<'d, T> {
    fn default() -> Self {
        Self {
            _t: PhantomData,
            _channel_1_pin: None,
            _channel_1_n_pin: None,
            _channel_2_pin: None,
            _channel_2_n_pin: None,
            _channel_3_pin: None,
            _channel_3_n_pin: None,
            _channel_4_pin: None,
        }
    }
}

impl<'d, T: Instance> Pwm<'d, T> {
    pub(super) fn new() -> Self {
        Default::default()
    }

    /// Check whether the channel exists on this timer.
    #[inline]
    fn check_channel(channel: Channel) {
        assert!(
            channel.index() < T::CHANNELS,
            "this timer does not support this channel (available channels = {})",
            T::CHANNELS
        );
    }

    /// Configure up to 4 channels at once (None means the channel is disabled).
    pub fn config(
        &mut self,
        channel_1_config: Option<ChannelConfig>,
        channel_2_config: Option<ChannelConfig>,
        channel_3_config: Option<ChannelConfig>,
        channel_4_config: Option<ChannelConfig>,
    ) -> Self {
        if let Some(config) = channel_1_config {
            Self::channel_config(Channel::CH1, config)
        }
        if let Some(config) = channel_2_config {
            Self::channel_config(Channel::CH2, config)
        }
        if let Some(config) = channel_3_config {
            Self::channel_config(Channel::CH3, config)
        }
        if let Some(config) = channel_4_config {
            Self::channel_config(Channel::CH4, config)
        }

        T::enable_auto_reload_buff(true);
        Default::default()
    }

    fn channel_config(channel: Channel, config: ChannelConfig) {
        Self::check_channel(channel);

        T::set_enable_channel(channel, ChannelOutput::P, false);
        T::set_enable_channel(channel, ChannelOutput::N, false);
        T::set_channel_output_config(
            channel,
            config.mode,
            config.clear,
            config.fast,
            config.preload,
        );
        T::set_channel_type(channel, ChannelType::Pwm);
        T::set_channel_compare(channel, config.compare);

        if let Some(ch) = config.ch {
            T::set_channel_output_effective_level(
                channel,
                ChannelOutput::P,
                ch.polarity,
                ch.state,
                ch.idle_state,
            );
        }

        if let Some(ch) = config.n_ch {
            T::set_channel_output_effective_level(
                channel,
                ChannelOutput::N,
                ch.polarity,
                ch.state,
                ch.idle_state,
            );
        }
    }


    pub fn set_channel_1_pin<OC_PIN>(&mut self, oc_pin: Option<OC_PIN>)
    where
        OC_PIN: super::TimerChannel1Pin<T> + 'd,
    {
        self._channel_1_pin = oc_pin.map(|pin| {
            into_ref!(pin);
            pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
            pin.map_into()
        });
    }

    pub fn set_channel_2_pin<OC_PIN>(&mut self, oc_pin: Option<OC_PIN>)
    where
        OC_PIN: super::TimerChannel2Pin<T> + 'd,
    {
        self._channel_2_pin = oc_pin.map(|pin| {
            into_ref!(pin);
            pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
            pin.map_into()
        });
    }

    pub fn set_channel_3_pin<OC_PIN>(&mut self, oc_pin: Option<OC_PIN>)
    where
        OC_PIN: super::TimerChannel3Pin<T> + 'd,
    {
        self._channel_3_pin = oc_pin.map(|pin| {
            into_ref!(pin);
            pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
            pin.map_into()
        });
    }

    /// CH4 has no complementary output.
    pub fn set_channel_4_pin<OC_PIN>(&mut self, oc_pin: Option<OC_PIN>)
    where
        OC_PIN: super::TimerChannel4Pin<T> + 'd,
    {
        self._channel_4_pin = oc_pin.map(|pin| {
            into_ref!(pin);
            pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
            pin.map_into()
        });
    }


    /// Enable the channel main output.
    pub fn enable(&mut self, channel: Channel) {
        Self::check_channel(channel);
        T::set_enable_channel(channel, ChannelOutput::P, true);
    }

    pub fn disable(&mut self, channel: Channel) {
        Self::check_channel(channel);
        T::set_enable_channel(channel, ChannelOutput::P, false);
    }

    #[inline]
    pub fn get_duty(&self, channel: Channel) -> u16 {
        Self::check_channel(channel);
        T::get_channel_capture(channel)
    }

    #[inline]
    pub fn get_max_duty(&self) -> u16 {
        T::get_reload()
    }

    /// Set the counting frequency (counter clock frequency, not the waveform frequency).
    pub fn set_frequency(&mut self, freq: u32) {
        let clk = T::get_time_pclk();
        let freq = if clk <= freq { clk } else { freq };
        let pre = clk / freq;
        T::set_prescaler((pre as u16).saturating_sub(1));
    }

    /// Return the current counting frequency.
    #[inline]
    pub fn get_freq(&self) -> u32 {
        T::counter_frequency()
    }

    #[inline]
    pub fn set_period(&mut self, period: u16) {
        T::set_auto_reload(period)
    }

    #[inline]
    pub fn get_period(&self) -> u16 {
        T::get_reload()
    }

    #[inline]
    pub fn set_duty(&mut self, channel: Channel, duty: u16) {
        Self::check_channel(channel);
        let duty = if duty > self.get_max_duty() {
            self.get_max_duty()
        } else {
            duty
        };
        T::set_channel_compare(channel, duty);
    }

    /// Start (and enable the main output MOE; a no-op for timers without BDTR).
    #[inline]
    pub fn start(&mut self) {
        T::enable_channel_output(true);
        T::start();
    }

    #[inline]
    pub fn stop(&mut self) {
        T::enable_channel_output(false);
        T::stop()
    }

    /// Wait for the update event (sync the shadow registers).
    #[inline]
    pub fn wait_update(&self) {
        while !T::event_flag(Event::UIF) {}
        T::event_clear(Event::UIF);
    }
}


impl<'d, T: Instance + WithComplementary> Pwm<'d, T> {
    /// Bind both CH1 and CH1N pins at once.
    pub fn set_channel_1_pins<OC_PIN, OC_N_PIN>(
        &mut self,
        oc_pin: Option<OC_PIN>,
        oc_n_pin: Option<OC_N_PIN>,
    ) where
        OC_PIN: super::TimerChannel1Pin<T> + 'd,
        OC_N_PIN: super::TimerChannel1NPin<T>,
    {
        self.set_channel_1_pin(oc_pin);
        self._channel_1_n_pin = oc_n_pin.map(|pin| {
            into_ref!(pin);
            pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
            pin.map_into()
        });
    }

    /// Bind both CH2 and CH2N pins at once.
    pub fn set_channel_2_pins<OC_PIN, OC_N_PIN>(
        &mut self,
        oc_pin: Option<OC_PIN>,
        oc_n_pin: Option<OC_N_PIN>,
    ) where
        OC_PIN: super::TimerChannel2Pin<T> + 'd,
        OC_N_PIN: super::TimerChannel2NPin<T>,
    {
        self.set_channel_2_pin(oc_pin);
        self._channel_2_n_pin = oc_n_pin.map(|pin| {
            into_ref!(pin);
            pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
            pin.map_into()
        });
    }

    /// Bind both CH3 and CH3N pins at once.
    pub fn set_channel_3_pins<OC_PIN, OC_N_PIN>(
        &mut self,
        oc_pin: Option<OC_PIN>,
        oc_n_pin: Option<OC_N_PIN>,
    ) where
        OC_PIN: super::TimerChannel3Pin<T> + 'd,
        OC_N_PIN: super::TimerChannel3NPin<T>,
    {
        self.set_channel_3_pin(oc_pin);
        self._channel_3_n_pin = oc_n_pin.map(|pin| {
            into_ref!(pin);
            pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
            pin.map_into()
        });
    }

    /// Configure the complementary output (CHxN) of a channel.
    pub fn channel_config_n(&self, channel: Channel, config: ChannelOutputConfig) {
        Self::check_channel(channel);
        T::set_channel_output_effective_level(
            channel,
            ChannelOutput::N,
            config.polarity,
            config.state,
            config.idle_state,
        );
    }

    /// Enable / disable the complementary output.
    #[inline]
    pub fn enable_channel_n(&self, channel: Channel, en: bool) {
        Self::check_channel(channel);
        T::set_enable_channel(channel, ChannelOutput::N, en);
    }

    /// Configure the main and complementary outputs together (complementary inverted).
    pub fn channel_config_complementary(&self, channel: Channel, mode: ChannelMode, duty: u16) {
        Self::check_channel(channel);
        T::set_channel_type(channel, ChannelType::Pwm);
        T::set_channel_output_config(channel, mode, false, false, true);
        T::set_channel_compare(channel, duty);
        T::set_channel_output_effective_level(channel, ChannelOutput::P, true, true, false);
        T::set_channel_output_effective_level(channel, ChannelOutput::N, true, true, false);
        T::set_enable_channel(channel, ChannelOutput::N, true);
    }
}


impl<'d, T: Instance + WithBreak> Pwm<'d, T> {
    /// Configure break & dead-time and enable the main output (MOE).
    pub fn enable_break(&self, config: BreakConfig) {
        T::config_break_deadtime(
            config.dead_time,
            config.lock_level,
            config.off_state_run,
            config.off_state_idle,
            config.break_polarity,
            config.break_enable,
            config.automatic_output,
        );
    }

    /// Enable / disable the main output (BDTR.MOE).
    #[inline]
    pub fn enable_main_output(&self, en: bool) {
        T::enable_channel_output(en);
    }
}


impl<'d, T: Instance + WithRepetition> Pwm<'d, T> {
    /// Set the repetition counter value.
    #[inline]
    pub fn set_repetition(&self, rep: u8) {
        T::set_repetition(rep);
    }
}


impl<'d, T: Instance> embedded_hal_027::Pwm for Pwm<'d, T> {
    type Channel = Channel;
    type Duty = u16;
    type Time = u16;

    fn enable(&mut self, channel: Self::Channel) {
        self.enable(channel)
    }

    fn disable(&mut self, channel: Self::Channel) {
        self.disable(channel)
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        self.get_duty(channel)
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.get_max_duty()
    }

    fn get_period(&self) -> Self::Time {
        self.get_period()
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        self.set_duty(channel, duty)
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        self.set_period(period.into())
    }
}
