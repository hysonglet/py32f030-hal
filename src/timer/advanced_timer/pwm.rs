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

pub struct PwmChannel<'d, T: Instance> {
    _id: Channel,
    _t: PhantomData<&'d T>,

    ch_pin: Option<PeripheralRef<'d, AnyPin>>,
}

impl<'d, T: Instance> Pwm<'d, T> {
    pub fn new(// channel_1_pin: Option<impl Peripheral<P = impl TimerChannel1Pin<T>> + 'd>,
        // channel_1_n_pin: Option<impl Peripheral<P = impl TimerChannel1NPin<T>> + 'd>,
        // channel_2_pin: Option<impl Peripheral<P = impl TimerChannel2Pin<T>> + 'd>,
        // channel_2_n_pin: Option<impl Peripheral<P = impl TimerChannel2NPin<T>> + 'd>,
        // channel_3_pin: Option<impl Peripheral<P = impl TimerChannel3Pin<T>> + 'd>,
        // channel_3_n_pin: Option<impl Peripheral<P = impl TimerChannel3NPin<T>> + 'd>,
        // channel_4_pin: Option<impl Peripheral<P = impl TimerChannel4Pin<T>> + 'd>,
    ) -> Self {
        T::set_dir(CountDirection::Up);
        T::set_enable_channel(Channel::CH1, false);
        T::set_channel_output_config(Channel::CH1, ChannelOutputMode::PWM1, false, false, false);
        T::set_channel_type(Channel::CH1, ChannelType::Pwm);
        T::set_channel_output_effective_level(Channel::CH1, true);
        T::enable_auto_reload_buff(true);
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
        T::start();
    }

    pub fn stop(&mut self) {
        T::stop()
    }

    pub fn debug(&self) {
        defmt::info!("cnt: {}", T::get_cnt());
        defmt::info!("reload: {}", T::get_reload());
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

impl<'d, T: Instance> embedded_hal::PwmPin for PwmChannel<'d, T> {
    type Duty = u16;
    fn enable(&mut self) {
        todo!()
    }

    fn disable(&mut self) {
        todo!()
    }

    fn get_duty(&self) -> Self::Duty {
        todo!()
    }

    fn set_duty(&mut self, duty: Self::Duty) {
        todo!()
    }

    fn get_max_duty(&self) -> Self::Duty {
        todo!()
    }
}
