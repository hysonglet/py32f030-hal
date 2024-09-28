use super::*;
use crate::gpio::{self, AnyPin};
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

/// 计数器
///
/// 使用向上计数模式
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

// pub struct PwmChannel<'d, T: Instance> {
//     _id: Channel,
//     _t: PhantomData<&'d T>,

//     ch_pin: Option<PeripheralRef<'d, AnyPin>>,
// }

impl<'d, T: Instance> Pwm<'d, T> {
    pub fn new() -> Self {
        T::set_dir(CountDirection::Up);
        T::set_enable_channel(Channel::CH1, false);
        T::set_channel_output_config(Channel::CH1, ChannelOutputMode::PWM1, false, false, true);
        T::set_channel_output_effective_level(Channel::CH1, true, false);
        T::set_channel_type(Channel::CH1, ChannelType::Pwm);
        T::enable_auto_reload_buff(true);
        T::enable_channel_output(true);

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
    pub fn set_channel_1_pin<OC_PIN, OC_N_PIN>(
        &mut self,
        oc_pin: Option<OC_PIN>,
        oc_n_pin: Option<OC_N_PIN>,
    ) where
        OC_PIN: TimerChannel1Pin<T> + 'd,
        OC_N_PIN: TimerChannel1NPin<T>,
    {
        let oc_pin = oc_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        let oc_n_pin = oc_n_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        self._channel_1_pin = oc_pin;
        self._channel_1_n_pin = oc_n_pin;
    }

    pub fn set_channel_2_pin<OC_PIN, OC_N_PIN>(
        &mut self,
        oc_pin: Option<OC_PIN>,
        oc_n_pin: Option<OC_N_PIN>,
    ) where
        OC_PIN: TimerChannel2Pin<T> + 'd,
        OC_N_PIN: TimerChannel2NPin<T> + 'd,
    {
        let oc_pin = oc_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        let oc_n_pin = oc_n_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        self._channel_2_pin = oc_pin;
        self._channel_2_n_pin = oc_n_pin;
    }

    pub fn set_channel_3_pin<OC_PIN, OC_N_PIN>(
        &mut self,
        oc_pin: Option<OC_PIN>,
        oc_n_pin: Option<OC_N_PIN>,
    ) where
        OC_PIN: TimerChannel3Pin<T> + 'd,
        OC_N_PIN: TimerChannel3NPin<T> + 'd,
    {
        let oc_pin = oc_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        let oc_n_pin = oc_n_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        self._channel_3_pin = oc_pin;
        self._channel_3_n_pin = oc_n_pin;
    }

    pub fn set_channel_4_pin<OC_PIN, OC_N_PIN>(&mut self, oc_pin: Option<OC_PIN>)
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
        T::set_enable_channel(channel, true)
    }

    pub fn disable(&mut self, channel: Channel) {
        T::set_enable_channel(channel, false)
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
        defmt::info!("psc: {}", pre - 1);
        T::set_prescaler(pre as u16 - 1);
    }

    fn get_period(&self) -> u16 {
        T::get_reload()
    }

    fn set_duty(&mut self, channel: Channel, duty: u16) {
        let duty = if duty > self.get_max_duty() {
            self.get_max_duty()
        } else {
            duty
        };
        T::set_channel_compare(channel, duty);
    }

    fn set_period<P>(&mut self, period: u16) {
        T::set_auto_reload(period)
    }

    pub fn start(&mut self) {
        T::enable_channel_output(true);
        T::start();
    }

    pub fn stop(&mut self) {
        T::enable_channel_output(false);
        T::stop()
    }

    pub fn debug(&self) {
        // defmt::info!("reload: {}", T::get_reload());
        // T::triggle(Triggle::CC1G);
        // defmt::info!(
        //     "cnt: {:04x} ccr1: {}  u: {} CC1IE: {}",
        //     T::get_cnt(),
        //     T::block().ccr1.read().bits(),
        //     T::event_flag(Event::UIF),
        //     T::event_flag(Event::CC1IF)
        // );
        // if T::event_flag(Event::CC1IF) {
        //     // T::event_clear(Event::UIF);
        //     T::event_clear(Event::CC1IF);
        // }
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
        self.set_period::<u16>(period.into())
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
