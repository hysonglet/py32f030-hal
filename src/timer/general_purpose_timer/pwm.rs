use super::*;
use crate::gpio::{self, AnyPin};
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, PeripheralRef};

/// PWM
///
pub struct Pwm<'d, T: Instance> {
    _t: PhantomData<&'d T>,

    _channel_1_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_2_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_3_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_4_pin: Option<PeripheralRef<'d, AnyPin>>,
}

impl<'d, T: Instance> Default for Pwm<'d, T> {
    fn default() -> Self {
        Self {
            _t: PhantomData,
            _channel_1_pin: None,
            _channel_2_pin: None,
            _channel_3_pin: None,
            _channel_4_pin: None,
        }
    }
}

impl<'d, T: Instance> Pwm<'d, T> {
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
        T::set_enable_channel(channel, false);
        T::set_enable_channel(channel, false);
        T::set_channel_output_config(
            channel,
            config.mode,
            config.clear,
            config.fast,
            config.preload,
        );
        T::set_channel_type(channel, ChannelType::Pwm);
    }

    pub fn new() -> Self {
        Default::default()
    }
}

impl<'d, T: Instance> Pwm<'d, T> {
    pub fn set_channel_1_pin<OC_PIN>(&mut self, oc_pin: Option<OC_PIN>)
    where
        OC_PIN: TimerChannel1Pin<T> + 'd,
    {
        let oc_pin = oc_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        self._channel_1_pin = oc_pin;
    }

    pub fn set_channel_2_pin<OC_PIN>(&mut self, oc_pin: Option<OC_PIN>)
    where
        OC_PIN: TimerChannel2Pin<T> + 'd,
    {
        let oc_pin = oc_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        self._channel_2_pin = oc_pin;
    }

    pub fn set_channel_3_pin<OC_PIN>(&mut self, oc_pin: Option<OC_PIN>)
    where
        OC_PIN: TimerChannel3Pin<T> + 'd,
    {
        let oc_pin = oc_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        self._channel_3_pin = oc_pin;
    }

    pub fn set_channel_4_pin<OC_PIN>(&mut self, oc_pin: Option<OC_PIN>)
    where
        OC_PIN: TimerChannel4Pin<T> + 'd,
    {
        let oc_pin = oc_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        self._channel_4_pin = oc_pin;
    }
}

impl<'d, T: Instance> Pwm<'d, T> {
    pub fn enable(&mut self, channel: Channel) {
        T::set_enable_channel(channel, true);
        T::set_enable_channel(channel, true);
    }

    pub fn disable(&mut self, channel: Channel) {
        T::set_enable_channel(channel, false);
        T::set_enable_channel(channel, false);
    }

    pub fn get_duty(&self, channel: Channel) -> u16 {
        T::get_channel_capture(channel)
    }

    pub fn get_max_duty(&self) -> u16 {
        T::get_reload()
    }

    /// 设置计数频率（该频率为计数器的频率，并非波形的频率）
    pub fn set_frequency(&mut self, freq: u32) {
        let freq = if T::get_time_pclk() <= freq {
            T::get_time_pclk()
        } else {
            freq
        };

        let pre = T::get_time_pclk() / freq;
        T::set_prescaler(pre as u16 - 1);
    }

    pub fn get_period(&self) -> u16 {
        T::get_reload()
    }

    pub fn set_duty(&mut self, channel: Channel, duty: u16) {
        let duty = if duty > self.get_max_duty() {
            self.get_max_duty()
        } else {
            duty
        };
        T::set_channel_compare(channel, duty);
    }

    pub fn set_period(&mut self, period: u16) {
        T::set_auto_reload(period)
    }

    pub fn start(&mut self) {
        T::start();
    }

    pub fn stop(&mut self) {
        T::stop()
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

impl<'d, T: Instance> embedded_hal::Pwm for Pwm<'d, T> {
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

// impl<'d, T: Instance> embedded_hal::PwmPin for PwmChannel<'d, T> {
//     type Duty = u16;
//     fn enable(&mut self) {
//         todo!()
//     }

//     fn disable(&mut self) {
//         todo!()
//     }

//     fn get_duty(&self) -> Self::Duty {
//         todo!()
//     }

//     fn set_duty(&mut self, duty: Self::Duty) {
//         todo!()
//     }

//     fn get_max_duty(&self) -> Self::Duty {
//         todo!()
//     }
// }
